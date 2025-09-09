//! DNS resolver configuration types
//!
//! This module provides configuration structures for DNS caching and retry behavior.

use std::time::Duration;

/// DNS cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_secs: u64,
    pub max_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_secs: 300, // 5 minutes default TTL
            max_entries: 1000,
        }
    }
}

/// DNS retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub backoff_multiplier: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 2,
        }
    }
}

impl CacheConfig {
    /// Create a new cache configuration
    pub fn new(enabled: bool, ttl_secs: u64, max_entries: usize) -> Self {
        Self {
            enabled,
            ttl_secs,
            max_entries,
        }
    }

    /// Create a disabled cache configuration
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ttl_secs: 0,
            max_entries: 0,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_retries: u32, initial_delay: Duration, backoff_multiplier: u32) -> Self {
        Self {
            max_retries,
            initial_delay,
            backoff_multiplier,
        }
    }

    /// Create a no-retry configuration
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            initial_delay: Duration::from_millis(0),
            backoff_multiplier: 1,
        }
    }
}
