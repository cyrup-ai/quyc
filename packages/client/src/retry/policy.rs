//! Retry policy configuration with exponential backoff and jitter
//!
//! Provides comprehensive retry policy configuration including timing,
//! error classification, and delay calculation with zero-allocation design.

use std::time::Duration;

use fastrand::Rng;

// prelude import removed - not used
use crate::error::types::Error as HttpError;

/// Retry policy configuration - all durations in milliseconds for zero allocation
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay in milliseconds before first retry
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds between retries
    pub max_delay_ms: u64,
    /// Backoff multiplier (typically 2.0 for exponential backoff)
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0 to 1.0) to prevent thundering herd
    pub jitter_factor: f64,
    /// Timeout per individual attempt in milliseconds
    pub attempt_timeout_ms: u64,
}

impl Default for RetryPolicy {
    /// Create default retry policy with balanced configuration
    ///
    /// Provides reasonable defaults suitable for most HTTP operations
    /// with moderate retry behavior and exponential backoff.
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000, // 1 second
            max_delay_ms: 30000,    // 30 seconds
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            attempt_timeout_ms: 30000, // 30 seconds per attempt
        }
    }
}

impl RetryPolicy {
    /// Create aggressive retry policy for critical operations
    ///
    /// Uses faster retry cycles with more attempts for operations
    /// that must succeed and can tolerate increased retry overhead.
    #[inline]
    #[must_use] 
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 100, // 100ms
            max_delay_ms: 10000,   // 10 seconds
            backoff_multiplier: 1.5,
            jitter_factor: 0.2,
            attempt_timeout_ms: 15000, // 15 seconds per attempt
        }
    }

    /// Create conservative retry policy for non-critical operations
    ///
    /// Uses longer delays and fewer attempts to minimize resource
    /// consumption for operations that can tolerate failure.
    #[inline]
    #[must_use] 
    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            initial_delay_ms: 2000, // 2 seconds
            max_delay_ms: 60000,    // 60 seconds
            backoff_multiplier: 3.0,
            jitter_factor: 0.05,
            attempt_timeout_ms: 60000, // 60 seconds per attempt
        }
    }

    /// Create no-retry policy (single attempt only)
    ///
    /// Disables retry logic entirely for operations that should
    /// fail fast without consuming additional resources.
    #[inline]
    #[must_use] 
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            initial_delay_ms: 0,
            max_delay_ms: 0,
            backoff_multiplier: 1.0,
            jitter_factor: 0.0,
            attempt_timeout_ms: 120_000, // 2 minutes for single attempt
        }
    }

    /// Calculate delay for specific attempt with exponential backoff and jitter
    ///
    /// Implements sophisticated delay calculation combining exponential backoff
    /// with random jitter to prevent thundering herd effects. Uses zero-allocation
    /// calculations for maximum performance.
    #[inline]
    #[must_use] 
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }

        // Calculate exponential backoff delay with safe precision handling
        let base_delay = if self.initial_delay_ms > (1u64 << 53) || self.max_delay_ms > (1u64 << 53) {
            // For very large delay values, use integer arithmetic to avoid precision loss
            let max_safe_multiplier = ((self.max_delay_ms / self.initial_delay_ms.max(1)) as f64).ln() / self.backoff_multiplier.ln();
            let safe_attempt = f64::from(attempt - 1).min(max_safe_multiplier);
            
            // Use integer calculation for large values
            let multiplier_times = self.backoff_multiplier.powf(safe_attempt);
            let delay_calc = (self.initial_delay_ms as f64) * multiplier_times;
            delay_calc.min(self.max_delay_ms as f64)
        } else {
            // Safe to use f64 for smaller delay values
            let base_calc = (self.initial_delay_ms as f64) * self.backoff_multiplier.powi(i32::try_from(attempt - 1).unwrap_or(i32::MAX));
            base_calc.min(self.max_delay_ms as f64)
        };
        
        let capped_delay = base_delay;

        // Add jitter to prevent thundering herd
        let jitter_range = capped_delay * self.jitter_factor;
        let mut rng = Rng::new();
        let jitter = rng.f64() * jitter_range - (jitter_range / 2.0);

        let final_delay = (capped_delay + jitter).max(0.0);
        Duration::from_millis(final_delay as u64)
    }

    /// Check if error is retryable based on error type and status codes
    ///
    /// Implements comprehensive error classification to determine whether
    /// an error should trigger a retry attempt. Network errors and server
    /// errors (5xx) are generally retryable, while client errors (4xx) are not.
    #[inline]
    #[must_use] 
    pub fn is_retryable_error(&self, error: &HttpError) -> bool {
        match &error.inner.kind {
            crate::error::types::Kind::Request |      // Request errors may be transient
            crate::error::types::Kind::Connect |      // Connection failures are retryable
            crate::error::types::Kind::Timeout |      // Timeout errors are retryable
            crate::error::types::Kind::Stream => true, // Stream errors may be retryable
            crate::error::types::Kind::Status(status, _) => {
                // Retry on server errors (5xx) and some client errors (429)
                status.as_u16() >= 500 || status.as_u16() == 429
            }
            // All other error types are not retryable
            crate::error::types::Kind::Builder 
            | crate::error::types::Kind::Redirect 
            | crate::error::types::Kind::Body 
            | crate::error::types::Kind::Decode 
            | crate::error::types::Kind::Upgrade 
            | crate::error::types::Kind::PayloadTooLarge => false,
        }
    }

    /// Validate policy configuration for consistency
    ///
    /// Ensures policy parameters are within reasonable bounds and
    /// consistent with each other to prevent configuration errors.
    #[inline]
    pub fn validate(&self) -> Result<(), String> {
        if self.max_attempts == 0 {
            return Err("max_attempts must be at least 1".to_string());
        }

        if self.backoff_multiplier <= 0.0 {
            return Err("backoff_multiplier must be positive".to_string());
        }

        if self.jitter_factor < 0.0 || self.jitter_factor > 1.0 {
            return Err("jitter_factor must be between 0.0 and 1.0".to_string());
        }

        if self.initial_delay_ms > self.max_delay_ms {
            return Err("initial_delay_ms cannot exceed max_delay_ms".to_string());
        }

        Ok(())
    }
}
