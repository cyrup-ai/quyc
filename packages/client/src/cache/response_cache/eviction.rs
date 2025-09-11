//! Cache eviction and cleanup operations
//!
//! LRU eviction, expired entry cleanup, and memory management operations
//! using lock-free patterns for concurrent cache maintenance.

use std::{sync::atomic::Ordering, time::Instant};

use super::core::ResponseCache;

impl ResponseCache {
    /// Evict least recently used entries to free space
    /// Returns the number of entries actually evicted
    pub(super) fn evict_lru_entries(&self) -> u32 {
        let mut candidates: Vec<(String, Instant, u64)> = Vec::new();

        // Collect candidates for eviction (key, last_accessed, size)
        for entry_ref in &self.entries {
            let key = entry_ref.key().clone();
            let entry = entry_ref.value();
            candidates.push((key, entry.last_accessed, entry.size_bytes));
        }

        // Sort by last accessed (oldest first)
        candidates.sort_by_key(|(_, last_accessed, _)| *last_accessed);

        // Evict oldest 25% of entries or until under limits
        let target_evictions = (candidates.len() / 4).max(1);
        let mut evicted_count = 0;

        for (key, _, size) in candidates.iter().take(target_evictions) {
            if self.entries.remove(key).is_some() {
                self.entry_count.fetch_sub(1, Ordering::Relaxed);
                self.memory_usage.fetch_sub(*size, Ordering::Relaxed);
                self.stats.evictions.fetch_add(1, Ordering::Relaxed);
                evicted_count += 1;

                // Stop if under limits
                let current_memory = self.memory_usage.load(Ordering::Relaxed);
                let entry_count_raw = self.entry_count.load(Ordering::Relaxed);
                let current_entries = usize::try_from(entry_count_raw).unwrap_or_else(|_| {
                    // If count exceeds usize::MAX, we definitely need eviction
                    tracing::warn!("Entry count {} exceeds usize::MAX during eviction", entry_count_raw);
                    usize::MAX
                });

                if current_memory < self.config.max_memory_bytes
                    && current_entries < self.config.max_entries
                {
                    break;
                }
            }
        }

        evicted_count
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        if self
            .cleanup_running
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err()
        {
            return; // Cleanup already running
        }

        let mut expired_keys = Vec::new();

        for entry_ref in &self.entries {
            if entry_ref.value().is_expired() {
                expired_keys.push(entry_ref.key().clone());
            }
        }

        for key in expired_keys {
            if let Some(entry_ref) = self.entries.get(&key) {
                let size_bytes = entry_ref.value().size_bytes;
                self.entries.remove(&key);
                self.entry_count.fetch_sub(1, Ordering::Relaxed);
                self.memory_usage.fetch_sub(size_bytes, Ordering::Relaxed);
            }
        }

        self.cleanup_running.store(false, Ordering::Release);
    }
}


