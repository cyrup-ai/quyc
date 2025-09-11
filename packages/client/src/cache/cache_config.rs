//! Cache configuration and preset policies
//!
//! Provides `CacheConfig` for configuring cache behavior including
//! memory limits, TTL settings, and cleanup intervals.

use std::time::Duration;

/// Cache configuration and limits
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in cache
    pub max_entries: usize,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Default TTL for entries without explicit expiration
    pub default_ttl: Duration,
    /// Enable automatic cleanup of expired entries
    pub auto_cleanup: bool,
    /// Cleanup interval
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            max_memory_bytes: 100 * 1024 * 1024,   // 100MB
            default_ttl: Duration::from_secs(300), // 5 minutes
            auto_cleanup: true,
            cleanup_interval: Duration::from_secs(60), // 1 minute
        }
    }
}

impl CacheConfig {
    /// Create aggressive caching configuration
    #[must_use] 
    pub fn aggressive() -> Self {
        Self {
            max_entries: 5000,
            max_memory_bytes: 500 * 1024 * 1024,    // 500MB
            default_ttl: Duration::from_secs(3600), // 1 hour
            auto_cleanup: true,
            cleanup_interval: Duration::from_secs(30),
        }
    }

    /// Create conservative caching configuration
    #[must_use] 
    pub fn conservative() -> Self {
        Self {
            max_entries: 200,
            max_memory_bytes: 20 * 1024 * 1024,   // 20MB
            default_ttl: Duration::from_secs(60), // 1 minute
            auto_cleanup: true,
            cleanup_interval: Duration::from_secs(120), // 2 minutes
        }
    }

    /// Create no-cache configuration (disabled caching)
    #[must_use] 
    pub fn no_cache() -> Self {
        Self {
            max_entries: 0,
            max_memory_bytes: 0,
            default_ttl: Duration::ZERO,
            auto_cleanup: false,
            cleanup_interval: Duration::MAX,
        }
    }
}
