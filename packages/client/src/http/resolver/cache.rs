//! DNS caching implementation with TTL support
//!
//! This module provides high-performance DNS caching with automatic expiration and eviction.

use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use tracing::debug;

use super::config::CacheConfig;

/// DNS cache entry with TTL
#[derive(Debug, Clone)]
pub struct DnsCacheEntry {
    pub addresses: Vec<SocketAddr>,
    expires_at: u64, // Unix timestamp in seconds
}

impl DnsCacheEntry {
    pub fn new(addresses: Vec<SocketAddr>, ttl_secs: u64) -> Self {
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + ttl_secs;

        Self {
            addresses,
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now >= self.expires_at
    }
}

/// High-performance DNS cache with automatic expiration and eviction
#[derive(Debug)]
pub struct DnsCache {
    cache: DashMap<String, DnsCacheEntry>,
    pub config: CacheConfig,
}

impl DnsCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: DashMap::new(),
            config,
        }
    }

    /// Get cache entry if it exists and is not expired
    pub fn get(&self, key: &str) -> Option<DnsCacheEntry> {
        if !self.config.enabled {
            return None;
        }

        if let Some(entry) = self.cache.get(key) {
            if !entry.is_expired() {
                debug!("DNS cache hit for {}", key);
                Some(entry.clone())
            } else {
                // Remove expired entry
                self.cache.remove(key);
                debug!("Removed expired DNS cache entry for {}", key);
                None
            }
        } else {
            None
        }
    }

    /// Insert entry into cache with automatic eviction if needed
    pub fn insert(&self, key: String, entry: DnsCacheEntry) {
        if !self.config.enabled {
            return;
        }

        // Check cache size limit and evict old entries if needed
        if self.cache.len() >= self.config.max_entries {
            self.evict_entries();
        }

        self.cache.insert(key.clone(), entry);
        debug!(
            "Cached DNS result for {} with TTL {}s",
            key, self.config.ttl_secs
        );
    }

    /// Evict old entries when cache is full
    fn evict_entries(&self) {
        // Simple eviction: remove a few random entries
        let to_remove: Vec<String> = self
            .cache
            .iter()
            .take(self.config.max_entries / 10) // Remove 10% when full
            .map(|entry| entry.key().clone())
            .collect();

        for key in to_remove {
            self.cache.remove(&key);
        }

        debug!(
            "Evicted old DNS cache entries, cache size: {}",
            self.cache.len()
        );
    }

    /// Get current cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}
