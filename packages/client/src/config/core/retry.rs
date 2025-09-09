//! Retry policy and connection reuse configuration types
//!
//! Contains enums and structs for configuring retry behavior,
//! connection reuse strategies, and error classification.

use std::time::Duration;

/// Connection reuse strategy
///
/// Defines how aggressively the client should reuse existing connections.
#[derive(Debug, Clone)]
pub enum ConnectionReuse {
    /// Reuse connections aggressively for maximum performance
    Aggressive,
    /// Reuse connections conservatively for better reliability
    Conservative,
    /// Disable connection reuse entirely
    Disabled,
}

/// Retry policy configuration
///
/// Configures automatic retry behavior for failed requests including
/// exponential backoff, jitter, and conditions for retry attempts.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retries
    pub max_retries: usize,

    /// Base delay between retries
    pub base_delay: Duration,

    /// Maximum delay between retries
    pub max_delay: Duration,

    /// Exponential backoff factor
    pub backoff_factor: f64,

    /// Jitter factor (0.0 to 1.0)
    pub jitter_factor: f64,

    /// Retry on specific status codes
    pub retry_on_status: Vec<u16>,

    /// Retry on specific errors
    pub retry_on_errors: Vec<RetryableError>,
}

/// Retryable error types
///
/// Classifies the types of errors that should trigger automatic retry attempts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryableError {
    /// Network connectivity errors
    Network,
    /// Request timeout errors
    Timeout,
    /// Connection establishment errors
    Connection,
    /// DNS resolution errors
    Dns,
    /// TLS handshake errors
    Tls,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
            jitter_factor: 0.1,
            retry_on_status: vec![429, 500, 502, 503, 504],
            retry_on_errors: vec![
                RetryableError::Network,
                RetryableError::Timeout,
                RetryableError::Connection,
                RetryableError::Dns,
            ],
        }
    }
}
