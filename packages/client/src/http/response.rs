//! HTTP response types with component-level streaming
//!
//! This module provides the CANONICAL `HttpResponse` implementation where each
//! HTTP component (status, headers, body) is exposed as an individual `AsyncStream`,
//! enabling real-time processing as data arrives from the wire.

use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::RwLock;
use std::time::Instant;

use bytes::Bytes;
use ystream::AsyncStream;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Version};

/// HTTP response with component-level streaming
///
/// This is the CANONICAL `HttpResponse` implementation that exposes each HTTP
/// component as a separate `AsyncStream`, allowing processing of headers before
/// the body arrives, and enabling constant-memory processing of large responses.
pub struct HttpResponse {
    /// HTTP status code - set once, read many times (atomic, 0 = not yet received)
    status: AtomicU16,

    /// Headers stream - internal implementation detail
    headers_internal: AsyncStream<HttpHeader, 256>,

    /// Body stream - internal implementation detail
    body_internal: AsyncStream<HttpBodyChunk, 1024>,

    /// Trailers stream - internal implementation detail
    trailers_internal: AsyncStream<HttpHeader, 64>,

    /// HTTP version used for this response
    pub version: Version,

    /// Stream ID for HTTP/2 and HTTP/3 multiplexing
    pub stream_id: u64,

    /// Atomic cached etag value (set from first frame) 
    cached_etag: once_cell::sync::OnceCell<String>,
    
    /// Atomic cached last-modified value (set from first frame)
    cached_last_modified: once_cell::sync::OnceCell<String>,
    
    /// Cached headers collected from the stream
    cached_headers: RwLock<Option<Vec<HttpHeader>>>,
    
    /// Cached body bytes collected from the stream
    cached_body: RwLock<Option<Vec<u8>>>,
}

/// HTTP status information
#[derive(Debug, Clone)]
pub struct HttpStatus {
    /// HTTP status code
    pub code: StatusCode,

    /// Status reason phrase (may be empty in HTTP/2 and HTTP/3)
    pub reason: String,

    /// HTTP version
    pub version: Version,

    /// Timestamp when status was received
    pub timestamp: Instant,
}

/// Individual HTTP header
#[derive(Debug, Clone)]
pub struct HttpHeader {
    /// Header name
    pub name: HeaderName,

    /// Header value
    pub value: HeaderValue,

    /// Timestamp when header was received
    pub timestamp: Instant,
}

/// HTTP chunk types for streaming data
#[derive(Debug, Clone, Default)]
pub enum HttpChunk {
    /// Response body data chunk
    Body(Bytes),
    
    /// Raw data chunk
    Data(Bytes),
    
    /// Generic chunk data
    Chunk(Bytes),
    
    /// HTTP headers chunk
    Headers(StatusCode, HeaderMap),
    
    /// HTTP trailers chunk - headers that come after the body
    Trailers(HeaderMap),
    
    /// Error occurred during streaming
    Error(String),
    
    /// End of stream marker
    #[default]
    End,
}

/// Protocol-agnostic download chunk for file downloads with progress tracking
///
/// Used by `DownloadBuilder` to provide consistent download functionality
/// across all protocols (HTTP/2, HTTP/3, QUIC) via the strategy pattern.
#[derive(Debug, Clone)]
pub enum HttpDownloadChunk {
    /// Data chunk with download progress information
    Data { 
        /// Raw chunk data
        chunk: Vec<u8>, 
        /// Total bytes downloaded so far
        downloaded: u64, 
        /// Total file size if known
        total_size: Option<u64> 
    },
    
    /// Progress update without data (for progress-only notifications)
    Progress { 
        /// Total bytes downloaded so far
        downloaded: u64, 
        /// Total file size if known
        total_size: Option<u64> 
    },
    
    /// Download completed successfully
    Complete,
    
    /// Error occurred during download
    Error { 
        /// Error message
        message: String 
    },
}

impl Default for HttpDownloadChunk {
    fn default() -> Self {
        HttpDownloadChunk::Progress {
            downloaded: 0,
            total_size: None,
        }
    }
}

/// Type alias for download streams - protocol-agnostic download streaming
pub type HttpDownloadStream = AsyncStream<HttpDownloadChunk, 1024>;

/// HTTP body chunk with metadata
#[derive(Debug, Clone)]
pub struct HttpBodyChunk {
    /// Chunk data
    pub data: Bytes,

    /// Offset in the overall body stream
    pub offset: u64,

    /// Whether this is the final chunk
    pub is_final: bool,

    /// Timestamp when chunk was received
    pub timestamp: Instant,
}

impl HttpBodyChunk {
    /// Get the data bytes from this chunk
    pub fn data(&self) -> Option<&[u8]> {
        Some(&self.data)
    }
    
    /// Check if this chunk has an error
    pub fn error(&self) -> Option<&str> {
        None // HttpBodyChunk doesn't carry errors directly
    }
}

impl From<HttpBodyChunk> for HttpChunk {
    fn from(chunk: HttpBodyChunk) -> Self {
        HttpChunk::Body(chunk.data)
    }
}



impl HttpChunk {
    /// Get the data bytes from any chunk variant that contains data
    pub fn data(&self) -> Option<&Bytes> {
        match self {
            HttpChunk::Body(data) | HttpChunk::Data(data) | HttpChunk::Chunk(data) => Some(data),
            HttpChunk::Headers(_, _) | HttpChunk::Trailers(_) | HttpChunk::Error(_) | HttpChunk::End => None,
        }
    }
    
    /// Check if this is an error chunk
    pub fn is_error(&self) -> bool {
        matches!(self, HttpChunk::Error(_))
    }
    
    /// Check if this is the end marker
    pub fn is_end(&self) -> bool {
        matches!(self, HttpChunk::End)
    }
}

impl ystream::prelude::MessageChunk for HttpChunk {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        HttpChunk::Error(error_message)
    }
    
    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            HttpChunk::Error(msg) => Some(msg.as_str()),
            _ => None,
        }
    }
}

impl ystream::prelude::MessageChunk for HttpDownloadChunk {
    #[inline]
    fn bad_chunk(error_message: String) -> Self {
        HttpDownloadChunk::Error { message: error_message }
    }
    
    #[inline]
    fn error(&self) -> Option<&str> {
        match self {
            HttpDownloadChunk::Error { message } => Some(message.as_str()),
            _ => None,
        }
    }
}

impl HttpDownloadChunk {
    /// Get the data bytes from this download chunk
    #[must_use] 
    pub fn data(&self) -> Option<&[u8]> {
        match self {
            HttpDownloadChunk::Data { chunk, .. } => Some(chunk.as_slice()),
            _ => None,
        }
    }
    
    /// Get download progress information
    #[must_use] 
    pub fn progress(&self) -> Option<(u64, Option<u64>)> {
        match self {
            HttpDownloadChunk::Data { downloaded, total_size, .. } 
            | HttpDownloadChunk::Progress { downloaded, total_size } => Some((*downloaded, *total_size)),
            _ => None,
        }
    }
    
    /// Check if this is the completion marker
    #[must_use] 
    pub fn is_complete(&self) -> bool {
        matches!(self, HttpDownloadChunk::Complete)
    }
}

impl HttpResponse {
    /// Create a new `HttpResponse` with the given component streams
    #[must_use] 
    pub fn new(
        headers_stream: AsyncStream<HttpHeader, 256>,
        body_stream: AsyncStream<HttpBodyChunk, 1024>,
        trailers_stream: AsyncStream<HttpHeader, 64>,
        version: Version,
        stream_id: u64,
    ) -> Self {
        Self {
            status: AtomicU16::new(0),
            headers_internal: headers_stream,
            body_internal: body_stream,
            trailers_internal: trailers_stream,
            version,
            stream_id,
            cached_etag: once_cell::sync::OnceCell::new(),
            cached_last_modified: once_cell::sync::OnceCell::new(),
            cached_headers: RwLock::new(None),
            cached_body: RwLock::new(None),
        }
    }

    /// Get status code (0 if not yet received) - lock-free, zero-cost
    #[inline]
    pub fn status(&self) -> u16 {
        self.status.load(Ordering::Acquire)
    }
    
    /// Get `StatusCode` if available
    #[inline]
    pub fn status_code(&self) -> Option<StatusCode> {
        match self.status() {
            0 => None,
            code => StatusCode::from_u16(code).ok()
        }
    }
    
    /// Set status (called by protocol layers only)
    #[inline]
    pub(crate) fn set_status(&self, status: StatusCode) {
        self.status.store(status.as_u16(), Ordering::Release);
    }
    
    /// Get HTTP version
    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }
    
    /// Check methods - zero-cost, lock-free
    #[inline]
    pub fn is_success(&self) -> bool {
        self.status_code().is_some_and(|s| s.is_success())
    }
    
    #[inline]
    pub fn is_error(&self) -> bool {
        self.status_code().is_some_and(|s| s.is_client_error() || s.is_server_error())
    }
    
    #[inline]
    pub fn is_redirect(&self) -> bool {
        self.status_code().is_some_and(|s| s.is_redirection())
    }
    
    /// Get a specific header value by name
    pub fn header(&self, name: &str) -> Option<HeaderValue> {
        // Check cached headers if available
        if let Ok(cache) = self.cached_headers.read()
            && let Some(ref headers) = *cache {
                // Search through cached headers for matching name (case-insensitive)
                for http_header in headers {
                    if http_header.name.as_str().eq_ignore_ascii_case(name) {
                        return Some(http_header.value.clone());
                    }
                }
            }
        
        // No cached headers available or header not found
        // For streaming responses, user should call collect_and_cache_headers() first
        tracing::debug!(
            target: "quyc::http::response",
            header_name = name,
            "Header lookup on uncached response - call collect_and_cache_headers() first for streaming responses"
        );
        None
    }
    
    /// Get all headers as a vector (sync version - returns cached headers if available)
    /// 
    /// NOTE: This is a synchronous convenience method that returns cached headers.
    /// For streaming responses, use `collect_headers()` async method instead.
    pub fn headers(&self) -> Vec<HttpHeader> {
        // Return cached headers if available
        if let Ok(cache) = self.cached_headers.read()
            && let Some(ref headers) = *cache {
                return headers.clone();
            }
        
        // No cached headers available - return empty vector
        // User should call collect_and_cache_headers() first for streaming responses
        Vec::new()
    }
    
    /// Get response body as raw bytes (sync version - returns cached body if available)
    /// 
    /// NOTE: This is a synchronous convenience method that returns cached body.
    /// For streaming responses, use `collect_body()` async method instead.
    pub fn body(&self) -> Vec<u8> {
        // Return cached body if available
        if let Ok(cache) = self.cached_body.read()
            && let Some(ref body) = *cache {
                return body.clone();
            }
        
        // No cached body available - return empty vector
        // User should call collect_and_cache_body() first for streaming responses
        Vec::new()
    }
    
    /// Get response body as UTF-8 text
    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body()).to_string()
    }
    
    /// Deserialize response body as JSON
    pub fn body_json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body())
    }
    
    /// Get content type header value
    pub fn content_type(&self) -> Option<String> {
        self.header("content-type")
            .and_then(|v| v.to_str().ok().map(std::string::ToString::to_string))
    }
    
    /// Get content length if available
    pub fn content_length(&self) -> Option<usize> {
        self.header("content-length")
            .and_then(|v| v.to_str().ok().map(std::string::ToString::to_string))
            .and_then(|s| s.parse().ok())
    }
    
    /// Consume the response and return its body stream
    /// 
    /// This consumes the `HttpResponse` and returns ownership of the body stream
    /// for streaming transformations.
    pub fn into_body_stream(self) -> AsyncStream<HttpBodyChunk, 1024> {
        self.body_internal
    }
    
    /// Consume the response and return all its streams
    /// 
    /// This consumes the `HttpResponse` and returns ownership of all internal streams.
    pub fn into_streams(self) -> (
        AsyncStream<HttpHeader, 256>,
        AsyncStream<HttpBodyChunk, 1024>,
        AsyncStream<HttpHeader, 64>
    ) {
        (self.headers_internal, self.body_internal, self.trailers_internal)
    }

    /// Create an empty response (used for errors)
    #[must_use] 
    pub fn empty() -> Self {
        // Create proper AsyncStreams using channel factory method
        let (_, headers_stream) = AsyncStream::channel();
        let (_, body_stream) = AsyncStream::channel();
        let (_, trailers_stream) = AsyncStream::channel();

        Self {
            status: AtomicU16::new(0),
            headers_internal: headers_stream,
            body_internal: body_stream,
            trailers_internal: trailers_stream,
            version: Version::HTTP_11,
            stream_id: 0,
            cached_etag: once_cell::sync::OnceCell::new(),
            cached_last_modified: once_cell::sync::OnceCell::new(),
            cached_headers: RwLock::new(None),
            cached_body: RwLock::new(None),
        }
    }

    /// Create `HttpResponse` from cached entry
    /// 
    /// Converts a `CacheEntry` back into an `HttpResponse` for serving cached responses.
    pub fn from_cache_entry(cache_entry: crate::cache::cache_entry::CacheEntry) -> Self {
        use crate::http::response::{HttpBodyChunk, HttpHeader};
        
        // Create channels for streaming the cached data
        let (headers_sender, headers_stream) = AsyncStream::channel();
        let (body_sender, body_stream) = AsyncStream::channel();
        let (_, trailers_stream) = AsyncStream::channel(); // No trailers in cache

        // Emit cached headers
        for (name, value) in &cache_entry.headers {
            let http_header = HttpHeader {
                name: name.clone(),
                value: value.clone(),
                timestamp: Instant::now(),
            };
            // Emit header to stream
            drop(headers_sender.send(http_header));
        }

        // Emit cached body as single chunk
        if !cache_entry.body.is_empty() {
            let body_chunk = HttpBodyChunk::new(cache_entry.body.clone(), 0, true);
            drop(body_sender.send(body_chunk));
        }

        Self {
            status: AtomicU16::new(cache_entry.status.as_u16()),
            headers_internal: headers_stream,
            body_internal: body_stream,
            trailers_internal: trailers_stream,
            version: cache_entry.version,
            stream_id: 0, // Cache entries don't have stream IDs
            cached_etag: if let Some(etag) = cache_entry.etag {
                let cell = once_cell::sync::OnceCell::new();
                cell.set(etag).ok();
                cell
            } else {
                once_cell::sync::OnceCell::new()
            },
            cached_last_modified: once_cell::sync::OnceCell::new(), // Would need conversion from SystemTime
            cached_headers: RwLock::new(None),
            cached_body: RwLock::new(Some(cache_entry.body.to_vec())),
        }
    }

    /// Create `HttpResponse` from HTTP/2 response
    #[must_use] 
    pub fn from_http2_response(
        status: StatusCode,
        headers: HeaderMap,
        body_stream: AsyncStream<HttpBodyChunk, 1024>,
        trailers_stream: AsyncStream<HttpHeader, 64>,
        stream_id: u64,
    ) -> Self {
        // Create channels for headers only
        let (headers_sender, headers_stream) = AsyncStream::channel();

        // Emit headers immediately
        for (name, value) in &headers {
            let http_header = HttpHeader {
                name: name.clone(),
                value: value.clone(),
                timestamp: Instant::now(),
            };
            // Intentionally ignore send result - channel may be closed if receiver dropped
            drop(headers_sender.send(http_header));
        }

        Self {
            status: AtomicU16::new(status.as_u16()),
            headers_internal: headers_stream,
            body_internal: body_stream,
            trailers_internal: trailers_stream,
            version: Version::HTTP_2,
            stream_id,
            cached_etag: once_cell::sync::OnceCell::new(),
            cached_last_modified: once_cell::sync::OnceCell::new(),
            cached_headers: RwLock::new(None),
            cached_body: RwLock::new(None),
        }
    }

    /// Create `HttpResponse` from HTTP/3 response
    #[must_use] 
    pub fn from_http3_response(
        status: StatusCode,
        headers: HeaderMap,
        body_stream: AsyncStream<HttpBodyChunk, 1024>,
        trailers_stream: AsyncStream<HttpHeader, 64>,
        stream_id: u64,
    ) -> Self {
        // Create channels for headers only
        let (headers_sender, headers_stream) = AsyncStream::channel();

        // Emit headers immediately
        for (name, value) in &headers {
            let http_header = HttpHeader {
                name: name.clone(),
                value: value.clone(),
                timestamp: Instant::now(),
            };
            // Intentionally ignore send result - channel may be closed if receiver dropped
            drop(headers_sender.send(http_header));
        }

        Self {
            status: AtomicU16::new(status.as_u16()),
            headers_internal: headers_stream,
            body_internal: body_stream,
            trailers_internal: trailers_stream,
            version: Version::HTTP_3,
            stream_id,
            cached_etag: once_cell::sync::OnceCell::new(),
            cached_last_modified: once_cell::sync::OnceCell::new(),
            cached_headers: RwLock::new(None),
            cached_body: RwLock::new(None),
        }
    }

    /// Create error `HttpResponse`
    #[must_use] 
    pub fn error(status_code: StatusCode, message: String) -> Self {
        use bytes::Bytes;

        // Create channels
        let (headers_sender, headers_stream) = AsyncStream::channel();
        let (body_sender, body_stream) = AsyncStream::channel();
        let (_, trailers_stream) = AsyncStream::channel();

        // Emit content-type header
        let content_type_header = HttpHeader {
            name: http::header::CONTENT_TYPE,
            value: HeaderValue::from_static("text/plain"),
            timestamp: Instant::now(),
        };
        // Intentionally ignore send result - error response setup
        drop(headers_sender.send(content_type_header));

        // Emit error message as body
        let error_chunk = HttpBodyChunk {
            data: Bytes::from(message),
            offset: 0,
            is_final: true,
            timestamp: Instant::now(),
        };
        // Intentionally ignore send result - error response setup
        drop(body_sender.send(error_chunk));

        Self {
            status: AtomicU16::new(status_code.as_u16()),
            headers_internal: headers_stream,
            body_internal: body_stream,
            trailers_internal: trailers_stream,
            version: Version::HTTP_11,
            stream_id: 0,
            cached_etag: once_cell::sync::OnceCell::new(),
            cached_last_modified: once_cell::sync::OnceCell::new(),
            cached_headers: RwLock::new(None),
            cached_body: RwLock::new(None),
        }
    }

    /// Collect all headers into a `HeaderMap`
    /// Note: This consumes the headers stream
    pub async fn collect_headers(&mut self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        while let Some(header) = self.headers_internal.next().await {
            headers.insert(header.name, header.value);
        }
        headers
    }

    /// Collect the entire body into a single Bytes buffer
    /// Note: This consumes the body stream and may use significant memory
    pub async fn collect_body(&mut self) -> Bytes {
        let mut chunks = Vec::new();
        let mut total_size = 0;

        while let Some(chunk) = self.body_internal.next().await {
            total_size += chunk.data.len();
            chunks.push(chunk.data);
        }

        // Combine all chunks into a single Bytes
        if chunks.is_empty() {
            Bytes::new()
        } else if chunks.len() == 1 {
            chunks.into_iter().next().unwrap_or_else(Bytes::new)
        } else {
            let mut combined = Vec::with_capacity(total_size);
            for chunk in chunks {
                combined.extend_from_slice(&chunk);
            }
            Bytes::from(combined)
        }
    }
    
    /// Collect and cache headers for synchronous access
    /// This consumes the headers stream and caches the results
    pub async fn collect_and_cache_headers(&mut self) -> Vec<HttpHeader> {
        let mut headers_vec = Vec::new();
        
        while let Some(header) = self.headers_internal.next().await {
            headers_vec.push(header);
        }
        
        // Cache the collected headers
        if let Ok(mut cache) = self.cached_headers.write() {
            *cache = Some(headers_vec.clone());
        }
        
        headers_vec
    }
    
    /// Collect and cache body for synchronous access
    /// This consumes the body stream and caches the results
    pub async fn collect_and_cache_body(&mut self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let mut total_size = 0;

        while let Some(chunk) = self.body_internal.next().await {
            total_size += chunk.data.len();
            chunks.push(chunk.data);
        }

        // Combine all chunks into a single buffer
        let body = if chunks.is_empty() {
            Vec::new()
        } else if chunks.len() == 1 {
            chunks.into_iter().next().unwrap_or_else(Bytes::new).to_vec()
        } else {
            let mut combined = Vec::with_capacity(total_size);
            for chunk in chunks {
                combined.extend_from_slice(&chunk);
            }
            combined
        };
        
        // Cache the collected body
        if let Ok(mut cache) = self.cached_body.write() {
            *cache = Some(body.clone());
        }
        
        body
    }

    /// Extract `ETag` header value - returns cached value from first frame
    #[inline]
    pub fn etag(&self) -> Option<&str> {
        self.cached_etag.get().map(std::string::String::as_str)
    }

    /// Extract Last-Modified header value - returns cached value from first frame  
    #[inline]
    pub fn last_modified(&self) -> Option<&str> {
        self.cached_last_modified.get().map(std::string::String::as_str)
    }

    /// Set cached etag value (called by protocol layer on first frame)
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn set_cached_etag(&self, etag: String) {
        // Intentionally ignore set result - may already be set
        drop(self.cached_etag.set(etag));
    }

    /// Set cached last-modified value (called by protocol layer on first frame)
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn set_cached_last_modified(&self, last_modified: String) {
        // Intentionally ignore set result - may already be set
        drop(self.cached_last_modified.set(last_modified));
    }




}

impl HttpStatus {
    /// Create a new `HttpStatus`
    #[must_use] 
    pub fn new(code: StatusCode, reason: String, version: Version) -> Self {
        Self {
            code,
            reason,
            version,
            timestamp: Instant::now(),
        }
    }

    /// Create from just a status code (for HTTP/2 and HTTP/3)
    #[must_use] 
    pub fn from_code(code: StatusCode, version: Version) -> Self {
        Self {
            code,
            reason: String::new(),
            version,
            timestamp: Instant::now(),
        }
    }
}

impl HttpHeader {
    /// Create a new `HttpHeader`
    pub fn new(name: HeaderName, value: HeaderValue) -> Self {
        Self {
            name,
            value,
            timestamp: Instant::now(),
        }
    }
}

impl HttpBodyChunk {
    /// Create a new `HttpBodyChunk`
    pub fn new(data: Bytes, offset: u64, is_final: bool) -> Self {
        Self {
            data,
            offset,
            is_final,
            timestamp: Instant::now(),
        }
    }


}

impl ystream::prelude::MessageChunk for HttpResponse {
    fn bad_chunk(_error: String) -> Self {
        use ystream::AsyncStream;

        let (_, headers_stream) = AsyncStream::channel();
        let (_, body_stream) = AsyncStream::channel();
        let (_, trailers_stream) = AsyncStream::channel();

        Self {
            status: AtomicU16::new(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
            headers_internal: headers_stream,
            body_internal: body_stream,
            trailers_internal: trailers_stream,
            version: Version::HTTP_11,
            stream_id: 0,
            cached_etag: once_cell::sync::OnceCell::new(),
            cached_last_modified: once_cell::sync::OnceCell::new(),
            cached_headers: RwLock::new(None),
            cached_body: RwLock::new(None),
        }
    }

    fn is_error(&self) -> bool {
        self.status_code().is_some_and(|s| s.is_client_error() || s.is_server_error())
    }

    fn error(&self) -> Option<&str> {
        if self.is_error() {
            let status_code = StatusCode::from_u16(self.status.load(std::sync::atomic::Ordering::Relaxed))
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            Some(status_code.canonical_reason().unwrap_or("Unknown Error"))
        } else {
            None
        }
    }
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self::empty()
    }
}

impl ystream::prelude::MessageChunk for HttpStatus {
    fn bad_chunk(error: String) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: error,
            version: Version::HTTP_11,
            timestamp: Instant::now(),
        }
    }

    fn is_error(&self) -> bool {
        self.code.is_server_error() || self.code.is_client_error()
    }

    fn error(&self) -> Option<&str> {
        if self.is_error() {
            Some(&self.reason)
        } else {
            None
        }
    }
}

impl Default for HttpStatus {
    fn default() -> Self {
        Self {
            code: StatusCode::OK,
            reason: "OK".to_string(),
            version: Version::HTTP_11,
            timestamp: Instant::now(),
        }
    }
}

impl ystream::prelude::MessageChunk for HttpHeader {
    fn bad_chunk(error: String) -> Self {
        Self {
            name: HeaderName::from_static("x-error"),
            value: HeaderValue::from_str(&error)
                .unwrap_or_else(|_| HeaderValue::from_static("invalid")),
            timestamp: Instant::now(),
        }
    }

    fn is_error(&self) -> bool {
        self.name == "x-error"
    }

    fn error(&self) -> Option<&str> {
        if self.is_error() {
            self.value.to_str().ok()
        } else {
            None
        }
    }
}

impl Default for HttpHeader {
    fn default() -> Self {
        Self {
            name: HeaderName::from_static("content-length"),
            value: HeaderValue::from_static("0"),
            timestamp: Instant::now(),
        }
    }
}

impl ystream::prelude::MessageChunk for HttpBodyChunk {
    fn bad_chunk(error: String) -> Self {
        Self {
            data: Bytes::from(error),
            offset: 0,
            is_final: true,
            timestamp: Instant::now(),
        }
    }

    fn is_error(&self) -> bool {
        false // Body chunks don't represent errors by themselves
    }

    fn error(&self) -> Option<&str> {
        None
    }
}

impl Default for HttpBodyChunk {
    fn default() -> Self {
        Self {
            data: Bytes::new(),
            offset: 0,
            is_final: false,
            timestamp: Instant::now(),
        }
    }
}
