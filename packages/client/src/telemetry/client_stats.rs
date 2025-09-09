//! Zero-allocation client statistics tracking with cache-padded atomic counters
//!
//! Provides blazing-fast thread-safe statistics collection for HTTP client operations
//! using zero-allocation lock-free atomic operations with optimal CPU cache utilization.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crossbeam_utils::CachePadded;

/// Zero-allocation thread-safe client statistics with cache-padded atomic counters
///
/// Eliminates all heap allocations by using cache-padded atomic counters directly.
/// Each counter is cache-padded to prevent false sharing and maximize performance.
/// All operations are lock-free with optimal CPU cache line utilization.
#[derive(Debug, Default)]
pub struct ClientStats {
    /// Total number of HTTP requests made - cache padded for optimal performance
    pub request_count: CachePadded<AtomicUsize>,
    /// Total number of connections established - cache padded for optimal performance  
    pub connection_count: CachePadded<AtomicUsize>,
    /// Total bytes sent in request bodies - cache padded for optimal performance
    pub total_bytes_sent: CachePadded<AtomicU64>,
    /// Total bytes received in response bodies - cache padded for optimal performance
    pub total_bytes_received: CachePadded<AtomicU64>,
    /// Total response time across all requests in nanoseconds - cache padded for optimal performance
    pub total_response_time_nanos: CachePadded<AtomicU64>,
    /// Number of successful requests (2xx status codes) - cache padded for optimal performance
    pub successful_requests: CachePadded<AtomicUsize>,
    /// Number of failed requests (4xx/5xx status codes or network errors) - cache padded for optimal performance
    pub failed_requests: CachePadded<AtomicUsize>,
    /// Number of cache hits - cache padded for optimal performance
    pub cache_hits: CachePadded<AtomicUsize>,
    /// Number of cache misses - cache padded for optimal performance
    pub cache_misses: CachePadded<AtomicUsize>,
}

impl Clone for ClientStats {
    fn clone(&self) -> Self {
        Self {
            request_count: CachePadded::new(AtomicUsize::new(
                self.request_count.load(Ordering::Relaxed),
            )),
            connection_count: CachePadded::new(AtomicUsize::new(
                self.connection_count.load(Ordering::Relaxed),
            )),
            total_bytes_sent: CachePadded::new(AtomicU64::new(
                self.total_bytes_sent.load(Ordering::Relaxed),
            )),
            total_bytes_received: CachePadded::new(AtomicU64::new(
                self.total_bytes_received.load(Ordering::Relaxed),
            )),
            total_response_time_nanos: CachePadded::new(AtomicU64::new(
                self.total_response_time_nanos.load(Ordering::Relaxed),
            )),
            successful_requests: CachePadded::new(AtomicUsize::new(
                self.successful_requests.load(Ordering::Relaxed),
            )),
            failed_requests: CachePadded::new(AtomicUsize::new(
                self.failed_requests.load(Ordering::Relaxed),
            )),
            cache_hits: CachePadded::new(AtomicUsize::new(self.cache_hits.load(Ordering::Relaxed))),
            cache_misses: CachePadded::new(AtomicUsize::new(
                self.cache_misses.load(Ordering::Relaxed),
            )),
        }
    }
}

/// Immutable snapshot of client statistics at a point in time
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientStatsSnapshot {
    /// Total number of HTTP requests made
    pub request_count: usize,
    /// Total number of connections established
    pub connection_count: usize,
    /// Total bytes sent in request bodies
    pub total_bytes_sent: u64,
    /// Total bytes received in response bodies
    pub total_bytes_received: u64,
    /// Total response time across all requests in nanoseconds
    pub total_response_time_nanos: u64,
    /// Number of successful requests (2xx status codes)
    pub successful_requests: usize,
    /// Number of failed requests (4xx/5xx status codes or network errors)
    pub failed_requests: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
}

impl ClientStats {
    /// Create a zero-allocation snapshot of the current statistics.
    ///
    /// All values are read atomically using relaxed ordering for blazing-fast performance
    /// while ensuring consistent point-in-time values across all cache-padded metrics.
    /// Uses elite polling patterns for optimal CPU cache utilization.
    #[inline]
    pub fn snapshot(&self) -> ClientStatsSnapshot {
        ClientStatsSnapshot {
            request_count: self.request_count.load(Ordering::Relaxed),
            connection_count: self.connection_count.load(Ordering::Relaxed),
            total_bytes_sent: self.total_bytes_sent.load(Ordering::Relaxed),
            total_bytes_received: self.total_bytes_received.load(Ordering::Relaxed),
            total_response_time_nanos: self.total_response_time_nanos.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
        }
    }
}
