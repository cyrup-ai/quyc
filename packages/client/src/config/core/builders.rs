//! Builder methods for HTTP configuration
//!
//! This module provides fluent builder methods for configuring HttpConfig
//! instances with common settings and optimizations.

use super::types::HttpConfig;

impl HttpConfig {
    /// Enable or disable HTTP/3 (QUIC) support
    ///
    /// Controls whether the client will attempt to use HTTP/3 over QUIC
    /// when available. HTTP/3 can provide better performance and features
    /// compared to HTTP/2, but may have compatibility considerations.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable HTTP/3 support
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_http3(true);
    /// assert!(config.http3_enabled);
    /// ```
    pub fn with_http3(mut self, enabled: bool) -> Self {
        self.http3_enabled = enabled;
        self
    }

    /// Enable or disable compression algorithms
    ///
    /// Controls support for gzip, brotli, and deflate compression.
    /// Compression reduces bandwidth usage but increases CPU overhead.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable compression support
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_compression(true);
    /// assert!(config.gzip && config.brotli && config.deflate);
    /// ```
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.gzip = enabled;
        self.brotli = enabled;
        self.deflate = enabled;
        self
    }

    /// Enable or disable metrics collection
    ///
    /// Controls whether the client collects performance and usage metrics.
    /// Metrics can be useful for monitoring and debugging but add overhead.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable metrics collection
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_metrics(true);
    /// assert!(config.metrics_enabled);
    /// ```
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.metrics_enabled = enabled;
        self
    }

    /// Enable or disable tracing
    ///
    /// Controls whether the client produces detailed trace information
    /// for debugging and observability. Tracing provides detailed insights
    /// but can significantly impact performance.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable tracing
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_tracing(true);
    /// assert!(config.tracing_enabled);
    /// ```
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.tracing_enabled = enabled;
        self
    }
}
