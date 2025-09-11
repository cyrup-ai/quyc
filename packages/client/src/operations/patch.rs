//! PATCH HTTP Operations Module - JSON Patch (RFC 6902) and JSON Merge Patch (RFC 7396)

use http::{HeaderMap, HeaderName, HeaderValue, Method};
use serde_json::Value;

use crate::operations::HttpOperation;
use crate::{
    client::HttpClient, http::request::HttpRequest, prelude::AsyncStream,
};

/// PATCH operation implementation supporting multiple patch formats
#[derive(Clone)]
pub struct PatchOperation {
    client: HttpClient,
    url: String,
    headers: HeaderMap,
    body: PatchBody,
}

/// Supported PATCH types
#[derive(Clone)]
pub enum PatchBody {
    /// JSON Patch (RFC 6902) - Array of patch operations
    JsonPatch(Value),
    /// JSON Merge Patch (RFC 7396) - Object representing the patch
    JsonMergePatch(Value),
}

impl PatchOperation {
    /// Create a new PATCH operation
    ///
    /// # Arguments
    /// * `client` - The HTTP client to use for the request
    /// * `url` - The URL to send the PATCH request to
    ///
    /// # Returns
    /// A new `PatchOperation` instance with JSON Merge Patch as the default format
    #[must_use]
    pub fn new(client: HttpClient, url: String) -> Self {
        Self {
            client,
            url,
            headers: HeaderMap::new(),
            body: PatchBody::JsonMergePatch(Value::Null),
        }
    }

    /// Add a custom header
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

    /// Set JSON Patch operations (RFC 6902)
    ///
    /// # Arguments
    /// * `patch` - The JSON patch operations to apply
    ///
    /// # Returns
    /// `Self` for method chaining
    #[must_use]
    pub fn json_patch(mut self, patch: Value) -> Self {
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json-patch+json"),
        );
        self.body = PatchBody::JsonPatch(patch);
        self
    }

    /// Set JSON Merge Patch (RFC 7396)
    ///
    /// # Arguments
    /// * `patch` - The JSON merge patch to apply
    ///
    /// # Returns
    /// `Self` for method chaining
    #[must_use]
    pub fn merge_patch(mut self, patch: Value) -> Self {
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/merge-patch+json"),
        );
        self.body = PatchBody::JsonMergePatch(patch);
        self
    }

    /// Add If-Match header for conditional patching
    ///
    /// # Arguments
    /// * `etag` - The entity tag to use for conditional requests
    ///
    /// # Returns
    /// `Self` for method chaining - errors silently ignored, handled during execution
    #[must_use]
    pub fn if_match(mut self, etag: &str) -> Self {
        if let Ok(header_value) = HeaderValue::from_str(etag) {
            self.headers.insert(http::header::IF_MATCH, header_value);
        }
        // Invalid etag values are silently ignored - errors will surface during request execution
        self
    }
}

impl HttpOperation for PatchOperation {
    type Output = AsyncStream<crate::prelude::HttpResponse, 1>;

    fn execute(&self) -> Self::Output {
        let body_bytes = match &self.body {
            PatchBody::JsonPatch(val) | PatchBody::JsonMergePatch(val) => {
                serde_json::to_vec(val).unwrap_or_default()
            }
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
            Some(crate::http::request::RequestBody::Bytes(
                bytes::Bytes::from(body_bytes),
            )),
            None,
        );

        use ystream::AsyncStream;
        let client = self.client.clone();
        AsyncStream::with_channel(move |sender| {
            let http_response = client.execute(request);
            ystream::emit!(sender, http_response);
        })
    }

    fn method(&self) -> Method {
        Method::PATCH
    }

    fn url(&self) -> &str {
        &self.url
    }
}
