//! # Fluent AI HTTP3 Client
//!
//! Zero-allocation HTTP/3 (QUIC) client with HTTP/2 fallback designed for AI providers.
//! Provides blazing-fast performance with connection pooling, intelligent caching,
//! and comprehensive error handling.
//!
//! ## Features
//!
//! - **HTTP/3 (QUIC) prioritization** with HTTP/2 fallback
//! - **Zero-allocation design** for maximum performance
//! - **Connection pooling** with intelligent reuse
//! - **Rustls TLS** with native root certificates
//! - **Compression support** (gzip, brotli, deflate)
//! - **Intelligent caching** with `ETag` and conditional requests
//! - **Streaming support** for real-time AI responses
//! - **File download streaming** with progress tracking and `on_chunk` Result handlers
//! - **Request/Response middleware** for customization
//! - **Comprehensive error handling** with detailed diagnostics
//!
//! ## Usage

#![feature(impl_trait_in_assoc_type)]
#![feature(impl_trait_in_fn_trait_return)]
#[deny(unsafe_code)]
#[warn(clippy::all)]
#[warn(clippy::pedantic)]

// Removed orphan rule violation - MessageChunk implementations moved to appropriate modules
// ### Basic HTTP Requests
//
// ```rust
// use quyc::Http3;
// use serde::Deserialize;
//
// #[derive(Deserialize)]
// struct ApiResponse {
//     status: String,
//     data: Vec<String>,
// }
//
// fn main() {
//     let response: ApiResponse = Http3::json()
//         .headers([("Authorization", "Bearer sk-...")])
//         .get("https://api.openai.com/v1/models")
//         .collect_one_or_else(|_e| ApiResponse {
//             status: "error".to_string(),
//             data: vec![],
//         });
//
//     println!("Status: {}", response.status);
//     println!("Data count: {}", response.data.len());
// }
// ```
//
// ### File Downloads with Progress Tracking
//
// ```rust
// use quyc::{Http3, DownloadChunk};
//
// fn main() {
//     let progress = Http3::new()
//         .download_file("https://example.com/large-file.zip")
//         .on_chunk(|result| match result {
//             Ok(chunk) => {
//                 if let Some(progress) = chunk.progress_percentage() {
//                     println!("Download progress: {:.1}%", progress);
//                 }
//                 println!("Downloaded {} bytes", chunk.bytes_downloaded);
//                 chunk
//             },
//             Err(error) => {
//                 println!("Download error: {}", error);
//                 DownloadChunk::bad_chunk(error.to_string())
//             }
//         })
//         .save("/tmp/large-file.zip");
//
//     println!("Download complete! Total size: {} bytes", progress.total_bytes);
// }
// ```

use std::sync::Arc;
use std::sync::OnceLock;

// Core modules
pub mod auth;
pub mod builder;
pub mod cache;
pub mod client;
pub mod config;
pub mod connect;
pub mod cookie;
pub mod crypto;
pub mod error;
pub mod http;
pub mod jsonpath;
pub mod middleware;
pub mod operations;
pub mod protocols;
pub mod proxy;
pub mod retry;
pub mod security;
pub mod telemetry;
pub mod tls;


// Prelude with canonical types
pub mod prelude;

// Essential public API - only what end users actually need
pub use crate::prelude::*;

// Builder convenience alias - this is genuinely ergonomic
pub type Http3 = builder::Http3Builder;

/// Global HTTP client instance with connection pooling
/// Uses the Default implementation which provides graceful fallback handling
/// in case the optimized configuration fails to initialize
static GLOBAL_CLIENT: OnceLock<Arc<HttpClient>> = OnceLock::new();

/// Get the global HTTP client instance
/// This provides a shared, high-performance HTTP client with connection pooling
/// and QUIC/HTTP3 support. The client is initialized once and reused across
/// all requests for maximum efficiency.
pub fn global_client() -> Arc<HttpClient> {
    GLOBAL_CLIENT
        .get_or_init(|| Arc::new(HttpClient::default()))
        .clone()
}

// Note: Convenience functions removed in favor of modular operations architecture.
// Use HttpClient directly or the global_client() function to access operation builders.

/// Get connection pool statistics
#[must_use]
pub fn connection_stats() -> ClientStatsSnapshot {
    global_client().as_ref().stats().snapshot()
}

/// Initialize the global HTTP client with custom configuration
/// Per fluent-ai architecture: NO Result returns, graceful error handling with fallback
pub fn init_global_client(config: HttpConfig) {
    if let Err(e) = validate_and_init_client(config) {
        // Log error and continue with default client - library code should never panic
        tracing::error!(
            "Failed to initialize HTTP client with custom config: {}, using default client",
            e
        );
        // Default client is already initialized via global_client() - graceful degradation
    }
}

/// Internal validation and initialization
fn validate_and_init_client(
    config: HttpConfig,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Validate the provided configuration
    validate_http_config(&config)?;

    // Initialize global client with the provided configuration
    initialize_global_client_internal(config)
}

/// Validate HTTP configuration before initialization
fn validate_http_config(config: &HttpConfig) -> std::result::Result<(), String> {
    // Validate timeout values
    if config.timeout.as_secs() == 0 {
        return Err("Timeout must be greater than zero".to_string());
    }
    if config.timeout.as_secs() > 3600 {
        return Err("Timeout must not exceed 1 hour".to_string());
    }

    // Validate connection timeout
    if config.connect_timeout.as_secs() == 0 {
        return Err("Connect timeout must be greater than zero".to_string());
    }
    if config.connect_timeout.as_secs() > 300 {
        return Err("Connect timeout must not exceed 5 minutes".to_string());
    }

    // Validate pool configuration
    if config.pool_max_idle_per_host == 0 {
        return Err("Pool max idle per host must be greater than zero".to_string());
    }
    if config.pool_max_idle_per_host > 1000 {
        return Err("Pool max idle per host must not exceed 1000".to_string());
    }

    // Validate user agent
    if config.user_agent.is_empty() {
        return Err("User agent cannot be empty".to_string());
    }
    if config.user_agent.len() > 1000 {
        return Err("User agent must not exceed 1000 characters".to_string());
    }

    Ok(())
}

/// Internal function to perform the actual global client initialization
fn initialize_global_client_internal(
    config: HttpConfig,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // ELIMINATED: hyper ClientBuilder - using direct ystream streaming

    // Build client with pure AsyncStream architecture - NO middleware
    let stats = crate::client::core::ClientStats::default();
    let new_client = crate::client::HttpClient::new_direct(config, stats);

    // Set the global client - OnceLock allows one-time initialization
    GLOBAL_CLIENT
        .set(Arc::new(new_client))
        .map_err(|_| "Global client already initialized")?;

    Ok(())
}
