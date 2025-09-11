//! Cache performance statistics and monitoring
//!
//! Provides `CacheStats` for tracking cache hits, misses, evictions,
//! and other performance metrics with atomic counters.

use std::sync::atomic::{AtomicU64, Ordering};

/// Cache statistics for monitoring
#[derive(Debug)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: AtomicU64,
    /// Number of cache misses
    pub misses: AtomicU64,
    /// Number of cache evictions
    pub evictions: AtomicU64,
    /// Number of cache validations
    pub validations: AtomicU64,
    /// Number of cache errors
    pub errors: AtomicU64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            validations: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
}

impl CacheStats {
    /// Get hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        // Precision loss acceptable for cache hit rate statistics
        #[allow(clippy::cast_precision_loss)]
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        #[allow(clippy::cast_precision_loss)]
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;

        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }

    /// Get statistics snapshot
    pub fn snapshot(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
            self.evictions.load(Ordering::Relaxed),
            self.validations.load(Ordering::Relaxed),
            self.errors.load(Ordering::Relaxed),
        )
    }
}
