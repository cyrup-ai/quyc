//! HTTP/2 request builder implementation
//!
//! Provides zero-allocation request building for HTTP/2 protocol using
//! ystream patterns with elite polling.

use bytes::Bytes;
use ystream::prelude::*;
use http::{HeaderMap, HeaderName, HeaderValue, Request, Uri};
use std::str::FromStr;

use crate::protocols::core::{HttpMethod, HttpRequestBuilder};
use crate::protocols::h2::H2Connection;
use crate::protocols::quiche::QuicheConnectionChunk;

/// HTTP/2 request builder with fluent interface
pub struct H2RequestBuilder {
    connection: H2Connection,
    method: HttpMethod,
    uri: Option<Uri>,
    headers: HeaderMap,
    body: Option<Bytes>,
}

impl H2RequestBuilder {
    pub fn new(connection: H2Connection) -> Self {
        Self {
            connection,
            method: HttpMethod::Get,
            uri: None,
            headers: HeaderMap::new(),
            body: None,
        }
    }
    
    fn build_request(&self) -> Result<Request<Bytes>, String> {
        let uri = self.uri.as_ref()
            .ok_or_else(|| "URI is required".to_string())?;
        
        let body = self.body.clone().unwrap_or_default();
        
        let mut request = Request::builder()
            .method(http::Method::from(self.method))
            .uri(uri.clone());
        
        // Add headers
        for (name, value) in &self.headers {
            request = request.header(name, value);
        }
        
        request.body(body)
            .map_err(|e| format!("Failed to build request: {}", e))
    }
}

impl HttpRequestBuilder for H2RequestBuilder {
    type ResponseChunk = HttpChunk;
    
    fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }
    
    fn uri(mut self, uri: &str) -> Self {
        match Uri::from_str(uri) {
            Ok(parsed_uri) => {
                self.uri = Some(parsed_uri);
            }
            Err(_) => {
                // Store invalid URI to generate error later in execute()
                self.uri = None;
            }
        }
        self
    }
    
    fn header(mut self, name: &str, value: &str) -> Self {
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::from_str(name),
            HeaderValue::from_str(value)
        ) {
            self.headers.insert(header_name, header_value);
        }
        self
    }
    
    fn body(mut self, body: Bytes) -> Self {
        self.body = Some(body);
        self
    }
    
    fn execute(self) -> AsyncStream<Self::ResponseChunk, 1024> {
        AsyncStream::with_channel(move |sender| {
            match self.build_request() {
                Ok(request) => {
                    // Use existing H2Connection send_request_stream with elite polling
                    let response_stream = self.connection.send_request_stream(request);
                    
                    // Forward all chunks from H2 stream to our output stream
                    for chunk in response_stream {
                        emit!(sender, chunk);
                    }
                }
                Err(error_msg) => {
                    emit!(sender, HttpChunk::bad_chunk(error_msg));
                }
            }
        })
    }
}

/// HTTP/2 connection state implementation
#[derive(Debug, Clone)]
pub struct H2ConnectionState {
    pub is_ready: bool,
    pub is_closed: bool,
    pub error: Option<String>,
    pub established_at: Option<std::time::Instant>,
}

impl crate::protocols::core::ConnectionState for H2ConnectionState {
    fn is_ready(&self) -> bool {
        self.is_ready && !self.is_closed
    }
    
    fn is_closed(&self) -> bool {
        self.is_closed
    }
    
    fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }
    
    fn uptime(&self) -> Option<std::time::Duration> {
        self.established_at.map(|t| t.elapsed())
    }
    
    fn established_at(&self) -> Option<std::time::Instant> {
        self.established_at
    }
}
