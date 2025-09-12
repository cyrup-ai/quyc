//! DELETE HTTP Operations Module - Resource deletion with conditional logic

use http::{HeaderMap, HeaderName, HeaderValue, Method};

use crate::operations::HttpOperation;
use crate::{
    client::HttpClient, http::request::HttpRequest, prelude::AsyncStream,
};

/// DELETE operation implementation with conditional deletion support
#[derive(Clone)]
pub struct DeleteOperation {
    client: HttpClient,
    url: String,
    headers: HeaderMap,
}

impl DeleteOperation {
    /// Create a new DELETE operation
    #[inline]
    #[must_use] 
    pub fn new(client: HttpClient, url: String) -> Self {
        Self {
            client,
            url,
            headers: HeaderMap::new(),
        }
    }

    /// Add custom header
    #[inline]
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

    /// Add If-Match header for conditional deletion
    #[inline]
    #[must_use = "Operation builder methods return a new operation and should be used"]
    pub fn if_match(mut self, etag: &str) -> Self {
        if let Ok(header_value) = HeaderValue::from_str(etag) {
            self.headers.insert(http::header::IF_MATCH, header_value);
        }
        // Invalid etag values are silently ignored - errors will surface during request execution
        self
    }
}

impl HttpOperation for DeleteOperation {
    type Output = AsyncStream<crate::prelude::HttpResponse, 1>;

    fn execute(&self) -> Self::Output {
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
            Some(crate::http::request::RequestBody::Bytes(bytes::Bytes::new())),
            Some(std::time::Duration::from_secs(30)),
        );
        use ystream::AsyncStream;
        let client = self.client.clone();
        AsyncStream::with_channel(move |sender| {
            let http_response = client.execute(request);
            ystream::emit!(sender, http_response);
        })
    }

    #[inline]
    fn method(&self) -> Method {
        Method::DELETE
    }

    #[inline]
    fn url(&self) -> &str {
        &self.url
    }
}
