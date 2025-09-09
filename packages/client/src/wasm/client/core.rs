//! Canonical WASM HTTP client extending HttpClient with browser capabilities
//!
//! Provides WasmClient that wraps the canonical HttpClient with WASM-specific
//! browser integration, fetch API support, and ergonomic builder patterns.

use std::{fmt, sync::Arc};

use ystream::AsyncStream;
use http::{HeaderMap, Method, Uri, header::Entry};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::UnwrapThrowExt as _;

use super::config::Config;
use crate::client::core::HttpClient;
use crate::wasm::request::types::WasmRequest;
use crate::wasm::response::WasmResponse;
use crate::{HttpConfig, HttpRequest};

/// Canonical WASM HTTP client extending HttpClient with browser capabilities
/// 
/// Wraps the canonical HttpClient to provide WASM-specific features like
/// browser fetch API integration, CORS handling, and credential management.
#[derive(Clone)]
pub struct WasmClient {
    /// Canonical HTTP client for core functionality
    http_client: HttpClient,
    /// WASM-specific configuration
    wasm_config: Arc<Config>,
}

impl WasmClient {
    /// Create new WASM client with default configuration
    pub fn new() -> Self {
        Self::builder().build().unwrap_throw()
    }

    /// Create WASM client with existing configuration
    pub(super) fn new_with_config(config: Config) -> Self {
        let http_config = HttpConfig::default();
        let http_client = HttpClient::with_config(http_config);
        
        Self {
            http_client,
            wasm_config: Arc::new(config),
        }
    }

    /// Create WASM client builder
    pub fn builder() -> super::builder::WasmClientBuilder {
        super::builder::WasmClientBuilder::new()
    }

    /// Get reference to underlying HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Convenience method to make a GET request with WASM extensions
    pub fn get(&self, uri: Uri) -> WasmRequest {
        WasmRequest::builder()
            .method(Method::GET)
            .uri(uri)
            .headers(self.wasm_config.headers.clone())
            .build()
    }

    /// Convenience method to make a POST request with WASM extensions
    pub fn post(&self, uri: Uri) -> WasmRequest {
        WasmRequest::builder()
            .method(Method::POST)
            .uri(uri)
            .headers(self.wasm_config.headers.clone())
            .build()
    }

    /// Convenience method to make a PUT request with WASM extensions
    pub fn put(&self, uri: Uri) -> WasmRequest {
        WasmRequest::builder()
            .method(Method::PUT)
            .uri(uri)
            .headers(self.wasm_config.headers.clone())
            .build()
    }

    /// Convenience method to make a PATCH request with WASM extensions
    pub fn patch(&self, uri: Uri) -> WasmRequest {
        WasmRequest::builder()
            .method(Method::PATCH)
            .uri(uri)
            .headers(self.wasm_config.headers.clone())
            .build()
    }

    /// Convenience method to make a DELETE request with WASM extensions
    pub fn delete(&self, uri: Uri) -> WasmRequest {
        WasmRequest::builder()
            .method(Method::DELETE)
            .uri(uri)
            .headers(self.wasm_config.headers.clone())
            .build()
    }

    /// Convenience method to make a HEAD request with WASM extensions
    pub fn head(&self, uri: Uri) -> WasmRequest {
        WasmRequest::builder()
            .method(Method::HEAD)
            .uri(uri)
            .headers(self.wasm_config.headers.clone())
            .build()
    }

    /// Build a WASM request with method and URI
    /// 
    /// Returns a WasmRequest builder with WASM-specific browser capabilities
    pub fn request(&self, method: Method, uri: Uri) -> WasmRequest {
        WasmRequest::builder()
            .method(method)
            .uri(uri)
            .headers(self.wasm_config.headers.clone())
            .build()
    }

    /// Execute WASM request with browser fetch API integration
    /// 
    /// Converts WasmRequest to HttpRequest, executes via canonical HttpClient,
    /// then wraps response with WASM-specific browser capabilities.
    pub fn execute(&self, wasm_request: WasmRequest) -> AsyncStream<WasmResponse, 1024> {
        use ystream::{AsyncStream, emit};

        // Convert WasmRequest to HttpRequest for canonical client
        let http_request = wasm_request.into_http_request();
        
        // Execute via canonical HttpClient
        let response_stream = self.http_client.execute(http_request);
        
        AsyncStream::with_channel(move |sender| {
            for streaming_response in response_stream {
                // Wrap HttpResponse with WASM capabilities
                let wasm_response = WasmResponse::from_http_response(streaming_response);
                emit!(sender, wasm_response);
            }
        })
    }

    /// Merge WASM configuration headers with request headers
    pub(super) fn merge_headers(&self, headers: &mut HeaderMap) {
        // Insert WASM default headers without overwriting existing headers
        for (key, value) in self.wasm_config.headers.iter() {
            if let Entry::Vacant(entry) = headers.entry(key) {
                entry.insert(value.clone());
            }
        }
    }
}

impl Default for WasmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for WasmClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = f.debug_struct("WasmClient");
        builder.field("http_client", &self.http_client);
        self.wasm_config.fmt_fields(&mut builder);
        builder.finish()
    }
}
