//! Basic timeout configuration structures

use std::time::Duration;

/// Timeout configuration provider trait
pub trait TimeoutConfigProvider {
    fn request_timeout(&self) -> Duration;
    fn connect_timeout(&self) -> Duration;
    fn dns_timeout(&self) -> Duration;
    fn idle_timeout(&self) -> Duration;
    fn keepalive_timeout(&self) -> Option<Duration>;
}

/// Runtime timeout configuration
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub request_timeout: Duration,
    pub connect_timeout: Duration,
    pub dns_timeout: Duration,
    pub idle_timeout: Duration,
    pub keepalive_timeout: Option<Duration>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            dns_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            keepalive_timeout: Some(Duration::from_secs(60)),
        }
    }
}

impl TimeoutConfig {
    /// Create aggressive timeout configuration
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            request_timeout: Duration::from_secs(10),
            connect_timeout: Duration::from_secs(3),
            dns_timeout: Duration::from_secs(2),
            idle_timeout: Duration::from_secs(30),
            keepalive_timeout: Some(Duration::from_secs(15)),
        }
    }
    
    /// Validate timeout configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - `request_timeout` is zero
    /// - `connect_timeout` is zero
    /// - Any timeout value is zero when it should be positive
    pub fn validate(&self) -> Result<(), String> {
        if self.request_timeout.is_zero() {
            return Err("request_timeout cannot be zero".to_string());
        }
        
        if self.connect_timeout.is_zero() {
            return Err("connect_timeout cannot be zero".to_string());
        }
        
        Ok(())
    }
}

impl TimeoutConfigProvider for TimeoutConfig {
    #[inline]
    fn request_timeout(&self) -> Duration {
        self.request_timeout
    }
    
    #[inline]
    fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }
    
    #[inline]
    fn dns_timeout(&self) -> Duration {
        self.dns_timeout
    }
    
    #[inline]
    fn idle_timeout(&self) -> Duration {
        self.idle_timeout
    }
    
    #[inline]
    fn keepalive_timeout(&self) -> Option<Duration> {
        self.keepalive_timeout
    }
}

/// Compile-time timeout configuration
pub struct StaticTimeoutConfig<
    const REQUEST_TIMEOUT_MS: u64 = 30000,
    const CONNECT_TIMEOUT_MS: u64 = 10000,
>;

impl<
    const REQUEST_TIMEOUT_MS: u64,
    const CONNECT_TIMEOUT_MS: u64,
> TimeoutConfigProvider for StaticTimeoutConfig<REQUEST_TIMEOUT_MS, CONNECT_TIMEOUT_MS> {
    #[inline]
    fn request_timeout(&self) -> Duration {
        Duration::from_millis(REQUEST_TIMEOUT_MS)
    }
    
    #[inline]
    fn connect_timeout(&self) -> Duration {
        Duration::from_millis(CONNECT_TIMEOUT_MS)
    }
    
    #[inline]
    fn dns_timeout(&self) -> Duration {
        Duration::from_millis(5000)
    }
    
    #[inline]
    fn idle_timeout(&self) -> Duration {
        Duration::from_millis(300_000)
    }
    
    #[inline]
    fn keepalive_timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(60000))
    }
}