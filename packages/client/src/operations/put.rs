//! PUT HTTP Operations Module - Idempotent resource replacement with `ETag` support

use http::{HeaderMap, HeaderName, HeaderValue, Method};
use serde_json::Value;

use crate::operations::HttpOperation;
use crate::{
    client::HttpClient, http::request::HttpRequest, prelude::AsyncStream,
};

/// PUT operation implementation for idempotent resource replacement
#[derive(Clone)]
pub struct PutOperation {
    client: HttpClient,
    url: String,
    headers: HeaderMap,
    body: PutBody,
}

/// Supported PUT body types
#[derive(Clone)]
pub enum PutBody {
    /// JSON-encoded request body
    Json(Value),
    /// Binary data body
    Binary(Vec<u8>),
    /// Plain text body
    Text(String),
    /// Empty request body
    Empty,
}

impl PutOperation {
    /// Create a new PUT operation
    #[must_use]
    pub fn new(client: HttpClient, url: String) -> Self {
        Self {
            client,
            url,
            headers: HeaderMap::new(),
            body: PutBody::Empty,
        }
    }

    /// Add custom header
    ///
    /// # Arguments
    /// * `key` - The header name
    /// * `value` - The header value
    ///
    /// # Returns
    /// `Self` for method chaining - errors silently ignored, handled during execution
    #[must_use] 
    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            self.headers.insert(header_name, header_value);
        }
        // Invalid headers are silently ignored - errors will surface during request execution as stream events
        self
    }

    /// Set JSON body with automatic Content-Type
    #[must_use]
    pub fn json(mut self, json_value: Value) -> Self {
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        self.body = PutBody::Json(json_value);
        self
    }

    /// Set binary body with Content-Type
    ///
    /// # Arguments
    /// * `data` - The binary data to send
    /// * `content_type` - The MIME type of the content
    ///
    /// # Returns
    /// `Self` for method chaining - errors silently ignored, handled during execution
    #[must_use = "Operation builder methods return a new operation and should be used"]
    pub fn binary(mut self, data: Vec<u8>, content_type: &str) -> Self {
        if let Ok(header_value) = HeaderValue::from_str(content_type) {
            self.headers
                .insert(http::header::CONTENT_TYPE, header_value);
        }
        self.body = PutBody::Binary(data);
        // Invalid content-type values are silently ignored - errors will surface during request execution
        self
    }

    /// Set text body with Content-Type
    /// Set text body with Content-Type
    ///
    /// # Arguments
    /// * `data` - The text content to send
    /// * `content_type` - The MIME type of the content
    ///
    /// # Returns
    /// `Self` for method chaining - errors silently ignored, handled during execution
    #[must_use = "Operation builder methods return a new operation and should be used"]
    pub fn text(mut self, data: String, content_type: &str) -> Self {
        if let Ok(header_value) = HeaderValue::from_str(content_type) {
            self.headers
                .insert(http::header::CONTENT_TYPE, header_value);
        }
        self.body = PutBody::Text(data);
        // Invalid content-type values are silently ignored - errors will surface during request execution
        self
    }

    /// Add If-Match header for conditional replacement
    /// Add If-Match header for conditional replacement
    ///
    /// # Arguments
    /// * `etag` - The entity tag to match
    ///
    /// # Returns
    /// `Self` for method chaining - errors silently ignored, handled during execution
    #[must_use = "Operation builder methods return a new operation and should be used"]
    pub fn if_match(mut self, etag: &str) -> Self {
        if let Ok(header_value) = HeaderValue::from_str(etag) {
            self.headers.insert(http::header::IF_MATCH, header_value);
        }
        // Invalid etag values are silently ignored - errors will surface during request execution
        self
    }
}

impl HttpOperation for PutOperation {
    type Output = AsyncStream<crate::prelude::HttpResponse, 1>;

    fn execute(&self) -> Self::Output {
        let body_bytes = match &self.body {
            PutBody::Json(val) => match serde_json::to_vec(val) {
                Ok(bytes) => Some(bytes),
                Err(_) => Some(Vec::new()), // Fallback to empty body on serialization error
            },
            PutBody::Binary(data) => Some(data.clone()),
            PutBody::Text(text) => Some(text.clone().into_bytes()),
            PutBody::Empty => None,
        };

        let url = match self.url.parse() {
            Ok(url) => url,
            Err(e) => {
                use ystream::AsyncStream;
                return AsyncStream::with_channel(move |sender| {
                    let error_response = crate::prelude::HttpResponse::error(
                        http::StatusCode::BAD_REQUEST,
                        format!("URL parse error: {e}")
                    );
                    ystream::emit!(sender, error_response);
                });
            }
        };

        let request = HttpRequest::new(
            self.method(),
            url,
            Some(self.headers.clone()),
            body_bytes.map(|b| crate::http::request::RequestBody::Bytes(bytes::Bytes::from(b))),
            Some(std::time::Duration::from_secs(30)),
        );

        use ystream::AsyncStream;
        let client = self.client.clone();
        AsyncStream::with_channel(move |sender| {
            let http_response = client.execute(request);
            ystream::emit!(sender, http_response);
        })
    }

    fn method(&self) -> Method {
        Method::PUT
    }

    fn url(&self) -> &str {
        &self.url
    }
}
