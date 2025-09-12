//! HTTP request types and builders
//!
//! This module provides the CANONICAL `HttpRequest` implementation that consolidates
//! all previous Request variants into a single, comprehensive request type.

use std::collections::HashMap;
use std::time::Duration;
use std::sync::LazyLock;

use bytes::Bytes;
use ystream::prelude::*;
use http::{HeaderMap, HeaderName, HeaderValue, Method, Version};
use url::Url;

use crate::prelude::*;
use crate::protocols::core::HttpMethod;

/// Safe fallback URL that is guaranteed to parse correctly
/// Used when URL parsing fails to prevent application crashes
static SAFE_FALLBACK_URL: LazyLock<Url> = LazyLock::new(|| {
    // This URL is guaranteed to parse and will never panic
    Url::parse("http://localhost:80/").unwrap_or_else(|_| {
        // If even this fails, try alternative safe URLs
        // This should never happen unless the system is completely broken
        Url::parse("http://127.0.0.1/").unwrap_or_else(|_| {
            // If all parsing fails, this indicates a completely broken URL library
            // Since we cannot use unsafe code, we panic with a clear message
            panic!("URL parsing library completely broken - cannot parse even basic URLs")
        })
    })
});

/// Create a guaranteed-safe URL with proper error recovery
fn create_safe_url(url_str: &str) -> Url {
    match Url::parse(url_str) {
        Ok(url) => url,
        Err(e) => {
            tracing::warn!("URL parsing failed: {} - using fallback", e);
            SAFE_FALLBACK_URL.clone()
        }
    }
}


/// HTTP request structure with comprehensive functionality
///
/// This is the CANONICAL `HttpRequest` implementation that consolidates all
/// previous Request variants into a single, feature-complete request type.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct HttpRequest {
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: Option<RequestBody>,
    timeout: Option<Duration>,
    retry_attempts: Option<u32>,
    version: Version,

    /// Stream ID for HTTP/2 and HTTP/3 multiplexing
    pub stream_id: Option<u64>,

    /// Request configuration
    pub cors: bool,
    pub follow_redirects: bool,
    pub max_redirects: u32,
    pub compress: bool,

    /// Authentication
    pub auth: Option<RequestAuth>,

    /// Caching
    pub cache_control: Option<String>,
    pub etag: Option<String>,

    /// Request metadata
    pub user_agent: Option<String>,
    pub referer: Option<String>,

    /// Protocol-specific options
    pub h2_prior_knowledge: bool,
    pub h3_alt_svc: bool,

    /// Internal error state for deferred error handling
    error: Option<String>,
}

/// Request body types
pub enum RequestBody {
    /// Raw bytes
    Bytes(Bytes),
    /// Text content
    Text(String),
    /// JSON data
    Json(serde_json::Value),
    /// Form data
    Form(HashMap<String, String>),
    /// Multipart form data
    Multipart(Vec<MultipartField>),
    /// Streaming body
    Stream(AsyncStream<HttpChunk, 1024>),
}

impl std::fmt::Debug for RequestBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestBody::Bytes(bytes) => f
                .debug_tuple("Bytes")
                .field(&format!("{} bytes", bytes.len()))
                .finish(),
            RequestBody::Text(text) => f
                .debug_tuple("Text")
                .field(&format!("{} chars", text.len()))
                .finish(),
            RequestBody::Json(value) => f.debug_tuple("Json").field(value).finish(),
            RequestBody::Form(form) => f.debug_tuple("Form").field(form).finish(),
            RequestBody::Multipart(fields) => f
                .debug_tuple("Multipart")
                .field(&format!("{} fields", fields.len()))
                .finish(),
            RequestBody::Stream(_) => f.debug_tuple("Stream").field(&"<AsyncStream>").finish(),
        }
    }
}

impl Clone for RequestBody {
    fn clone(&self) -> Self {
        match self {
            RequestBody::Bytes(bytes) => RequestBody::Bytes(bytes.clone()),
            RequestBody::Text(text) => RequestBody::Text(text.clone()),
            RequestBody::Json(value) => RequestBody::Json(value.clone()),
            RequestBody::Form(form) => RequestBody::Form(form.clone()),
            RequestBody::Multipart(fields) => RequestBody::Multipart(fields.clone()),
            RequestBody::Stream(_) => RequestBody::Bytes(bytes::Bytes::new()), // Convert to empty bytes instead of panic
        }
    }
}

impl RequestBody {
    /// Get the length of the body in bytes
    pub fn len(&self) -> usize {
        match self {
            RequestBody::Bytes(bytes) => bytes.len(),
            RequestBody::Text(text) => text.len(),
            RequestBody::Json(json) => serde_json::to_vec(json).map(|v| v.len()).unwrap_or(0),
            RequestBody::Form(form) => serde_urlencoded::to_string(form)
                .map(|s| s.len())
                .unwrap_or(0),
            RequestBody::Multipart(fields) => Self::calculate_multipart_size(fields),
            RequestBody::Stream(_) => 0,    // Streaming body size is unknown
        }
    }

    /// Calculate multipart form body size without building the actual body
    fn calculate_multipart_size(fields: &[MultipartField]) -> usize {
        // Fixed boundary format: "----formdata-fluent-" + 16 hex chars = 36 chars
        let boundary_len = 36;
        let mut total_size = 0;
        
        for field in fields {
            // Boundary separator: "--{boundary}\r\n"
            total_size += 2 + boundary_len + 2; // "--" + boundary + "\r\n"
            
            // Content-Disposition header variations
            match (&field.filename, &field.content_type) {
                (Some(filename), Some(content_type)) => {
                    // Content-Disposition: form-data; name="{name}"; filename="{filename}"\r\n
                    total_size += 54 + field.name.len() + filename.len(); // Fixed: was 64, now 54
                    // Content-Type: {content_type}\r\n\r\n
                    total_size += 16 + content_type.len(); // "Content-Type: " + type + "\r\n\r\n"
                }
                (Some(filename), None) => {
                    // Content-Disposition: form-data; name="{name}"; filename="{filename}"\r\n
                    total_size += 54 + field.name.len() + filename.len(); // Fixed: was 64, now 54
                    // Content-Type: application/octet-stream\r\n\r\n
                    total_size += 42; // Fixed: was 40, now 42 ("Content-Type: application/octet-stream\r\n\r\n")
                }
                (None, Some(content_type)) => {
                    // Content-Disposition: form-data; name="{name}"\r\n
                    total_size += 39 + field.name.len(); // Header + field name
                    // Content-Type: {content_type}\r\n\r\n
                    total_size += 16 + content_type.len();
                }
                (None, None) => {
                    // Content-Disposition: form-data; name="{name}"\r\n\r\n
                    total_size += 41 + field.name.len(); // Header + field name + extra \r\n
                }
            }
            
            // Field value
            match &field.value {
                MultipartValue::Text(text) => total_size += text.len(),
                MultipartValue::Bytes(bytes) => total_size += bytes.len(),
            }
            
            // Trailing \r\n after field value
            total_size += 2;
        }
        
        // Final boundary: "--{boundary}--\r\n"
        total_size += 2 + boundary_len + 2 + 2; // "--" + boundary + "--" + "\r\n"
        
        total_size
    }

    /// Check if the body is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Multipart form field
#[derive(Debug, Clone)]
pub struct MultipartField {
    pub name: String,
    pub value: MultipartValue,
    pub content_type: Option<String>,
    pub filename: Option<String>,
}

/// Multipart field value
#[derive(Debug, Clone)]
pub enum MultipartValue {
    Text(String),
    Bytes(Bytes),
}

/// Authentication methods
#[derive(Debug, Clone)]
pub enum RequestAuth {
    Basic { username: String, password: String },
    Bearer(String),
    ApiKey { key: String, value: String },
    Custom(HeaderMap),
}

/// HTTP request builder that implements the `HttpRequestBuilder` trait
#[derive(Debug, Clone)]
pub struct HttpRequestBuilder {
    method: HttpMethod,
    uri: Option<String>,
    headers: HeaderMap,
    #[allow(dead_code)]
    body: Option<Bytes>,
}

impl HttpRequestBuilder {
    /// Create a new request builder
    #[must_use] 
    pub fn new() -> Self {
        Self {
            method: HttpMethod::Get,
            uri: None,
            headers: HeaderMap::new(),
            body: None,
        }
    }

    /// Set the HTTP method
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn method(mut self, method: &Method) -> Self {
        self.method = match *method {
            Method::POST => HttpMethod::Post,
            Method::PUT => HttpMethod::Put,
            Method::DELETE => HttpMethod::Delete,
            Method::PATCH => HttpMethod::Patch,
            Method::HEAD => HttpMethod::Head,
            Method::OPTIONS => HttpMethod::Options,
            Method::TRACE => HttpMethod::Trace,
            Method::CONNECT => HttpMethod::Connect,
            _ => HttpMethod::Get, // Default fallback (includes GET)
        };
        self
    }

    /// Set the URI
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn uri<U: Into<String>>(mut self, uri: U) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Set headers
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Build the final `HttpRequest`
    pub fn build(self) -> HttpRequest {
        let url = match self.uri {
            Some(uri_str) => create_safe_url(&uri_str),
            None => SAFE_FALLBACK_URL.clone(),
        };

        let method = match self.method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Patch => Method::PATCH,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Options => Method::OPTIONS,
            HttpMethod::Trace => Method::TRACE,
            HttpMethod::Connect => Method::CONNECT,
        };

        HttpRequest::new(method, url, Some(self.headers), None, None)
    }
}

impl Default for HttpRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}


impl HttpRequest {
    /// Create a new request builder
    #[inline]
    #[must_use] 
    pub fn builder() -> HttpRequestBuilder {
        HttpRequestBuilder::default()
    }

    /// Creates a new `HttpRequest`
    #[inline]
    #[must_use] 
    pub fn new(
        method: Method,
        url: Url,
        headers: Option<HeaderMap>,
        body: Option<RequestBody>,
        timeout: Option<Duration>,
    ) -> Self {
        Self {
            method,
            url,
            headers: match headers {
                Some(h) => h,
                None => HeaderMap::new(),
            },
            body,
            timeout: timeout.or_else(|| Some(Duration::from_secs(30))),
            retry_attempts: Some(3),
            version: Version::HTTP_3,
            stream_id: None,
            cors: true,
            follow_redirects: true,
            max_redirects: 10,
            compress: true,
            auth: None,
            cache_control: None,
            etag: None,
            user_agent: Some("fluent-ai-http3/1.0".to_string()),
            referer: None,
            h2_prior_knowledge: false,
            h3_alt_svc: true,
            error: None,
        }
    }

    /// Create GET request
    #[inline]
    pub fn get<U: TryInto<Url>>(url: U) -> Self {
        if let Ok(parsed_url) = url.try_into() { 
            Self::new(Method::GET, parsed_url, None, None, None) 
        } else {
            // Create a request with error state - use safe dummy URL
            let dummy_url = create_safe_url("http://invalid.localhost/");
            let mut request = Self::new(Method::GET, dummy_url, None, None, None);
            request.error = Some("Invalid URL provided".to_string());
            request
        }
    }

    /// Create POST request
    #[inline]
    pub fn post<U: TryInto<Url>>(url: U) -> Self {
        if let Ok(parsed_url) = url.try_into() { 
            Self::new(Method::POST, parsed_url, None, None, None) 
        } else {
            let dummy_url = create_safe_url("http://invalid.localhost/");
            let mut request = Self::new(Method::POST, dummy_url, None, None, None);
            request.error = Some("Invalid URL provided".to_string());
            request
        }
    }

    /// Create PUT request
    #[inline]
    pub fn put<U: TryInto<Url>>(url: U) -> Self {
        if let Ok(parsed_url) = url.try_into() { 
            Self::new(Method::PUT, parsed_url, None, None, None) 
        } else {
            let dummy_url = create_safe_url("http://invalid.localhost/");
            let mut request = Self::new(Method::PUT, dummy_url, None, None, None);
            request.error = Some("Invalid URL provided".to_string());
            request
        }
    }

    /// Create DELETE request
    #[inline]
    pub fn delete<U: TryInto<Url>>(url: U) -> Self {
        if let Ok(parsed_url) = url.try_into() { 
            Self::new(Method::DELETE, parsed_url, None, None, None) 
        } else {
            let dummy_url = create_safe_url("http://invalid.localhost/");
            let mut request = Self::new(Method::DELETE, dummy_url, None, None, None);
            request.error = Some("Invalid URL provided".to_string());
            request
        }
    }

    /// Create PATCH request
    #[inline]
    pub fn patch<U: TryInto<Url>>(url: U) -> Self {
        if let Ok(parsed_url) = url.try_into() { 
            Self::new(Method::PATCH, parsed_url, None, None, None) 
        } else {
            let dummy_url = create_safe_url("http://invalid.localhost/");
            let mut request = Self::new(Method::PATCH, dummy_url, None, None, None);
            request.error = Some("Invalid URL provided".to_string());
            request
        }
    }

    /// Create HEAD request
    #[inline]
    pub fn head<U: TryInto<Url>>(url: U) -> Self {
        if let Ok(parsed_url) = url.try_into() { 
            Self::new(Method::HEAD, parsed_url, None, None, None) 
        } else {
            let dummy_url = create_safe_url("http://invalid.localhost/");
            let mut request = Self::new(Method::HEAD, dummy_url, None, None, None);
            request.error = Some("Invalid URL provided".to_string());
            request
        }
    }

    /// Create OPTIONS request
    #[inline]
    pub fn options<U: TryInto<Url>>(url: U) -> Self {
        if let Ok(parsed_url) = url.try_into() { 
            Self::new(Method::OPTIONS, parsed_url, None, None, None) 
        } else {
            let dummy_url = create_safe_url("http://invalid.localhost/");
            let mut request = Self::new(Method::OPTIONS, dummy_url, None, None, None);
            request.error = Some("Invalid URL provided".to_string());
            request
        }
    }

    // Getters

    /// Get the HTTP method
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get the URL
    #[inline]
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the URI as a string (alias for URL compatibility)
    #[inline]
    pub fn uri(&self) -> String {
        self.url.to_string()
    }

    /// Get the headers
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get mutable reference to headers
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Get the body
    #[inline]
    pub fn body(&self) -> Option<&RequestBody> {
        self.body.as_ref()
    }

    /// Get the timeout
    #[inline]
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Set the stream ID for HTTP/2 and HTTP/3 multiplexing
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn with_stream_id(mut self, stream_id: u64) -> Self {
        self.stream_id = Some(stream_id);
        self
    }

    /// Get retry attempts
    #[inline]
    pub fn retry_attempts(&self) -> Option<u32> {
        self.retry_attempts
    }

    /// Get HTTP version
    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    // Setters (builder pattern)

    /// Set the HTTP method
    #[inline]
    #[must_use]
    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Set the URL
    #[inline]
    #[must_use]
    pub fn with_url(mut self, url: Url) -> Self {
        self.url = url;
        self
    }

    /// Set HTTP version
    #[inline]
    #[must_use]
    pub fn with_version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    /// Add a header
    #[inline]
    #[must_use]
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

    /// Extend headers
    #[inline]
    #[must_use]
    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Set body as bytes
    #[inline]
    #[must_use]
    pub fn body_bytes<B: Into<Bytes>>(mut self, body: B) -> Self {
        self.body = Some(RequestBody::Bytes(body.into()));
        self
    }

    /// Set body as text
    #[inline]
    #[must_use]
    pub fn body_text<S: Into<String>>(mut self, body: S) -> Self {
        self.body = Some(RequestBody::Text(body.into()));
        self
    }

    /// Set body as JSON
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn json<T: serde::Serialize>(mut self, json: &T) -> Self {
        match serde_json::to_value(json) {
            Ok(value) => {
                self.body = Some(RequestBody::Json(value));
                self.headers.insert(
                    http::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                self
            }
            Err(e) => {
                self.error = Some(format!("JSON serialization failed: {e}"));
                self
            }
        }
    }

    /// Set body as form data
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn form(mut self, form: HashMap<String, String>) -> Self {
        self.body = Some(RequestBody::Form(form));
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        self
    }

    /// Set body as multipart form
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn multipart(mut self, fields: Vec<MultipartField>) -> Self {
        self.body = Some(RequestBody::Multipart(fields));
        // Content-Type with boundary will be set during serialization
        self
    }

    /// Set streaming body
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn body_stream(mut self, stream: AsyncStream<HttpChunk, 1024>) -> Self {
        self.body = Some(RequestBody::Stream(stream));
        self
    }

    /// Set timeout
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set retry attempts
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn with_retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = Some(attempts);
        self
    }

    /// Enable/disable CORS
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn cors(mut self, enable: bool) -> Self {
        self.cors = enable;
        self
    }

    /// Enable/disable redirect following
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// Set maximum redirects
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn max_redirects(mut self, max: u32) -> Self {
        self.max_redirects = max;
        self
    }

    /// Enable/disable compression
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn compress(mut self, enable: bool) -> Self {
        self.compress = enable;
        self
    }

    /// Set basic authentication
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn basic_auth<U, P>(mut self, username: U, password: P) -> Self
    where
        U: Into<String>,
        P: Into<String>,
    {
        self.auth = Some(RequestAuth::Basic {
            username: username.into(),
            password: password.into(),
        });
        self
    }

    /// Set bearer token authentication
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn bearer_auth<T: Into<String>>(mut self, token: T) -> Self {
        self.auth = Some(RequestAuth::Bearer(token.into()));
        self
    }

    /// Set API key authentication
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn api_key<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.auth = Some(RequestAuth::ApiKey {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Set custom authentication headers
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn custom_auth(mut self, headers: HeaderMap) -> Self {
        self.auth = Some(RequestAuth::Custom(headers));
        self
    }

    /// Set user agent
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set referer
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn referer<S: Into<String>>(mut self, referer: S) -> Self {
        self.referer = Some(referer.into());
        self
    }

    /// Add query parameters
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn query<K, V>(mut self, params: &[(K, V)]) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut query_pairs = self.url.query_pairs_mut();
        for (key, value) in params {
            query_pairs.append_pair(key.as_ref(), value.as_ref());
        }
        drop(query_pairs);
        self
    }

    /// Add query parameters with builder pattern - alias for query method
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn with_query_params<K, V>(self, params: &[(K, V)]) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.query(params)
    }

    /// Enable HTTP/2 prior knowledge
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn h2_prior_knowledge(mut self, enable: bool) -> Self {
        self.h2_prior_knowledge = enable;
        self
    }

    /// Enable HTTP/3 Alt-Svc
    #[inline]
    #[must_use = "Request builder methods return a new request and should be used"]
    pub fn h3_alt_svc(mut self, enable: bool) -> Self {
        self.h3_alt_svc = enable;
        self
    }

    // Utility methods

    /// Check if request has body
    #[inline]
    pub fn has_body(&self) -> bool {
        self.body.is_some()
    }

    /// Get content length if known
    pub fn content_length(&self) -> Option<u64> {
        match &self.body {
            Some(RequestBody::Bytes(bytes)) => Some(bytes.len() as u64),
            Some(RequestBody::Text(text)) => Some(text.len() as u64),
            Some(RequestBody::Json(json)) => serde_json::to_vec(json).ok().map(|v| v.len() as u64),
            Some(RequestBody::Form(form)) => {
                let encoded = serde_urlencoded::to_string(form).ok()?;
                Some(encoded.len() as u64)
            }
            _ => None,
        }
    }

    /// Check if there were any request building errors
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Get the error message if any
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl MultipartField {
    /// Create text field
    #[inline]
    pub fn text<N: Into<String>, V: Into<String>>(name: N, value: V) -> Self {
        Self {
            name: name.into(),
            value: MultipartValue::Text(value.into()),
            content_type: Some("text/plain".to_string()),
            filename: None,
        }
    }

    /// Create file field
    #[inline]
    pub fn file<N: Into<String>, F: Into<String>>(
        name: N,
        filename: F,
        content_type: Option<String>,
        data: Bytes,
    ) -> Self {
        Self {
            name: name.into(),
            value: MultipartValue::Bytes(data),
            content_type,
            filename: Some(filename.into()),
        }
    }
}
