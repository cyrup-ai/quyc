//! Core ResponseCache structure and initialization
//!
//! Provides the main ResponseCache struct with lock-free storage using crossbeam SkipMap
//! and atomic counters for concurrent operations.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crossbeam_skiplist::SkipMap;

use super::super::{cache_config::CacheConfig, cache_entry::CacheEntry, cache_stats::CacheStats};

/// Lock-free HTTP response cache using crossbeam skiplist
pub struct ResponseCache {
    /// Main cache storage (key -> entry)
    pub(super) entries: SkipMap<String, CacheEntry>,
    /// Configuration
    pub(super) config: CacheConfig,
    /// Current memory usage estimate
    pub(super) memory_usage: AtomicU64,
    /// Entry count
    pub(super) entry_count: AtomicU64,
    /// Cache statistics
    pub(super) stats: CacheStats,
    /// Cleanup task running flag
    pub(super) cleanup_running: AtomicBool,
}

impl ResponseCache {
    /// Create new response cache with configuration
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: SkipMap::new(),
            config,
            memory_usage: AtomicU64::new(0),
            entry_count: AtomicU64::new(0),
            stats: CacheStats::default(),
            cleanup_running: AtomicBool::new(false),
        }
    }

    /// Create cache with default configuration
    pub fn default() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get current cache size information
    pub fn size_info(&self) -> (usize, u64, f64) {
        let entries = self.entry_count.load(Ordering::Relaxed) as usize;
        let memory = self.memory_usage.load(Ordering::Relaxed);
        let memory_pct = (memory as f64 / self.config.max_memory_bytes as f64) * 100.0;

        (entries, memory, memory_pct)
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        self.entries.clear();
        self.entry_count.store(0, Ordering::Relaxed);
        self.memory_usage.store(0, Ordering::Relaxed);
    }
}
