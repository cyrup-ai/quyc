//! Retry Configuration Module
//!
//! Configurable retry logic with exponential backoff and circuit breaker patterns.

use std::time::Duration;

/// Retry configuration provider trait
pub trait RetryConfigProvider {
    fn max_retries(&self) -> u32;
    fn initial_backoff(&self) -> Duration;
    fn max_backoff(&self) -> Duration;
    fn backoff_multiplier(&self) -> f64;
    fn jitter_enabled(&self) -> bool;
    fn circuit_breaker_enabled(&self) -> bool;
    fn circuit_breaker_threshold(&self) -> u32;
}

/// Runtime retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
    pub jitter_enabled: bool,
    pub circuit_breaker_enabled: bool,
    pub circuit_breaker_threshold: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_enabled: true,
            circuit_breaker_enabled: false,
            circuit_breaker_threshold: 5,
        }
    }
}

impl RetryConfig {
    /// Create aggressive retry configuration for critical requests
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            max_retries: 5,
            initial_backoff: Duration::from_millis(50),
            max_backoff: Duration::from_secs(10),
            backoff_multiplier: 1.5,
            circuit_breaker_enabled: true,
            circuit_breaker_threshold: 3,
            ..Self::default()
        }
    }
    
    /// Create conservative retry configuration
    #[must_use]
    pub fn conservative() -> Self {
        Self {
            max_retries: 1,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 3.0,
            jitter_enabled: false,
            ..Self::default()
        }
    }
    
    /// Validate retry configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - `backoff_multiplier` is less than 1.0
    /// - `initial_backoff` exceeds `max_backoff`
    /// - Circuit breaker threshold is invalid
    /// - Retry parameters are out of acceptable ranges
    pub fn validate(&self) -> Result<(), String> {
        if self.backoff_multiplier < 1.0 {
            return Err("backoff_multiplier must be >= 1.0".to_string());
        }
        
        if self.initial_backoff > self.max_backoff {
            return Err("initial_backoff cannot exceed max_backoff".to_string());
        }
        
        if self.circuit_breaker_threshold == 0 {
            return Err("circuit_breaker_threshold must be greater than 0".to_string());
        }
        
        Ok(())
    }
}

impl RetryConfigProvider for RetryConfig {
    #[inline]
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
    
    #[inline]
    fn initial_backoff(&self) -> Duration {
        self.initial_backoff
    }
    
    #[inline]
    fn max_backoff(&self) -> Duration {
        self.max_backoff
    }
    
    #[inline]
    fn backoff_multiplier(&self) -> f64 {
        self.backoff_multiplier
    }
    
    #[inline]
    fn jitter_enabled(&self) -> bool {
        self.jitter_enabled
    }
    
    #[inline]
    fn circuit_breaker_enabled(&self) -> bool {
        self.circuit_breaker_enabled
    }
    
    #[inline]
    fn circuit_breaker_threshold(&self) -> u32 {
        self.circuit_breaker_threshold
    }
}

/// Compile-time retry configuration
pub struct StaticRetryConfig<const MAX_RETRIES: u32 = 3>;

impl<const MAX_RETRIES: u32> RetryConfigProvider for StaticRetryConfig<MAX_RETRIES> {
    #[inline]
    fn max_retries(&self) -> u32 {
        MAX_RETRIES
    }
    
    #[inline]
    fn initial_backoff(&self) -> Duration {
        Duration::from_millis(100)
    }
    
    #[inline]
    fn max_backoff(&self) -> Duration {
        Duration::from_secs(30)
    }
    
    #[inline]
    fn backoff_multiplier(&self) -> f64 {
        2.0
    }
    
    #[inline]
    fn jitter_enabled(&self) -> bool {
        true
    }
    
    #[inline]
    fn circuit_breaker_enabled(&self) -> bool {
        false
    }
    
    #[inline]
    fn circuit_breaker_threshold(&self) -> u32 {
        5
    }
}