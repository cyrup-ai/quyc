//! Rate limiting implementation for DNS resolver
//!
//! Uses sliding window algorithm with `DashMap` for thread-safe rate limiting

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
    #[must_use] 
    pub fn new(max_requests_per_second: u32) -> Self {
        Self {
            limits: Arc::new(DashMap::new()),
            max_requests: max_requests_per_second,
            window: Duration::from_secs(1),
            enabled: true,
        }
    }
    
    #[must_use] 
    pub fn with_window(mut self, window: Duration) -> Self {
        self.window = window;
        self
    }
    
    /// Check if DNS query is within rate limits
    ///
    /// # Errors
    /// 
    /// Returns `String` error if:
    /// - Query rate exceeds configured limits for the `hostname/query_type` combination
    /// - Rate limiter window calculation fails due to time arithmetic errors
    /// - Internal rate tracking data structures encounter consistency issues
    pub fn check_rate_limit(&self, hostname: &str, query_type: &str) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        
        let key = format!("{hostname}:{query_type}");
        let now = Instant::now();
        let window_start = now.checked_sub(self.window)
            .ok_or_else(|| format!("Rate limiter window duration ({:?}) exceeds current time", self.window))?;
        
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
                "Rate limit exceeded for {key}. Retry after {retry_after:?}"
            ));
        }
        
        // Record this request
        state.timestamps.push(now);
        state.total_requests.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    pub fn reset(&self, hostname: &str, query_type: &str) {
        let key = format!("{hostname}:{query_type}");
        self.limits.remove(&key);
    }
    
    pub fn reset_all(&self) {
        self.limits.clear();
    }
    
    #[must_use] 
    pub fn get_metrics(&self, hostname: &str, query_type: &str) -> Option<RateLimitMetrics> {
        let key = format!("{hostname}:{query_type}");
        self.limits.get(&key).map(|entry| {
            let state = entry.value();
            let current_requests = u32::try_from(state.timestamps.len()).unwrap_or_else(|_| {
                tracing::warn!(
                    target: "quyc::rate_limiter",
                    timestamps_len = state.timestamps.len(),
                    max_u32 = u32::MAX,
                    "Timestamp count exceeds u32 limits, clamping to max"
                );
                u32::MAX
            });
            RateLimitMetrics {
                current_requests,
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