//! WASM-specific response types
//!
//! This module provides the CANONICAL WasmResponse implementation that consolidates
//! all WASM-specific response handling for browser environments.

use std::fmt;

use bytes::Bytes;
use ystream::prelude::*;
use http::{HeaderMap, StatusCode, Version};
use serde::de::DeserializeOwned;
use url::Url;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, ArrayBuffer, Uint8Array};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys;

use crate::http::response::{HttpResponse, HttpBodyChunk};
use crate::telemetry::ClientStatsSnapshot;

/// WASM-specific response wrapper that extends HttpResponse with browser capabilities
/// 
/// This is the CANONICAL WasmResponse implementation that consolidates all
/// WASM-specific response handling into a single, comprehensive type.
pub struct WasmResponse {
    /// Core streaming response
    pub inner: HttpResponse,
    
    /// WASM-specific data
    #[cfg(target_arch = "wasm32")]
    pub web_response: Option<web_sys::Response>,
    #[cfg(target_arch = "wasm32")]
    pub abort_controller: Option<web_sys::AbortController>,
    
    /// Response URL
    pub url: Url,
    
    /// WASM-specific metadata
    pub redirected: bool,
    pub response_type: String,
    
    /// Error message for MessageChunk implementation
    pub error_message: Option<String>,
    
    /// Non-WASM fallback
    #[cfg(not(target_arch = "wasm32"))]
    pub web_response: Option<String>,
}

/// WASM response types
#[derive(Debug, Clone, Copy)]
pub enum WasmResponseType {
    Basic,
    Cors,
    Error,
    Opaque,
    OpaqueRedirect,
}

impl fmt::Debug for WasmResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WasmResponse")
            .field("status", &self.inner.status)
            .field("url", &self.url)
            .field("redirected", &self.redirected)
            .field("response_type", &self.response_type)
            .finish()
    }
}

impl WasmResponse {
    /// Create new WASM response from HttpResponse
    #[inline]
    pub fn new(inner: HttpResponse, url: Url) -> Self {
        Self {
            inner,
            #[cfg(target_arch = "wasm32")]
            web_response: None,
            #[cfg(target_arch = "wasm32")]
            abort_controller: None,
            url,
            redirected: false,
            response_type: "basic".to_string(),
            error_message: None,
            #[cfg(not(target_arch = "wasm32"))]
            web_response: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// Create WASM response from web_sys::Response
    pub fn from_web_response(
        web_response: web_sys::Response,
        url: Url,
        abort_controller: Option<web_sys::AbortController>,
    ) -> Result<Self, JsValue> {
        let status = StatusCode::from_u16(web_response.status())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        let mut headers = HeaderMap::new();
        
        // Extract headers from web response
        if let Ok(headers_iter) = web_response.headers().entries() {
            let mut iter = js_sys::try_iter(&headers_iter)?
                .ok_or_else(|| JsValue::from_str("Headers iterator failed"))?;
            
            while let Some(entry) = iter.next()? {
                if let Ok(array) = entry.dyn_into::<js_sys::Array>() {
                    if array.length() >= 2 {
                        if let (Some(key), Some(value)) = (array.get(0).as_string(), array.get(1).as_string()) {
                            if let (Ok(header_name), Ok(header_value)) = 
                                (key.parse::<http::HeaderName>(), value.parse::<http::HeaderValue>()) {
                                headers.insert(header_name, header_value);
                            }
                        }
                    }
                }
            }
        }

        // Create response chunks from web response body
        let response_chunks = Self::create_chunks_from_web_response(&web_response)?;
        
        let http_response = HttpResponse::from_http2_response(
            status,
            headers,
            response_chunks,
            0, // stream_id - 0 for WASM since it's not multiplexed
        );

        let response_type = match web_response.type_() {
            web_sys::ResponseType::Basic => WasmResponseType::Basic,
            web_sys::ResponseType::Cors => WasmResponseType::Cors,
            web_sys::ResponseType::Error => WasmResponseType::Error,
            web_sys::ResponseType::Opaque => WasmResponseType::Opaque,
            web_sys::ResponseType::Opaqueredirect => WasmResponseType::OpaqueRedirect,
            _ => WasmResponseType::Basic,
        };

        Ok(WasmResponse {
            inner: http_response,
            web_response: Some(web_response),
            abort_controller,
            url,
            redirected: web_response.redirected(), // Extract redirected status from web response
            response_type: response_type.to_string(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    /// Create response chunks from web_sys::Response - NO Result wrapping
    fn create_chunks_from_web_response(
        web_response: &web_sys::Response,
    ) -> AsyncStream<HttpBodyChunk, 1024> {
        let body = web_response.body();
        
        if let Some(readable_stream) = body {
            let reader = readable_stream.get_reader();
            
            AsyncStream::with_channel(move |sender| {
                use std::sync::Arc;
                use std::sync::atomic::{AtomicU64, Ordering};
                
                // Create shared offset tracker
                let offset_tracker = Arc::new(AtomicU64::new(0));
                
                // Use callback-based stream reading instead of Future-based approach
                fn read_next_chunk(
                    reader: web_sys::ReadableStreamDefaultReader, 
                    sender: ystream::AsyncStreamSender<HttpBodyChunk>,
                    offset_tracker: Arc<AtomicU64>
                ) {
                    use wasm_bindgen::prelude::*;
                    
                    let read_promise = reader.read();
                    
                    let success_callback = Closure::once_into_js({
                        let reader = reader.clone();
                        let sender = sender.clone();
                        let offset_tracker = offset_tracker.clone();
                        move |chunk: JsValue| {
                            let chunk_obj = match chunk.dyn_into::<js_sys::Object>() {
                                Ok(obj) => obj,
                                Err(_) => {
                                    emit!(sender, HttpBodyChunk::bad_chunk("Failed to convert chunk to Object".to_string()));
                                    return;
                                }
                            };
                            
                            // Check if done
                            let done = js_sys::Reflect::get(&chunk_obj, &JsValue::from_str("done"))
                                .unwrap_or(JsValue::FALSE)
                                .as_bool()
                                .unwrap_or(false);
                            
                            if done {
                                // Emit final chunk with current offset
                                let final_offset = offset_tracker.load(Ordering::Acquire);
                                emit!(sender, HttpBodyChunk::new(
                                    Bytes::new(),
                                    final_offset,
                                    true // is_final
                                ));
                                return;
                            }
                            
                            // Get value
                            if let Ok(value) = js_sys::Reflect::get(&chunk_obj, &JsValue::from_str("value")) {
                                if let Ok(uint8_array) = value.dyn_into::<js_sys::Uint8Array>() {
                                    let mut bytes = vec![0; uint8_array.length() as usize];
                                    uint8_array.copy_to(&mut bytes);
                                    
                                    // Get current offset and update atomically
                                    let current_offset = offset_tracker.fetch_add(
                                        bytes.len() as u64,
                                        Ordering::SeqCst
                                    );
                                    
                                    emit!(sender, HttpBodyChunk::new(
                                        Bytes::from(bytes),
                                        current_offset,
                                        false // is_final
                                    ));
                                }
                            }
                            
                            // Read next chunk recursively
                            read_next_chunk(reader, sender, offset_tracker);
                        }
                    });
                    
                    let error_callback = Closure::once_into_js(move |_error: JsValue| {
                        emit!(sender, HttpBodyChunk::bad_chunk("Stream read error".to_string()));
                    });
                    
                    // Use native JavaScript Promise.then() method
                    let _ = read_promise.then2(&success_callback, &error_callback);
                }
                
                // Start reading the first chunk
                read_next_chunk(reader, sender, offset_tracker);
            })
        } else {
            // No body - return empty stream
            AsyncStream::with_channel(|_sender| {
                // Empty body - no chunks to emit
            })
        }
    }

    // Delegate core methods to inner HttpResponse

    /// Get the status code
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Get the headers - NOTE: This requires consuming the headers stream
    #[inline]
    pub async fn headers(&mut self) -> HeaderMap {
        self.inner.collect_headers().await
    }

    /// Get the HTTP version
    #[inline]
    pub fn version(&self) -> Version {
        self.inner.version
    }

    /// Get the URL
    #[inline]
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Check if response was redirected
    #[inline]
    pub fn redirected(&self) -> bool {
        self.redirected
    }

    /// Get response type
    #[inline]
    pub fn response_type(&self) -> &str {
        &self.response_type
    }

    /// Check if response is successful
    #[inline]
    pub fn is_success(&self) -> bool {
        self.inner.is_success()
    }

    /// Check if response is redirect
    #[inline]
    pub fn is_redirect(&self) -> bool {
        self.inner.is_redirect()
    }

    /// Check if response is client error
    #[inline]
    pub fn is_client_error(&self) -> bool {
        self.inner.is_client_error()
    }

    /// Check if response is server error
    #[inline]
    pub fn is_server_error(&self) -> bool {
        self.inner.is_server_error()
    }

    /// Get next response chunk
    #[inline]
    pub async fn try_next(&mut self) -> Option<HttpBodyChunk> {
        self.inner.body_stream.next().await
    }

    /// Collect response body as bytes
    pub async fn collect_bytes(mut self) -> Vec<u8> {
        let body = self.inner.collect_body().await;
        body.to_vec()
    }

    /// Collect response body as string
    pub async fn collect_string(mut self) -> Result<String, std::string::FromUtf8Error> {
        let body = self.inner.collect_body().await;
        String::from_utf8(body.to_vec())
    }

    /// Collect and deserialize JSON
    pub async fn collect_json<T>(mut self) -> Result<T, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        let body = self.inner.collect_body().await;
        serde_json::from_slice(&body)
    }

    /// Get response statistics
    #[inline]
    pub fn stats(&self) -> ClientStatsSnapshot {
        // Return default stats since HttpResponse doesn't have stats method
        ClientStatsSnapshot::default()
    }

    #[cfg(target_arch = "wasm32")]
    /// Get the underlying web_sys::Response
    #[inline]
    pub fn web_response(&self) -> Option<&web_sys::Response> {
        self.web_response.as_ref()
    }

    #[cfg(target_arch = "wasm32")]
    /// Clone the web response (if available)
    pub fn clone_web_response(&self) -> Option<web_sys::Response> {
        self.web_response.as_ref().and_then(|resp| resp.clone().ok())
    }

    #[cfg(target_arch = "wasm32")]
    /// Abort the response (if abort controller is available)
    pub fn abort(&self) {
        if let Some(controller) = &self.abort_controller {
            controller.abort();
        }
    }

    /// Convert from HttpResponse
    #[inline]
    pub fn from_http_response(response: HttpResponse, url: Url) -> Self {
        Self::new(response, url)
    }

    /// Convert to HttpResponse
    #[inline]
    pub fn into_http_response(self) -> HttpResponse {
        self.inner
    }

    /// Get reference to inner HttpResponse
    #[inline]
    pub fn as_http_response(&self) -> &HttpResponse {
        &self.inner
    }

    /// Get mutable reference to inner HttpResponse
    #[inline]
    pub fn as_http_response_mut(&mut self) -> &mut HttpResponse {
        &mut self.inner
    }

    /// Transform response chunks with a mapping function
    pub fn map_chunks<F, T>(self, mapper: F) -> AsyncStream<T, 1024>
    where
        F: Fn(HttpBodyChunk) -> T + Send + 'static,
        T: Send + 'static,
    {
        use ystream::AsyncStream;
        
        AsyncStream::with_channel(move |sender| {
            // Transform the body stream - use thread executor for async
            ystream::thread_pool::global_executor().execute(move || {
                let mut body_stream = self.inner.body_stream;
                loop {
                    match body_stream.try_next() {
                        Some(chunk) => {
                            let transformed = mapper(chunk);
                            ystream::emit!(sender, transformed);
                        }
                        None => break,
                    }
                }
            });
        })
    }

    /// Filter response chunks based on predicate
    pub fn filter_chunks<F>(self, predicate: F) -> AsyncStream<HttpBodyChunk, 1024>
    where
        F: Fn(&HttpBodyChunk) -> bool + Send + 'static,
    {
        use ystream::AsyncStream;
        
        AsyncStream::with_channel(move |sender| {
            // Filter the body stream - use thread executor for async
            ystream::thread_pool::global_executor().execute(move || {
                let mut body_stream = self.inner.body_stream;
                loop {
                    match body_stream.try_next() {
                        Some(chunk) => {
                            if predicate(&chunk) {
                                ystream::emit!(sender, chunk);
                            }
                        }
                        None => break,
                    }
                }
            });
        })
    }
}

/// WASM response builder for ergonomic construction
#[derive(Debug)]
pub struct WasmResponseBuilder {
    status: StatusCode,
    headers: HeaderMap,
    version: Version,
    url: Url,
    response_type: WasmResponseType,
    redirected: bool,
}

impl WasmResponseBuilder {
    /// Create new builder
    #[inline]
    pub fn new(url: Url) -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            version: Version::HTTP_11,
            url,
            response_type: "basic".to_string(),
            redirected: false,
        }
    }

    /// Set status code
    #[inline]
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Add header
    #[inline]
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<http::HeaderName>,
        V: TryInto<http::HeaderValue>,
    {
        if let (Ok(name), Ok(val)) = (key.try_into(), value.try_into()) {
            self.headers.insert(name, val);
        }
        self
    }

    /// Set HTTP version
    #[inline]
    pub fn version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    /// Set response type
    #[inline]
    pub fn response_type(mut self, response_type: WasmResponseType) -> Self {
        self.response_type = response_type;
        self
    }

    /// Set redirected flag
    #[inline]
    pub fn redirected(mut self, redirected: bool) -> Self {
        self.redirected = redirected;
        self
    }

    /// Build WasmResponse with chunk stream
    pub fn build_with_chunks(self, chunks: AsyncStream<HttpBodyChunk, 1024>) -> WasmResponse {
        let http_response = HttpResponse::from_http2_response(
            self.status,
            self.headers,
            chunks,
            0, // stream_id
        );

        let mut wasm_response = WasmResponse::new(http_response, self.url);
        wasm_response.response_type = self.response_type.to_string();
        wasm_response.redirected = self.redirected;
        wasm_response
    }

    /// Build WasmResponse with body bytes
    pub fn build_with_bytes(self, body: Vec<u8>) -> WasmResponse {
        let chunks = AsyncStream::with_channel(move |sender| {
            if !body.is_empty() {
                ystream::emit!(sender, HttpBodyChunk::new(
                    Bytes::from(body),
                    0, // offset
                    true, // is_final
                ));
            }
        });

        self.build_with_chunks(chunks)
    }

    /// Build WasmResponse with JSON body
    pub fn build_with_json<T: serde::Serialize>(
        self,
        json: &T,
    ) -> Result<WasmResponse, serde_json::Error> {
        let body = serde_json::to_vec(json)?;
        Ok(self.build_with_bytes(body))
    }
}

// Implement MessageChunk for WasmResponse to support fluent-ai streams-first architecture
impl ystream::prelude::MessageChunk for WasmResponse {
    fn bad_chunk(error: String) -> Self {
        use http::StatusCode;
        
        // Create an error HttpResponse
        let error_response = HttpResponse::error(
            StatusCode::INTERNAL_SERVER_ERROR,
            error.clone(),
        );
            
        WasmResponse {
            inner: error_response,
            #[cfg(target_arch = "wasm32")]
            web_response: None,
            #[cfg(target_arch = "wasm32")]
            abort_controller: None,
            url: url::Url::parse("https://error.local").unwrap_or_else(|_| {
                // This should never fail, but provide absolute fallback
                url::Url::parse("http://localhost/").unwrap_or_else(|parse_error| {
                    log::error!("All error URL parsing failed: {}", parse_error);
                    // Absolute last resort - return a synthetic URL
                    url::Url::parse("data:text/plain,url-error").expect("data URL must parse")
                })
            }),
            redirected: false,
            response_type: "error".to_string(),
            error_message: Some(error),
            #[cfg(not(target_arch = "wasm32"))]
            web_response: None,
        }
    }

    fn is_error(&self) -> bool {
        self.error_message.is_some() || self.inner.status().is_client_error() || self.inner.status().is_server_error()
    }

    fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

/// Type alias for WASM Response - enables pure streaming architecture
pub type Response = WasmResponse;