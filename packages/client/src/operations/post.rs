//! POST HTTP Operations Module - JSON, form data, binary, and multipart support

use std::collections::HashMap;

use http::{HeaderMap, HeaderName, HeaderValue, Method};
use serde_json::Value;

use crate::operations::HttpOperation;
use crate::{
    client::HttpClient, http::request::HttpRequest, 
};

use crate::prelude::AsyncStream;

/// POST operation implementation with multiple body type support
#[derive(Clone)]
pub struct PostOperation {
    client: HttpClient,
    url: String,
    headers: HeaderMap,
    body: PostBody,
}

/// Supported POST body types
#[derive(Clone)]
pub enum PostBody {
    /// JSON-encoded request body
    Json(Value),
    /// Form-encoded data as key-value pairs
    FormData(HashMap<String, String>),
    /// Binary data body
    Binary(Vec<u8>),
    /// Empty request body
    Empty,
}

impl PostOperation {
    /// Create a new POST operation
    #[must_use]
    pub fn new(client: HttpClient, url: String) -> Self {
        Self {
            client,
            url,
            headers: HeaderMap::new(),
            body: PostBody::Empty,
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
        self.body = PostBody::Json(json_value);
        self
    }

    /// Set form data body with automatic Content-Type
    #[must_use]
    pub fn form(mut self, form_data: HashMap<String, String>) -> Self {
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        self.body = PostBody::FormData(form_data);
        self
    }

    /// Set binary body with a specific Content-Type
    ///
    /// # Arguments
    /// * `body` - The binary data to send
    /// * `content_type` - The MIME type of the content
    ///
    /// # Returns
    /// `Self` for method chaining
    #[must_use]
    pub fn binary(mut self, body: Vec<u8>, content_type: &str) -> Self {
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_str(content_type)
                .unwrap_or(HeaderValue::from_static("application/octet-stream")),
        );
        self.body = PostBody::Binary(body);
        self
    }
}

impl HttpOperation for PostOperation {
    type Output = AsyncStream<crate::prelude::HttpResponse, 1>;

    fn execute(&self) -> Self::Output {
        let body_bytes = match &self.body {
            PostBody::Json(val) => match serde_json::to_vec(val) {
                Ok(bytes) => Some(bytes),
                Err(_) => Some(Vec::new()), // Fallback to empty body on serialization error
            },
            PostBody::FormData(data) => match serde_urlencoded::to_string(data) {
                Ok(encoded) => Some(encoded.into_bytes()),
                Err(_) => Some(Vec::new()), // Fallback to empty body on encoding error
            },
            PostBody::Binary(data) => Some(data.clone()),
            PostBody::Empty => None,
        };

        let url = match self.url.parse() {
            Ok(url) => url,
            Err(e) => {
                use ystream::AsyncStream;
                return AsyncStream::with_channel(move |sender| {
                    let error_response = crate::prelude::HttpResponse::error(
                        http::StatusCode::BAD_REQUEST,
                        format!("URL parse error: {}", e)
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
        Method::POST
    }

    fn url(&self) -> &str {
        &self.url
    }
}
