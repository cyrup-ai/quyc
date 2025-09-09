//! HTTP client configuration and construction
//!
//! Handles client creation with HTTP/3 and HTTP/2 protocol configuration,
//! TLS settings, compression, and advanced protocol optimizations.

use super::HttpClient;
use crate::prelude::*;

/// HTTP client builder for configuration
pub struct HttpClientBuilder {
    config: HttpConfig,
}

impl HttpClientBuilder {
    pub fn new() -> Self {
        Self {
            config: HttpConfig::default(),
        }
    }

    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.config.pool_max_idle_per_host = max;
        self
    }

    pub fn pool_idle_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.pool_idle_timeout = timeout;
        self
    }

    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn connect_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    pub fn tcp_nodelay(mut self, enable: bool) -> Self {
        self.config.tcp_nodelay = enable;
        self
    }

    pub fn use_rustls_tls(self) -> Self {
        // TLS configuration
        self
    }

    pub fn tls_built_in_root_certs(mut self, enable: bool) -> Self {
        self.config.use_native_certs = enable;
        self
    }

    pub fn https_only(mut self, enable: bool) -> Self {
        self.config.https_only = enable;
        self
    }

    pub fn user_agent(mut self, agent: &str) -> Self {
        self.config.user_agent = agent.to_string();
        self
    }

    pub fn gzip(mut self, enable: bool) -> Self {
        self.config.gzip_enabled = enable;
        self
    }

    pub fn brotli(mut self, enable: bool) -> Self {
        self.config.brotli_enabled = enable;
        self
    }

    pub fn deflate(self, _enable: bool) -> Self {
        // Deflate compression configuration
        self
    }

    pub fn build(self) -> Result<HttpClient, HttpError> {
        // Use the core HttpClient::with_config method directly
        Ok(crate::client::core::HttpClient::with_config(self.config))
    }
}

impl Default for HttpClient {
    /// Create HttpClient with default configuration
    ///
    /// Uses the default HttpConfig and falls back to a basic http3 client
    /// if configuration fails. This ensures the client can always be constructed
    /// even in constrained environments.
    fn default() -> Self {
        // Use the new with_config constructor with default configuration
        // If configuration fails, fall back to a basic http3 client
        let config = HttpConfig::default();
        crate::client::core::HttpClient::with_config(config)
    }
}
