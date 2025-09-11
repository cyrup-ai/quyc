//! QUIC timeout and window configuration methods
//!
//! Provides builder methods for configuring QUIC connection timeouts and flow control windows.

use std::time::Duration;

use super::super::core::HttpConfig;

impl HttpConfig {
    /// Set QUIC connection maximum idle timeout
    ///
    /// Controls how long a QUIC connection can remain idle before being closed.
    /// This affects HTTP/3 connection lifecycle and resource usage.
    ///
    /// # Arguments
    /// * `timeout` - Maximum idle time before connection closure
    ///
    /// # Examples
    /// ```no_run
    /// use std::time::Duration;
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_quic_max_idle_timeout(Duration::from_secs(60));
    /// assert_eq!(config.quic_max_idle_timeout, Some(Duration::from_secs(60)));
    /// ```
    #[must_use] 
    pub fn with_quic_max_idle_timeout(mut self, timeout: Duration) -> Self {
        self.quic_max_idle_timeout = Some(timeout);
        self
    }

    /// Set QUIC per-stream receive window size
    ///
    /// Controls flow control for individual HTTP/3 streams. Larger windows
    /// allow more data buffering but use more memory per stream.
    ///
    /// # Arguments
    /// * `window_size` - Receive window size in bytes
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_quic_stream_receive_window(512 * 1024); // 512KB
    /// assert_eq!(config.quic_stream_receive_window, Some(512 * 1024));
    /// ```
    #[must_use] 
    pub fn with_quic_stream_receive_window(mut self, window_size: u32) -> Self {
        self.quic_stream_receive_window = Some(window_size);
        self
    }

    /// Set QUIC connection-wide receive window size
    ///
    /// Controls aggregate flow control across all streams in a connection.
    /// This limits total buffered data for the entire connection.
    ///
    /// # Arguments
    /// * `window_size` - Connection receive window size in bytes
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_quic_receive_window(2 * 1024 * 1024); // 2MB
    /// assert_eq!(config.quic_receive_window, Some(2 * 1024 * 1024));
    /// ```
    #[must_use] 
    pub fn with_quic_receive_window(mut self, window_size: u32) -> Self {
        self.quic_receive_window = Some(window_size);
        self
    }

    /// Set QUIC send window size
    ///
    /// Controls how much data can be sent without acknowledgment. Larger
    /// windows improve throughput over high-latency networks.
    ///
    /// # Arguments
    /// * `window_size` - Send window size in bytes
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::config::HttpConfig;
    ///
    /// let config = HttpConfig::default()
    ///     .with_quic_send_window(1024 * 1024); // 1MB
    /// assert_eq!(config.quic_send_window, Some(1024 * 1024));
    /// ```
    #[must_use] 
    pub fn with_quic_send_window(mut self, window_size: u64) -> Self {
        self.quic_send_window = Some(window_size);
        self
    }
}
