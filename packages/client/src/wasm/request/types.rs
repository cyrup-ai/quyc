//! WASM-specific request types
//!
//! This module provides the CANONICAL WasmRequest implementation that consolidates
//! all WASM-specific request variants for browser environments.

use std::time::Duration;

use bytes::Bytes;
use http::{Method, HeaderMap, HeaderName, HeaderValue};
use url::Url;
use ystream::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{Headers, Request as WebRequest, RequestCredentials, RequestCache, RequestMode};

use super::Body;
use crate::http::request::{HttpRequest, RequestBody, RequestAuth};

/// WASM-specific request wrapper that extends HttpRequest with browser capabilities
/// 
/// This is the CANONICAL WasmRequest implementation that consolidates all
/// WASM-specific request handling into a single, comprehensive type.
#[derive(Debug, Clone)]
pub struct WasmRequest {
    /// Core HTTP request
    pub inner: HttpRequest,
    
    /// WASM-specific options
    #[cfg(target_arch = "wasm32")]
    pub credentials: Option<RequestCredentials>,
    #[cfg(target_arch = "wasm32")]
    pub cache: Option<RequestCache>,
    #[cfg(target_arch = "wasm32")]
    pub mode: Option<RequestMode>,
    #[cfg(target_arch = "wasm32")]
    pub redirect: Option<web_sys::RequestRedirect>,
    #[cfg(target_arch = "wasm32")]
    pub referrer: Option<String>,
    #[cfg(target_arch = "wasm32")]
    pub referrer_policy: Option<web_sys::ReferrerPolicy>,
    #[cfg(target_arch = "wasm32")]
    pub integrity: Option<String>,
    #[cfg(target_arch = "wasm32")]
    pub keep_alive: bool,
    #[cfg(target_arch = "wasm32")]
    pub signal: Option<web_sys::AbortSignal>,
    
    /// Non-WASM fallback fields
    #[cfg(not(target_arch = "wasm32"))]
    pub credentials: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    pub cache: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    pub mode: Option<String>,
    
    /// Error message for MessageChunk implementation
    pub error_message: Option<String>,
}

impl WasmRequest {
    /// Create new WASM request from HttpRequest
    #[inline]
    pub fn new(inner: HttpRequest) -> Self {
        Self {
            inner,
            #[cfg(target_arch = "wasm32")]
            credentials: None,
            #[cfg(target_arch = "wasm32")]
            cache: None,
            #[cfg(target_arch = "wasm32")]
            mode: None,
            #[cfg(target_arch = "wasm32")]
            redirect: None,
            #[cfg(target_arch = "wasm32")]
            referrer: None,
            #[cfg(target_arch = "wasm32")]
            referrer_policy: None,
            #[cfg(target_arch = "wasm32")]
            integrity: None,
            #[cfg(target_arch = "wasm32")]
            keep_alive: false,
            #[cfg(target_arch = "wasm32")]
            signal: None,
            #[cfg(not(target_arch = "wasm32"))]
            credentials: None,
            #[cfg(not(target_arch = "wasm32"))]
            cache: None,
            #[cfg(not(target_arch = "wasm32"))]
            mode: None,
            error_message: None,
        }
    }

    /// Create WASM request from method and URL
    #[inline]
    pub fn from_parts(method: Method, url: Url) -> Self {
        Self::new(HttpRequest::new(method, url, None, None, None))
    }

    /// Create GET request
    #[inline]
    pub fn get<U: TryInto<Url>>(url: U) -> Result<Self, U::Error> {
        let url = url.try_into()?;
        Ok(Self::new(HttpRequest::new(Method::GET, url, None, None, None)))
    }

    /// Create POST request
    #[inline]
    pub fn post<U: TryInto<Url>>(url: U) -> Result<Self, U::Error> {
        let url = url.try_into()?;
        Ok(Self::new(HttpRequest::new(Method::POST, url, None, None, None)))
    }

    /// Create PUT request
    #[inline]
    pub fn put<U: TryInto<Url>>(url: U) -> Result<Self, U::Error> {
        let url = url.try_into()?;
        Ok(Self::new(HttpRequest::new(Method::PUT, url, None, None, None)))
    }

    /// Create DELETE request
    #[inline]
    pub fn delete<U: TryInto<Url>>(url: U) -> Result<Self, U::Error> {
        let url = url.try_into()?;
        Ok(Self::new(HttpRequest::new(Method::DELETE, url, None, None, None)))
    }

    // Delegate core HTTP methods to inner HttpRequest

    /// Get the method
    #[inline]
    pub fn method(&self) -> &Method {
        self.inner.method()
    }

    /// Get the URL
    #[inline]
    pub fn url(&self) -> &Url {
        self.inner.url()
    }

    /// Get the headers
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Get the body
    #[inline]
    pub fn body(&self) -> Option<&RequestBody> {
        self.inner.body()
    }

    /// Get the timeout
    #[inline]
    pub fn timeout(&self) -> Option<Duration> {
        self.inner.timeout()
    }

    // Builder methods for core HTTP functionality

    /// Set method
    #[inline]
    pub fn method(mut self, method: Method) -> Self {
        self.inner = self.inner.method(method);
        self
    }

    /// Set URL
    #[inline]
    pub fn url(mut self, url: Url) -> Self {
        self.inner = self.inner.url(url);
        self
    }

    /// Add header
    #[inline]
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
    {
        self.inner = self.inner.header(key, value);
        self
    }

    /// Set headers
    #[inline]
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.inner = self.inner.headers(headers);
        self
    }

    /// Set body as bytes
    #[inline]
    pub fn body_bytes<B: Into<Bytes>>(mut self, body: B) -> Self {
        self.inner = self.inner.body_bytes(body);
        self
    }

    /// Set body as text
    #[inline]
    pub fn body_text<S: Into<String>>(mut self, body: S) -> Self {
        self.inner = self.inner.body_text(body);
        self
    }

    /// Set body as JSON
    #[inline]
    pub fn json<T: serde::Serialize>(mut self, json: &T) -> Result<Self, serde_json::Error> {
        self.inner = self.inner.json(json)?;
        Ok(self)
    }

    /// Set timeout
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.inner = self.inner.timeout(timeout);
        self
    }

    /// Add query parameters
    #[inline]
    pub fn query<K, V>(mut self, params: &[(K, V)]) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.inner = self.inner.query(params);
        self
    }

    // WASM-specific builder methods

    #[cfg(target_arch = "wasm32")]
    /// Set request credentials
    #[inline]
    pub fn credentials(mut self, credentials: RequestCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    #[cfg(target_arch = "wasm32")]
    /// Set request cache mode
    #[inline]
    pub fn cache(mut self, cache: RequestCache) -> Self {
        self.cache = Some(cache);
        self
    }

    #[cfg(target_arch = "wasm32")]
    /// Set request mode
    #[inline]
    pub fn mode(mut self, mode: RequestMode) -> Self {
        self.mode = Some(mode);
        self
    }

    #[cfg(target_arch = "wasm32")]
    /// Set redirect handling
    #[inline]
    pub fn redirect(mut self, redirect: web_sys::RequestRedirect) -> Self {
        self.redirect = Some(redirect);
        self
    }

    #[cfg(target_arch = "wasm32")]
    /// Set referrer
    #[inline]
    pub fn referrer<S: Into<String>>(mut self, referrer: S) -> Self {
        self.referrer = Some(referrer.into());
        self
    }

    #[cfg(target_arch = "wasm32")]
    /// Set referrer policy
    #[inline]
    pub fn referrer_policy(mut self, policy: web_sys::ReferrerPolicy) -> Self {
        self.referrer_policy = Some(policy);
        self
    }

    #[cfg(target_arch = "wasm32")]
    /// Set integrity hash
    #[inline]
    pub fn integrity<S: Into<String>>(mut self, integrity: S) -> Self {
        self.integrity = Some(integrity.into());
        self
    }

    #[cfg(target_arch = "wasm32")]
    /// Set keep-alive
    #[inline]
    pub fn keep_alive(mut self, keep_alive: bool) -> Self {
        self.keep_alive = keep_alive;
        self
    }

    #[cfg(target_arch = "wasm32")]
    /// Set abort signal
    #[inline]
    pub fn signal(mut self, signal: web_sys::AbortSignal) -> Self {
        self.signal = Some(signal);
        self
    }

    // Non-WASM fallback methods

    #[cfg(not(target_arch = "wasm32"))]
    /// Set credentials (fallback for non-WASM)
    #[inline]
    pub fn credentials<S: Into<String>>(mut self, credentials: S) -> Self {
        self.credentials = Some(credentials.into());
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Set cache mode (fallback for non-WASM)
    #[inline]
    pub fn cache<S: Into<String>>(mut self, cache: S) -> Self {
        self.cache = Some(cache.into());
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Set request mode (fallback for non-WASM)
    #[inline]
    pub fn mode<S: Into<String>>(mut self, mode: S) -> Self {
        self.mode = Some(mode.into());
        self
    }

    // Conversion methods

    #[cfg(target_arch = "wasm32")]
    /// Convert to web_sys::Request
    pub fn to_web_request(&self) -> Result<WebRequest, wasm_bindgen::JsValue> {
        let mut init = web_sys::RequestInit::new();
        
        // Set method
        init.method(self.inner.method().as_str());

        // Set headers
        let headers = Headers::new()?;
        for (name, value) in self.inner.headers() {
            headers.set(name.as_str(), value.to_str().unwrap_or(""))?;
        }
        init.headers(&headers);

        // Set body if present
        if let Some(body) = self.inner.body() {
            match body {
                RequestBody::Bytes(bytes) => {
                    let array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
                    array.copy_from(bytes);
                    init.body(Some(&array));
                }
                RequestBody::Text(text) => {
                    init.body(Some(&wasm_bindgen::JsValue::from_str(text)));
                }
                RequestBody::Json(json) => {
                    if let Ok(json_str) = serde_json::to_string(json) {
                        init.body(Some(&wasm_bindgen::JsValue::from_str(&json_str)));
                    }
                }
                _ => {
                    // Other body types not supported in WASM
                }
            }
        }

        // Set WASM-specific options
        if let Some(credentials) = &self.credentials {
            init.credentials(credentials.clone());
        }

        if let Some(cache) = &self.cache {
            init.cache(cache.clone());
        }

        if let Some(mode) = &self.mode {
            init.mode(mode.clone());
        }

        if let Some(redirect) = &self.redirect {
            init.redirect(redirect.clone());
        }

        if let Some(referrer) = &self.referrer {
            init.referrer(referrer);
        }

        if let Some(referrer_policy) = &self.referrer_policy {
            init.referrer_policy(referrer_policy.clone());
        }

        if let Some(integrity) = &self.integrity {
            init.integrity(integrity);
        }

        init.keep_alive(self.keep_alive);

        if let Some(signal) = &self.signal {
            init.signal(Some(signal));
        }

        WebRequest::new_with_str_and_init(self.inner.url().as_str(), &init)
    }

    /// Convert from HttpRequest
    #[inline]
    pub fn from_http_request(request: HttpRequest) -> Self {
        Self::new(request)
    }

    /// Convert to HttpRequest
    #[inline]
    pub fn into_http_request(self) -> HttpRequest {
        self.inner
    }

    /// Get reference to inner HttpRequest
    #[inline]
    pub fn as_http_request(&self) -> &HttpRequest {
        &self.inner
    }

    /// Get mutable reference to inner HttpRequest
    #[inline]
    pub fn as_http_request_mut(&mut self) -> &mut HttpRequest {
        &mut self.inner
    }
}

/// WASM request builder for ergonomic construction
#[derive(Debug, Default)]
pub struct WasmRequestBuilder {
    method: Option<Method>,
    url: Option<Url>,
    headers: HeaderMap,
    body: Option<RequestBody>,
    timeout: Option<Duration>,
    
    #[cfg(target_arch = "wasm32")]
    credentials: Option<RequestCredentials>,
    #[cfg(target_arch = "wasm32")]
    cache: Option<RequestCache>,
    #[cfg(target_arch = "wasm32")]
    mode: Option<RequestMode>,
    
    #[cfg(not(target_arch = "wasm32"))]
    credentials: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    cache: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    mode: Option<String>,
}

impl WasmRequestBuilder {
    /// Create new builder
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set method
    #[inline]
    pub fn method(mut self, method: Method) -> Self {
        self.method = Some(method);
        self
    }

    /// Set URL
    #[inline]
    pub fn url(mut self, url: Url) -> Self {
        self.url = Some(url);
        self
    }

    /// Add header
    #[inline]
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
    {
        if let (Ok(name), Ok(val)) = (key.try_into(), value.try_into()) {
            self.headers.insert(name, val);
        }
        self
    }

    /// Set body
    #[inline]
    pub fn body(mut self, body: RequestBody) -> Self {
        self.body = Some(body);
        self
    }

    /// Set timeout
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    #[cfg(target_arch = "wasm32")]
    /// Set credentials
    #[inline]
    pub fn credentials(mut self, credentials: RequestCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Set credentials (fallback)
    #[inline]
    pub fn credentials<S: Into<String>>(mut self, credentials: S) -> Self {
        self.credentials = Some(credentials.into());
        self
    }

    /// Build the WasmRequest
    pub fn build(self) -> Result<WasmRequest, &'static str> {
        let method = self.method.ok_or("Method is required")?;
        let url = self.url.ok_or("URL is required")?;

        let mut http_request = HttpRequest::new(method, url, None, None, None).headers(self.headers);

        if let Some(body) = self.body {
            match body {
                RequestBody::Bytes(bytes) => {
                    http_request = http_request.body_bytes(bytes);
                }
                RequestBody::Text(text) => {
                    http_request = http_request.body_text(text);
                }
                _ => {
                    // Handle other body types as needed
                }
            }
        }

        if let Some(timeout) = self.timeout {
            http_request = http_request.timeout(timeout);
        }

        let mut wasm_request = WasmRequest::new(http_request);

        #[cfg(target_arch = "wasm32")]
        {
            wasm_request.credentials = self.credentials;
            wasm_request.cache = self.cache;
            wasm_request.mode = self.mode;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            wasm_request.credentials = self.credentials;
            wasm_request.cache = self.cache;
            wasm_request.mode = self.mode;
        }

        Ok(wasm_request)
    }
}

// Implement MessageChunk for WasmRequest to support fluent-ai streams-first architecture
impl MessageChunk for WasmRequest {
    fn bad_chunk(error: String) -> Self {
        // Create error request with safe URL construction
        let error_url = match Url::parse("https://localhost") {
            Ok(url) => url,
            Err(_) => match Url::parse("https://127.0.0.1") {
                Ok(url) => url,
                Err(_) => match Url::parse("about:blank") {
                    Ok(url) => url,
                    Err(_) => {
                        // If all URL parsing fails, create default and return early
                        let mut default_req = WasmRequest::default();
                        if let Some(ref mut inner_error) = default_req.inner.error {
                            *inner_error = error;
                        } else {
                            default_req.inner.error = Some(error);
                        }
                        return default_req;
                    }
                }
            }
        };
        let mut inner = HttpRequest::new(Method::GET, error_url, None, None, None);
        inner.error = Some(error);
        WasmRequest {
            inner,
            #[cfg(target_arch = "wasm32")]
            credentials: None,
            #[cfg(target_arch = "wasm32")]
            cache: None,
            #[cfg(target_arch = "wasm32")]
            mode: None,
            #[cfg(target_arch = "wasm32")]
            integrity: None,
            #[cfg(target_arch = "wasm32")]
            referrer: None,
            #[cfg(target_arch = "wasm32")]
            referrer_policy: None,
            #[cfg(not(target_arch = "wasm32"))]
            credentials: None,
            #[cfg(not(target_arch = "wasm32"))]
            cache: None,
            #[cfg(not(target_arch = "wasm32"))]
            mode: None,
            error_message: Some(error),
        }
    }

    fn is_error(&self) -> bool {
        self.error_message.is_some()
    }

    fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

/// Type alias for WASM Request - enables pure streaming architecture
pub type Request = WasmRequest;