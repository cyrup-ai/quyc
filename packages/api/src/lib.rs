//! Fluent AI HTTP3 Public API
//!
//! Zero-allocation HTTP/3 client with fluent builder pattern and streaming-first architecture.
//! All HTTP operations return streams by default, with `.collect()` available for traditional usage.

#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

pub mod builder;

// Re-export all public API components
pub use builder::*;

// Re-export important types from client package
pub use quyc_client::{
    HttpChunk, HttpClient, HttpError, HttpRequest
};
pub use quyc_client::builder::fluent::DownloadBuilder;

/// Error chunk for MessageChunk pattern compatibility
#[derive(Debug, Clone)]
pub struct BadChunk {
    pub error: String,
}

impl BadChunk {
    pub fn new(error: String) -> Self {
        Self { error }
    }
}



// Main builder type alias for convenience
pub use builder::core::Http3Builder;

/// Main Http3 entry point providing static builder methods
pub struct Http3;

impl Http3 {
    /// Create a new JSON HTTP builder
    /// 
    /// Shorthand for `Http3Builder::json()`
    pub fn json() -> Http3Builder {
        Http3Builder::json()
    }



    /// Create a new form-urlencoded HTTP builder
    /// 
    /// Shorthand for `Http3Builder::form_urlencoded()`
    pub fn form_urlencoded() -> Http3Builder {
        Http3Builder::form_urlencoded()
    }

    /// Create a new HTTP builder with custom client
    /// 
    /// # Arguments
    /// * `client` - Custom HttpClient instance
    /// 
    /// # Returns
    /// `Http3Builder` for method chaining
    pub fn with_client(client: &HttpClient) -> Http3Builder {
        Http3Builder::new(client)
    }
}

/// Create a new JSON HTTP builder
/// 
/// Shorthand for `Http3Builder::json()`
pub fn json() -> Http3Builder {
    Http3Builder::json()
}

/// Create a new form-urlencoded HTTP builder
/// 
/// Shorthand for `Http3Builder::form_urlencoded()`
pub fn form() -> Http3Builder {
    Http3Builder::form_urlencoded()
}

/// Create a new HTTP builder with custom client
/// 
/// # Arguments
/// * `client` - Custom HttpClient instance
/// 
/// # Returns
/// `Http3Builder` for method chaining
pub fn with_client(client: &HttpClient) -> Http3Builder {
    Http3Builder::new(client)
}