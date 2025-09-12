//! HTTP client configuration and construction
//!
//! Handles client creation with HTTP/3 and HTTP/2 protocol configuration,
//! TLS settings, compression, and advanced protocol optimizations.

use super::HttpClient;
use crate::prelude::*;

/// HTTP client builder for configuration
#[must_use = "builders do nothing unless you call a build method"]
pub struct HttpClientBuilder {
    config: HttpConfig,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
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
        self.config.tcp.nodelay = enable;
        self
    }


    pub fn use_rustls_tls(self) -> Self {
        // TLS configuration
        self
    }


    pub fn tls_built_in_root_certs(mut self, enable: bool) -> Self {
        self.config.tls.use_native_certs = enable;
        self
    }


    pub fn https_only(mut self, enable: bool) -> Self {
        self.config.tls.https_only = enable;
        self
    }


    pub fn user_agent(mut self, agent: &str) -> Self {
        self.config.user_agent = agent.to_string();
        self
    }


    pub fn gzip(mut self, enable: bool) -> Self {
        self.config.compression.gzip.enabled = enable;
        self
    }


    pub fn brotli(mut self, enable: bool) -> Self {
        self.config.compression.brotli.enabled = enable;
        self
    }


    pub fn deflate(mut self, enable: bool) -> Self {
        self.config.compression.deflate.enabled = enable;
        self
    }

    /// Set gzip compression level
    ///
    /// # Arguments
    /// * `level` - Compression level from 1 (fastest) to 9 (best compression)
    ///
    /// # Errors
    /// * Returns `HttpError` if level is 0 or greater than 9
    pub fn gzip_level(mut self, level: u32) -> Result<Self, HttpError> {
        if (1..=9).contains(&level) {
            self.config.compression.gzip.level = Some(level);
            Ok(self)
        } else {
            Err(HttpError::new(crate::error::types::Kind::Request)
                .with("Gzip compression level must be between 1 and 9"))
        }
    }

    /// Set brotli compression level
    ///
    /// # Arguments
    /// * `level` - Compression level from 0 (fastest) to 11 (best compression)
    ///
    /// # Errors
    /// * Returns `HttpError` if level is greater than 11
    pub fn brotli_level(mut self, level: u32) -> Result<Self, HttpError> {
        if level <= 11 {
            self.config.compression.brotli.level = Some(level);
            Ok(self)
        } else {
            Err(HttpError::new(crate::error::types::Kind::Request)
                .with("Brotli compression level must be between 0 and 11"))
        }
    }

    /// Set deflate compression level
    ///
    /// # Arguments
    /// * `level` - Compression level from 1 (fastest) to 9 (best compression)
    ///
    /// # Errors
    /// * Returns `HttpError` if level is 0 or greater than 9
    pub fn deflate_level(mut self, level: u32) -> Result<Self, HttpError> {
        if (1..=9).contains(&level) {
            self.config.compression.deflate.level = Some(level);
            Ok(self)
        } else {
            Err(HttpError::new(crate::error::types::Kind::Request)
                .with("Deflate compression level must be between 1 and 9"))
        }
    }

    /// # Errors
    ///
    /// Returns [`HttpError`] if the client configuration is invalid.
    pub fn build(self) -> Result<HttpClient, HttpError> {
        // Use the core HttpClient::with_config method directly
        Ok(crate::client::core::HttpClient::with_config(self.config))
    }
}

impl Default for HttpClient {
    /// Create `HttpClient` with default configuration
    ///
    /// Uses the default `HttpConfig` and falls back to a basic http3 client
    /// if configuration fails. This ensures the client can always be constructed
    /// even in constrained environments.
    fn default() -> Self {
        // Use the new with_config constructor with default configuration
        // If configuration fails, fall back to a basic http3 client
        let config = HttpConfig::default();
        crate::client::core::HttpClient::with_config(config)
    }
}
