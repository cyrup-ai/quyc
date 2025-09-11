//! Lock-free metrics collection for HTTP operations

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

/// Lock-free metrics collector using atomic counters
#[derive(Debug)]
pub struct MetricsCollector {
    /// Total requests processed
    pub total_requests: AtomicUsize,
    /// Successful requests (2xx status)
    pub successful_requests: AtomicUsize,
    /// Failed requests (non-2xx status or errors)
    pub failed_requests: AtomicUsize,
    /// Total response time in nanoseconds
    pub total_response_time_nanos: AtomicU64,
    /// Total bytes sent
    pub total_bytes_sent: AtomicU64,
    /// Total bytes received
    pub total_bytes_received: AtomicU64,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create new metrics collector
    #[must_use] 
    pub fn new() -> Self {
        Self {
            total_requests: AtomicUsize::new(0),
            successful_requests: AtomicUsize::new(0),
            failed_requests: AtomicUsize::new(0),
            total_response_time_nanos: AtomicU64::new(0),
            total_bytes_sent: AtomicU64::new(0),
            total_bytes_received: AtomicU64::new(0),
        }
    }

    /// Record successful request
    pub fn record_success(&self, response_time: Duration, bytes_sent: u64, bytes_received: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_response_time_nanos
            .fetch_add(response_time.as_nanos() as u64, Ordering::Relaxed);
        self.total_bytes_sent
            .fetch_add(bytes_sent, Ordering::Relaxed);
        self.total_bytes_received
            .fetch_add(bytes_received, Ordering::Relaxed);
    }

    /// Record failed request
    pub fn record_failure(&self, response_time: Duration) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        self.total_response_time_nanos
            .fetch_add(response_time.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> RequestMetrics {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let successful_requests = self.successful_requests.load(Ordering::Relaxed);
        let failed_requests = self.failed_requests.load(Ordering::Relaxed);
        let total_response_time_nanos = self.total_response_time_nanos.load(Ordering::Relaxed);
        let total_bytes_sent = self.total_bytes_sent.load(Ordering::Relaxed);
        let total_bytes_received = self.total_bytes_received.load(Ordering::Relaxed);

        let average_response_time = if total_requests > 0 {
            Duration::from_nanos(total_response_time_nanos / total_requests as u64)
        } else {
            Duration::ZERO
        };

        RequestMetrics {
            total_requests,
            successful_requests,
            failed_requests,
            average_response_time,
            total_bytes_sent,
            total_bytes_received,
        }
    }
}

/// Request metrics snapshot
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    /// Total number of HTTP requests made
    pub total_requests: usize,
    /// Number of successful requests (2xx status)
    pub successful_requests: usize,
    /// Number of failed requests (4xx/5xx status or errors)
    pub failed_requests: usize,
    /// Average response time across all requests
    pub average_response_time: Duration,
    /// Total bytes sent in request bodies
    pub total_bytes_sent: u64,
    /// Total bytes received in response bodies
    pub total_bytes_received: u64,
}

impl RequestMetrics {
    /// Calculate success rate as percentage
    #[must_use] 
    pub fn success_rate(&self) -> f64 {
        if self.total_requests > 0 {
            // Precision loss acceptable for request success rate statistics
            #[allow(clippy::cast_precision_loss)]
            { (self.successful_requests as f64 / self.total_requests as f64) * 100.0 }
        } else {
            0.0
        }
    }
}

/// Operation-specific metrics
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    /// Type of HTTP operation (GET, POST, etc.)
    pub operation_type: String,
    /// Number of operations of this type
    pub count: usize,
    /// Average response time for this operation type
    pub average_response_time: Duration,
    /// Success rate for this operation type (0.0-1.0)
    pub success_rate: f64,
}

/// Global metrics instance
pub static GLOBAL_METRICS: std::sync::LazyLock<MetricsCollector> = std::sync::LazyLock::new(MetricsCollector::new);
