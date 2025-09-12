//! Keep-alive configuration methods
//!
//! Provides builder methods for configuring TCP and HTTP/2 keep-alive settings.

use std::time::Duration;

use super::super::HttpConfig;

impl HttpConfig {
    /// Set TCP keep-alive duration
    ///
    /// Enables TCP keep-alive with the specified interval. This helps detect
    /// dead connections and maintain long-lived connections through NATs/firewalls.
    ///
    /// # Arguments
    /// * `duration` - Keep-alive interval, or None to disable
    ///
    /// # Examples
    /// ```no_run
    /// use std::time::Duration;
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_tcp_keepalive(Some(Duration::from_secs(30)));
    /// assert_eq!(config.tcp.keepalive, Some(Duration::from_secs(30)));
    /// ```
    #[must_use] 
    pub fn with_tcp_keepalive(mut self, duration: Option<Duration>) -> Self {
        self.tcp.keepalive = duration;
        self
    }

    /// Set HTTP/2 keep-alive interval
    ///
    /// Controls how frequently HTTP/2 PING frames are sent to keep connections alive.
    /// More frequent pings improve connection reliability but increase overhead.
    ///
    /// # Arguments
    /// * `interval` - PING frame interval, or None to disable
    ///
    /// # Examples
    /// ```no_run
    /// use std::time::Duration;
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_http2_keep_alive_interval(Some(Duration::from_secs(20)));
    /// assert_eq!(config.http2_keep_alive_interval, Some(Duration::from_secs(20)));
    /// ```
    #[must_use] 
    pub fn with_http2_keep_alive_interval(mut self, interval: Option<Duration>) -> Self {
        self.http2_keep_alive_interval = interval;
        self
    }

    /// Set HTTP/2 keep-alive timeout
    ///
    /// Controls how long to wait for PING frame responses before considering
    /// the connection dead.
    ///
    /// # Arguments
    /// * `timeout` - PING response timeout, or None for default
    ///
    /// # Examples
    /// ```no_run
    /// use std::time::Duration;
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_http2_keep_alive_timeout(Some(Duration::from_secs(10)));
    /// assert_eq!(config.http2_keep_alive_timeout, Some(Duration::from_secs(10)));
    /// ```
    #[must_use] 
    pub fn with_http2_keep_alive_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.http2_keep_alive_timeout = timeout;
        self
    }
}
