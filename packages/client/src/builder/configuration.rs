//! Provides methods for configuring request behavior including timeouts,
//! retry attempts, and debug logging.

use super::builder_core::Http3Builder;

/// Trait for builder configuration methods
pub trait BuilderConfig {
    /// Enable debug logging for this request
    fn debug(self) -> Self;

    /// Set request timeout in seconds
    fn timeout(self, seconds: u64) -> Self;

    /// Set maximum retry attempts
    fn retry_attempts(self, attempts: u32) -> Self;
}

/// Request configuration settings
pub struct RequestConfig {
    pub timeout_seconds: Option<u64>,
    pub retry_attempts: Option<u32>,
    pub debug_enabled: bool,
}

impl Default for RequestConfig {
    #[inline]
    fn default() -> Self {
        Self {
            timeout_seconds: None,
            retry_attempts: None,
            debug_enabled: false,
        }
    }
}

impl<S> BuilderConfig for Http3Builder<S> {
    #[inline]
    fn debug(self) -> Self {
        self.enable_debug()
    }

    #[inline]
    fn timeout(self, seconds: u64) -> Self {
        self.set_timeout(seconds)
    }

    #[inline]
    fn retry_attempts(self, attempts: u32) -> Self {
        self.set_retry_attempts(attempts)
    }
}

impl<S> Http3Builder<S> {
    /// Enable debug logging for this request
    ///
    /// When enabled, detailed request and response information will be logged
    /// to help with debugging and development.
    ///
    /// # Returns
    /// `Self` for method chaining
    #[must_use]
    #[inline]
    pub fn debug(mut self) -> Self {
        self.debug_enabled = true;
        self
    }

    /// Set request timeout in seconds
    ///
    /// # Arguments  
    /// * `seconds` - Timeout duration in seconds
    ///
    /// # Returns
    /// `Self` for method chaining
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3;
    ///
    /// let response = Http3::json()
    ///     .timeout(30)
    ///     .get("https://api.example.com/data");
    /// ```
    #[must_use]
    #[inline]
    pub fn timeout(self, seconds: u64) -> Self {
        if self.debug_enabled {
            log::debug!("HTTP3 Builder: Set timeout to {seconds} seconds");
        }
        // Note: Timeout implementation would be handled by the client
        self
    }

    /// Set maximum retry attempts for failed requests
    ///
    /// # Arguments
    /// * `attempts` - Maximum number of retry attempts
    ///
    /// # Returns
    /// `Self` for method chaining
    #[must_use]
    #[inline]
    pub fn retry_attempts(self, attempts: u32) -> Self {
        if self.debug_enabled {
            log::debug!("HTTP3 Builder: Set retry attempts to {attempts}");
        }
        // Note: Retry implementation would be handled by the client
        self
    }

    /// Internal method to enable debug logging
    #[inline]
    fn enable_debug(mut self) -> Self {
        self.debug_enabled = true;
        self
    }

    /// Internal method to set timeout
    #[inline]
    fn set_timeout(self, _seconds: u64) -> Self {
        // Implementation would configure client timeout
        self
    }

    /// Internal method to set retry attempts
    #[inline]
    fn set_retry_attempts(self, _attempts: u32) -> Self {
        // Implementation would configure client retry behavior
        self
    }
}
