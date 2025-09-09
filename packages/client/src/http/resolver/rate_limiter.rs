//! Rate limiting implementation for DNS resolver
//!
//! Uses sliding window algorithm with DashMap for thread-safe rate limiting

use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};

/// Rate limiter using sliding window algorithm
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Map of key -> rate limit state
    limits: Arc<DashMap<String, RateLimitState>>,
    /// Maximum requests per window
    max_requests: u32,
    /// Window duration
    window: Duration,
    /// Whether rate limiting is enabled
    enabled: bool,
}

#[derive(Debug)]
struct RateLimitState {
    /// Request timestamps in current window
    timestamps: Vec<Instant>,
    /// Total request count (for metrics)
    total_requests: AtomicU32,
}

impl RateLimiter {
    pub fn new(max_requests_per_second: u32) -> Self {
        Self {
            limits: Arc::new(DashMap::new()),
            max_requests: max_requests_per_second,
            window: Duration::from_secs(1),
            enabled: true,
        }
    }
    
    pub fn with_window(mut self, window: Duration) -> Self {
        self.window = window;
        self
    }
    
    pub fn check_rate_limit(&self, hostname: &str, query_type: &str) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        
        let key = format!("{}:{}", hostname, query_type);
        let now = Instant::now();
        let window_start = now - self.window;
        
        // Get or create rate limit state
        let mut entry = self.limits.entry(key.clone()).or_insert_with(|| {
            RateLimitState {
                timestamps: Vec::with_capacity(self.max_requests as usize),
                total_requests: AtomicU32::new(0),
            }
        });
        
        let state = entry.value_mut();
        
        // Remove timestamps outside the window
        state.timestamps.retain(|&ts| ts > window_start);
        
        // Check if limit exceeded
        if state.timestamps.len() >= self.max_requests as usize {
            let retry_after = state.timestamps[0] + self.window - now;
            return Err(format!(
                "Rate limit exceeded for {}. Retry after {:?}",
                key, retry_after
            ));
        }
        
        // Record this request
        state.timestamps.push(now);
        state.total_requests.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    pub fn reset(&self, hostname: &str, query_type: &str) {
        let key = format!("{}:{}", hostname, query_type);
        self.limits.remove(&key);
    }
    
    pub fn reset_all(&self) {
        self.limits.clear();
    }
    
    pub fn get_metrics(&self, hostname: &str, query_type: &str) -> Option<RateLimitMetrics> {
        let key = format!("{}:{}", hostname, query_type);
        self.limits.get(&key).map(|entry| {
            let state = entry.value();
            RateLimitMetrics {
                current_requests: state.timestamps.len() as u32,
                total_requests: state.total_requests.load(Ordering::Relaxed),
                max_requests: self.max_requests,
                window: self.window,
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitMetrics {
    pub current_requests: u32,
    pub total_requests: u32,
    pub max_requests: u32,
    pub window: Duration,
}