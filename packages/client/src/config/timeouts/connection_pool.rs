//! Connection pool configuration methods
//!
//! Provides builder methods for configuring connection pooling and idle connection management.

use std::time::Duration;

use super::super::core::HttpConfig;

impl HttpConfig {
    /// Set the connection pool size
    ///
    /// Controls the maximum number of active connections maintained in the pool.
    /// Larger pools can handle more concurrent requests but use more resources.
    ///
    /// # Arguments
    /// * `size` - Maximum number of connections in the pool
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_pool_size(50);
    /// assert_eq!(config.pool_size, 50);
    /// ```
    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.pool_size = size;
        self
    }

    /// Set the maximum idle connections per host
    ///
    /// Controls how many idle connections to keep alive for each host to enable
    /// connection reuse. Higher values improve performance for repeated requests
    /// to the same host.
    ///
    /// # Arguments
    /// * `max_idle` - Maximum idle connections per host
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_max_idle_per_host(64);
    /// assert_eq!(config.pool_max_idle_per_host, 64);
    /// ```
    pub fn with_max_idle_per_host(mut self, max_idle: usize) -> Self {
        self.pool_max_idle_per_host = max_idle;
        self
    }

    /// Set the pool idle timeout
    ///
    /// Controls how long idle connections are kept alive before being closed.
    /// Longer timeouts improve connection reuse but consume more resources.
    ///
    /// # Arguments
    /// * `timeout` - Duration to keep idle connections alive
    ///
    /// # Examples
    /// ```no_run
    /// use std::time::Duration;
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_pool_idle_timeout(Duration::from_secs(120));
    /// assert_eq!(config.pool_idle_timeout, Duration::from_secs(120));
    /// ```
    pub fn with_pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.pool_idle_timeout = timeout;
        self
    }
}
