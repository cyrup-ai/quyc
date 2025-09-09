//! Zero allocation HTTP response caching with lock-free skiplist storage
//!
//! This module provides a high-performance, concurrent HTTP response cache using:
//! - Lock-free SkipMap for O(log n) concurrent operations
//! - Atomic counters for statistics and cache management
//! - Blake3 hashing for fast cache key generation
//! - TTL-based expiration with microsecond precision
//! - LRU eviction using atomic access counters
//! - Zero-allocation cache hits through Arc<T> sharing

pub mod cache_config;
pub mod cache_entry;
pub mod cache_integration;
pub mod cache_key;
pub mod cache_stats;
pub mod http_date;
pub mod response_cache;

// Re-export all public types and functions
pub use cache_config::CacheConfig;
pub use cache_entry::CacheEntry;
pub use cache_integration::{GLOBAL_CACHE, cached_stream, conditional_headers_for_key};
pub use cache_key::CacheKey;
pub use http_date::{HttpDateParseError, httpdate};
pub use response_cache::ResponseCache;
