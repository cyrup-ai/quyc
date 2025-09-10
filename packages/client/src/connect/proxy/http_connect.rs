//! HTTP CONNECT proxy configuration
//!
//! This module contains HTTP CONNECT proxy support with authentication
//! and custom header capabilities for tunneling connections.

/// HTTP CONNECT proxy configuration
#[derive(Clone, Debug)]
pub struct HttpConnectConfig {
    pub target_host: String,
    pub target_port: u16,
    pub auth: Option<String>, // Basic auth header value
    pub custom_headers: Option<hyper::HeaderMap>,
}

impl HttpConnectConfig {
    /// Create new HTTP CONNECT configuration
    pub fn new(target_host: String, target_port: u16) -> Self {
        Self {
            target_host,
            target_port,
            auth: None,
            custom_headers: None,
        }
    }

    /// Add basic authentication
    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        use base64::Engine;
        let credentials = format!("{username}:{password}");
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        self.auth = Some(encoded);
        self
    }

    /// Add custom headers
    pub fn with_headers(mut self, headers: hyper::HeaderMap) -> Self {
        self.custom_headers = Some(headers);
        self
    }
}
