//! Basic timeout configuration methods
//!
//! Provides builder methods for configuring request and connection timeouts.

use std::time::Duration;

use super::super::core::HttpConfig;

impl HttpConfig {
    /// Set the request timeout
    ///
    /// Controls how long to wait for a complete request/response cycle before timing out.
    /// This includes connection establishment, request sending, and response receiving.
    ///
    /// # Arguments
    /// * `timeout` - Maximum duration to wait for request completion
    ///
    /// # Examples
    /// ```no_run
    /// use std::time::Duration;
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_timeout(Duration::from_secs(30));
    /// assert_eq!(config.timeout, Duration::from_secs(30));
    /// ```
    #[must_use] 
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the connection timeout
    ///
    /// Controls how long to wait when establishing initial connections to servers.
    /// This only covers the TCP/QUIC handshake time, not the full request time.
    ///
    /// # Arguments
    /// * `timeout` - Maximum duration to wait for connection establishment
    ///
    /// # Examples
    /// ```no_run
    /// use std::time::Duration;
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_connect_timeout(Duration::from_secs(5));
    /// assert_eq!(config.connect_timeout, Duration::from_secs(5));
    /// ```
    #[must_use] 
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set DNS cache duration
    ///
    /// Controls how long DNS resolution results are cached. Longer caching
    /// improves performance but may delay detection of DNS changes.
    ///
    /// # Arguments
    /// * `duration` - DNS cache lifetime
    ///
    /// # Examples
    /// ```no_run
    /// use std::time::Duration;
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_dns_cache_duration(Duration::from_secs(600)); // 10 minutes
    /// assert_eq!(config.dns_cache_duration, Duration::from_secs(600));
    /// ```
    #[must_use] 
    pub fn with_dns_cache_duration(mut self, duration: Duration) -> Self {
        self.dns_cache_duration = duration;
        self
    }
}
