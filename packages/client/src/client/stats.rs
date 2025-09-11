//! Client statistics and performance metrics

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// HTTP client statistics and metrics
#[derive(Debug)]
pub struct ClientStats {
    /// Total number of requests made
    pub requests_total: AtomicU64,
    /// Number of successful requests (2xx status)
    pub requests_successful: AtomicU64,
    /// Number of failed requests (4xx/5xx status)
    pub requests_failed: AtomicU64,
    /// Total bytes sent
    pub bytes_sent: AtomicU64,
    /// Total bytes received
    pub bytes_received: AtomicU64,
    /// Number of active connections
    pub connections_active: AtomicU64,
    /// Total connection attempts
    pub connections_total: AtomicU64,
    /// Number of connection failures
    pub connections_failed: AtomicU64,
    /// Client creation time
    pub created_at: Instant,
}

impl Default for ClientStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientStats {
    /// Create new client statistics
    #[must_use] 
    pub fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            requests_successful: AtomicU64::new(0),
            requests_failed: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            connections_active: AtomicU64::new(0),
            connections_total: AtomicU64::new(0),
            connections_failed: AtomicU64::new(0),
            created_at: Instant::now(),
        }
    }

    /// Record a request
    pub fn record_request(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful request
    pub fn record_success(&self) {
        self.requests_successful.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed request
    pub fn record_failure(&self) {
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record bytes sent
    pub fn record_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record bytes received
    pub fn record_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record new connection
    pub fn record_connection(&self) {
        self.connections_total.fetch_add(1, Ordering::Relaxed);
        self.connections_active.fetch_add(1, Ordering::Relaxed);
    }

    /// Record connection closed
    pub fn record_connection_closed(&self) {
        self.connections_active.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record connection failure
    pub fn record_connection_failure(&self) {
        self.connections_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Get success ratio
    pub fn success_ratio(&self) -> f64 {
        let total = self.requests_total.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            let successful = self.requests_successful.load(Ordering::Relaxed);
            // Precision loss acceptable for success rate statistics
            #[allow(clippy::cast_precision_loss)]
            { successful as f64 / total as f64 }
        }
    }

    /// Get client age
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Create a snapshot of current statistics
    pub fn snapshot(&self) -> ClientStatsSnapshot {
        ClientStatsSnapshot {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_successful: self.requests_successful.load(Ordering::Relaxed),
            requests_failed: self.requests_failed.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            connections_active: self.connections_active.load(Ordering::Relaxed),
            connections_total: self.connections_total.load(Ordering::Relaxed),
            connections_failed: self.connections_failed.load(Ordering::Relaxed),
            success_ratio: self.success_ratio(),
            age: self.age(),
        }
    }
}

/// Snapshot of client statistics at a point in time
#[derive(Debug, Clone)]
pub struct ClientStatsSnapshot {
    pub requests_total: u64,
    pub requests_successful: u64,
    pub requests_failed: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_active: u64,
    pub connections_total: u64,
    pub connections_failed: u64,
    pub success_ratio: f64,
    pub age: Duration,
}
