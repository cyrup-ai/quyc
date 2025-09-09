//! Global retry statistics for application-wide monitoring
//!
//! Provides atomic counters for tracking retry behavior across all
//! operations in the application with zero-allocation updates.

use std::sync::atomic::{AtomicU64, Ordering};

/// Global retry statistics for monitoring across all operations
pub struct GlobalRetryStats {
    total_operations: AtomicU64,
    total_retries: AtomicU64,
    total_failures: AtomicU64,
    total_successes: AtomicU64,
}

impl GlobalRetryStats {
    /// Create new global retry statistics
    ///
    /// Initializes all atomic counters to zero for clean startup state.
    /// This is a const function to allow static initialization.
    #[inline]
    pub const fn new() -> Self {
        Self {
            total_operations: AtomicU64::new(0),
            total_retries: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            total_successes: AtomicU64::new(0),
        }
    }

    /// Record an operation start
    ///
    /// Increments the total operation counter atomically using relaxed
    /// ordering for maximum performance in high-throughput scenarios.
    #[inline]
    pub fn record_operation(&self) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a retry attempt
    ///
    /// Increments the retry counter to track how often retry logic
    /// is triggered across the application.
    #[inline]
    pub fn record_retry(&self) {
        self.total_retries.fetch_add(1, Ordering::Relaxed);
    }

    /// Record final success
    ///
    /// Increments the success counter when an operation completes
    /// successfully, either on first attempt or after retries.
    #[inline]
    pub fn record_success(&self) {
        self.total_successes.fetch_add(1, Ordering::Relaxed);
    }

    /// Record final failure
    ///
    /// Increments the failure counter when an operation fails
    /// permanently after exhausting all retry attempts.
    #[inline]
    pub fn record_failure(&self) {
        self.total_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current statistics snapshot
    ///
    /// Returns a consistent snapshot of all counters read atomically.
    /// Values represent: (total_operations, total_retries, total_successes, total_failures)
    #[inline]
    pub fn snapshot(&self) -> (u64, u64, u64, u64) {
        (
            self.total_operations.load(Ordering::Relaxed),
            self.total_retries.load(Ordering::Relaxed),
            self.total_successes.load(Ordering::Relaxed),
            self.total_failures.load(Ordering::Relaxed),
        )
    }

    /// Calculate success rate percentage
    ///
    /// Returns the percentage of operations that completed successfully.
    /// Returns 0.0 if no operations have been recorded yet.
    #[inline]
    pub fn success_rate(&self) -> f64 {
        let (total_ops, _, successes, _) = self.snapshot();
        if total_ops > 0 {
            (successes as f64 / total_ops as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate failure rate percentage
    ///
    /// Returns the percentage of operations that failed permanently.
    /// Complements the success rate (success_rate + failure_rate = 100%).
    #[inline]
    pub fn failure_rate(&self) -> f64 {
        let (total_ops, _, _, failures) = self.snapshot();
        if total_ops > 0 {
            (failures as f64 / total_ops as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate average retries per operation
    ///
    /// Returns the average number of retry attempts per operation.
    /// Higher values indicate more frequent retry scenarios.
    #[inline]
    pub fn avg_retries_per_operation(&self) -> f64 {
        let (total_ops, total_retries, _, _) = self.snapshot();
        if total_ops > 0 {
            total_retries as f64 / total_ops as f64
        } else {
            0.0
        }
    }

    /// Reset all statistics to zero
    ///
    /// Clears all counters for fresh monitoring periods.
    /// Use with caution as this loses historical data.
    #[inline]
    pub fn reset(&self) {
        self.total_operations.store(0, Ordering::Relaxed);
        self.total_retries.store(0, Ordering::Relaxed);
        self.total_successes.store(0, Ordering::Relaxed);
        self.total_failures.store(0, Ordering::Relaxed);
    }
}

/// Global instance for tracking retry statistics across the application
///
/// This static instance provides centralized statistics collection
/// that can be accessed from anywhere in the application without
/// requiring dependency injection or context passing.
pub static GLOBAL_RETRY_STATS: GlobalRetryStats = GlobalRetryStats::new();
