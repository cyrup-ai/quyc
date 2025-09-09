//! Cache key generation and hashing for HTTP responses
//!
//! Provides CacheKey struct for generating consistent cache keys based on
//! request URL, method, and cache-relevant headers.

use std::collections::HashMap;

/// Cache key for HTTP responses based on URL and headers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKey {
    /// Request URL
    pub url: String,
    /// Normalized headers that affect caching (e.g., Accept, Authorization)
    pub cache_headers: HashMap<String, String>,
    /// HTTP method (GET, POST, etc.)
    pub method: String,
}

impl CacheKey {
    /// Create cache key from request components
    pub fn new(url: String, method: String, headers: HashMap<String, String>) -> Self {
        // Only include headers that affect caching behavior
        let cache_headers = headers
            .into_iter()
            .filter(|(key, _)| {
                let key_lower = key.to_lowercase();
                matches!(
                    key_lower.as_str(),
                    "accept"
                        | "accept-encoding"
                        | "accept-language"
                        | "authorization"
                        | "cache-control"
                        | "if-none-match"
                        | "if-modified-since"
                        | "user-agent"
                )
            })
            .collect();

        Self {
            url,
            method,
            cache_headers,
        }
    }

    /// Generate hash key for storage
    pub fn hash_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl std::hash::Hash for CacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.method.hash(state);

        // Sort headers for consistent hashing
        let mut sorted_headers: Vec<_> = self.cache_headers.iter().collect();
        sorted_headers.sort_by_key(|(key, _)| *key);

        for (key, value) in sorted_headers {
            key.hash(state);
            value.hash(state);
        }
    }
}
