//! Core HTTP protocol types
//!
//! Provides core types and configuration traits for HTTP/2, HTTP/3, and QUIC protocols
//! using ystream patterns.

use std::time::Duration;

// bytes import removed - not used
// ystream imports removed - not used

use crate::config::HttpConfig;

/// HTTP protocol versions supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpVersion {
    Http2,
    Http3,
}

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
    Trace,
    Connect,
}

impl From<HttpMethod> for http::Method {
    #[inline]
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => http::Method::GET,
            HttpMethod::Post => http::Method::POST,
            HttpMethod::Put => http::Method::PUT,
            HttpMethod::Delete => http::Method::DELETE,
            HttpMethod::Head => http::Method::HEAD,
            HttpMethod::Options => http::Method::OPTIONS,
            HttpMethod::Patch => http::Method::PATCH,
            HttpMethod::Trace => http::Method::TRACE,
            HttpMethod::Connect => http::Method::CONNECT,
        }
    }
}

/// Timeout configuration for protocol operations
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub request_timeout: Duration,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub keepalive_timeout: Option<Duration>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(90),
            keepalive_timeout: Some(Duration::from_secs(60)),
        }
    }
}


/// Protocol-specific configuration trait
pub trait ProtocolConfig: Clone + Send + Sync {
    /// Validate configuration parameters
    fn validate(&self) -> Result<(), String>;

    /// Get timeout settings
    fn timeout_config(&self) -> TimeoutConfig;

    /// Convert to HttpConfig for compatibility
    fn to_http_config(&self) -> HttpConfig;
}

/// Connection establishment state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Idle,
    Closed,
    Error,
}

/// Protocol capability flags
#[derive(Debug, Clone, Copy)]
pub struct ProtocolCapabilities {
    pub supports_multiplexing: bool,
    pub supports_server_push: bool,
    pub supports_early_data: bool,
    pub supports_0rtt: bool,
    pub max_concurrent_streams: Option<u32>,
}

impl ProtocolCapabilities {

    pub const fn http2() -> Self {
        Self {
            supports_multiplexing: true,
            supports_server_push: true,
            supports_early_data: false,
            supports_0rtt: false,
            max_concurrent_streams: Some(100),
        }
    }

    pub const fn http3() -> Self {
        Self {
            supports_multiplexing: true,
            supports_server_push: false,
            supports_early_data: true,
            supports_0rtt: true,
            max_concurrent_streams: Some(1000),
        }
    }
}
