//! JSONPath processing statistics

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Statistics for JSONPath stream processing
#[derive(Debug)]
pub struct JsonPathStats {
    /// Number of JSON objects processed
    pub objects_processed: AtomicU64,
    /// Number of JSONPath matches found
    pub matches_found: AtomicU64,
    /// Number of processing errors
    pub errors: AtomicU64,
    /// Total bytes processed
    pub bytes_processed: AtomicU64,
    /// Processing start time
    pub start_time: Instant,
}

impl JsonPathStats {
    /// Create new JSONPath statistics
    pub fn new() -> Self {
        Self {
            objects_processed: AtomicU64::new(0),
            matches_found: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            bytes_processed: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    /// Record an object processed
    pub fn record_object(&self) {
        self.objects_processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a match found
    pub fn record_match(&self) {
        self.matches_found.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an error
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record bytes processed
    pub fn record_bytes(&self, bytes: u64) {
        self.bytes_processed.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Get processing duration
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}
