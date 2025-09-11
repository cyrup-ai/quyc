//! DNS resolver statistics and telemetry
//!
//! This module provides performance statistics for DNS resolution operations.

use std::time::Duration;

/// Resolver performance statistics
#[derive(Debug, Clone)]
pub struct ResolverStats {
    pub request_count: u32,
    pub success_count: u64,
    pub failure_count: u64,
    pub timeout: Duration,
}

impl ResolverStats {
    /// Create a new `ResolverStats` instance
    #[must_use] 
    pub fn new(
        request_count: u32,
        success_count: u64,
        failure_count: u64,
        timeout: Duration,
    ) -> Self {
        Self {
            request_count,
            success_count,
            failure_count,
            timeout,
        }
    }

    /// Calculate success rate as percentage
    #[must_use] 
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.0
        } else {
            // Precision loss acceptable for DNS resolution success rate statistics
            #[allow(clippy::cast_precision_loss)]
            { (self.success_count as f64 / total as f64) * 100.0 }
        }
    }

    /// Calculate failure rate as percentage
    #[must_use] 
    pub fn failure_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }

    /// Calculate total requests processed
    #[must_use] 
    pub fn total_requests(&self) -> u64 {
        self.success_count + self.failure_count
    }

    /// Check if the resolver has processed any requests
    #[must_use] 
    pub fn has_activity(&self) -> bool {
        self.total_requests() > 0
    }
}
