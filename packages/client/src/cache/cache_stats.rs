//! Cache statistics and metrics tracking

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Statistics for HTTP response caching
#[derive(Debug)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: AtomicU64,
    /// Number of cache misses
    pub misses: AtomicU64,
    /// Number of cache evictions
    pub evictions: AtomicU64,
    /// Total bytes stored in cache
    pub bytes_stored: AtomicU64,
    /// Number of entries in cache
    pub entries: AtomicU64,
    /// Number of cache validations
    pub validations: AtomicU64,
    /// Cache creation time
    pub created_at: Instant,
}

impl CacheStats {
    /// Create new cache statistics
    #[must_use] 
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            bytes_stored: AtomicU64::new(0),
            entries: AtomicU64::new(0),
            validations: AtomicU64::new(0),
            created_at: Instant::now(),
        }
    }

    /// Record a cache hit
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache eviction
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Update bytes stored
    pub fn update_bytes_stored(&self, bytes: u64) {
        self.bytes_stored.store(bytes, Ordering::Relaxed);
    }

    /// Update entry count
    pub fn update_entries(&self, count: u64) {
        self.entries.store(count, Ordering::Relaxed);
    }

    /// Get hit ratio
    #[allow(clippy::cast_precision_loss)]
    pub fn hit_ratio(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            // Use safe precision-aware hit rate calculation
            if hits > (1u64 << 53) || total > (1u64 << 53) {
                // For very large cache stats, use high-precision integer calculation
                let hit_rate_scaled = (u128::from(hits) * 1_000_000_000) / u128::from(total);
                (hit_rate_scaled as f64) / 1_000_000_000.0
            } else {
                // Safe to use f64 for smaller cache stats
                (hits as f64) / (total as f64)
            }
        }
    }

    /// Get cache age
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            bytes_stored: AtomicU64::new(0),
            entries: AtomicU64::new(0),
            validations: AtomicU64::new(0),
            created_at: Instant::now(),
        }
    }
}
