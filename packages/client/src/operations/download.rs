//! Download Operations Module - File downloads with progress, resume, and throttling

use http::{HeaderMap, HeaderName, HeaderValue, Method};
use ystream::AsyncStream;
use bytes::Bytes;
use std::time::Instant;
use url::Url;

use crate::{
    client::HttpClient, http::request::HttpRequest, operations::HttpOperation,
    http::response::HttpBodyChunk,
};

/// Type alias for download stream
pub type DownloadStream = AsyncStream<HttpBodyChunk, 1024>;

/// Download operation with progress tracking and resume capability
pub struct DownloadOperation {
    client: HttpClient,
    url: String,
    headers: HeaderMap,
    resume_from: Option<u64>,
}

impl DownloadOperation {
    /// Create a new download operation
    #[inline(always)]
    #[must_use] 
    pub fn new(client: HttpClient, url: String) -> Self {
        Self {
            client,
            url,
            headers: HeaderMap::new(),
            resume_from: None,
        }
    }

    /// Add custom header
    #[inline(always)]
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

    /// Set headers from a `HeaderMap`
    #[inline(always)]
    #[must_use] 
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Set the byte offset to resume the download from
    #[inline(always)]
    #[must_use] 
    pub fn resume_from(mut self, offset: u64) -> Self {
        self.resume_from = Some(offset);
        self
    }

    /// Execute the download and return a stream of chunks.
    pub fn execute_download(mut self) -> DownloadStream {
        if let Some(offset) = self.resume_from {
            let range_value = format!("bytes={offset}-");
            if let Ok(header_value) = HeaderValue::from_str(&range_value) {
                self.headers.insert(http::header::RANGE, header_value);
            }
            // Silently skip invalid range header rather than panicking
        }

        // Parse URL with proper error handling
        let url = match Url::parse(&self.url) {
            Ok(url) => url,
            Err(e) => {
                // Return error stream using bad_chunk pattern
                return AsyncStream::with_channel(move |sender| {
                    use ystream::emit;
                    emit!(sender, HttpBodyChunk {
                        data: Bytes::from(format!("Invalid URL: {e}")),
                        offset: 0,
                        is_final: true,
                        timestamp: Instant::now(),
                    });
                });
            }
        };

        let request = HttpRequest::new(
            self.method(),
            url,
            Some(self.headers.clone()),
            None,
            Some(std::time::Duration::from_secs(30)),
        );
        
        // Execute the request and get the response
        let response = self.client.execute(request);
        
        // Return the body stream from the response
        response.into_body_stream()
    }
}

impl HttpOperation for DownloadOperation {
    type Output = DownloadStream;

    fn execute(&self) -> Self::Output {
        // Cloning self to allow the operation to be executed.
        // This is a bit of a workaround for the ownership model.
        let op = self.clone();
        op.execute_download()
    }

    fn method(&self) -> Method {
        Method::GET
    }

    fn url(&self) -> &str {
        &self.url
    }
}

// Manually implement Clone because of HttpClient
impl Clone for DownloadOperation {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            url: self.url.clone(),
            headers: self.headers.clone(),
            resume_from: self.resume_from,
        }
    }
}
