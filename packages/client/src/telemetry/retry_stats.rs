//! Individual retry operation statistics tracking
//!
//! Provides detailed statistics collection for individual retry sequences
//! with comprehensive timing and error tracking.

use std::time::{Duration, Instant};

/// Retry statistics for monitoring and observability
#[derive(Debug, Clone)]
pub struct RetryStats {
    /// Total number of attempts made
    pub total_attempts: u32,
    /// Number of successful retries  
    pub successful_retries: u32,
    /// Total time spent in retries (excluding delays)
    pub total_retry_time: Duration,
    /// Total time spent waiting between retries
    pub total_delay_time: Duration,
    /// Timestamp when retry sequence started
    pub start_time: Instant,
    /// Final result timestamp
    pub end_time: Option<Instant>,
    /// List of errors encountered during retries
    pub retry_errors: Vec<String>,
}

impl Default for RetryStats {
    /// Create new retry statistics with current timestamp
    ///
    /// Initializes all counters to zero and sets the start time
    /// to the current instant for accurate timing measurements.
    fn default() -> Self {
        Self {
            total_attempts: 0,
            successful_retries: 0,
            total_retry_time: Duration::ZERO,
            total_delay_time: Duration::ZERO,
            start_time: Instant::now(),
            end_time: None,
            retry_errors: Vec::new(),
        }
    }
}

impl RetryStats {
    /// Mark retry sequence as completed
    ///
    /// Records the completion timestamp for accurate total elapsed
    /// time calculation. Should be called when the retry sequence
    /// finishes regardless of success or failure.
    #[inline]
    pub fn complete(&mut self) {
        self.end_time = Some(Instant::now());
    }

    /// Get total elapsed time for entire retry sequence
    ///
    /// Returns the total wall-clock time from start to completion,
    /// including both retry attempts and delay periods. If the sequence
    /// is still active, returns elapsed time to current moment.
    #[inline]
    pub fn total_elapsed(&self) -> Duration {
        match self.end_time {
            Some(end) => end.duration_since(self.start_time),
            None => self.start_time.elapsed(),
        }
    }

    /// Check if any retries were successful
    ///
    /// Returns true if at least one retry attempt succeeded after
    /// an initial failure, indicating the retry logic was beneficial.
    #[inline]
    pub fn had_successful_retry(&self) -> bool {
        self.successful_retries > 0
    }

    /// Calculate success rate as percentage
    ///
    /// Returns the percentage of attempts that ultimately succeeded.
    /// Only meaningful after the retry sequence has completed.
    #[inline]
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts > 0 {
            let successful_attempts = if self.had_successful_retry() { 1 } else { 0 };
            (successful_attempts as f64 / self.total_attempts as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get average time per retry attempt
    ///
    /// Returns the average time spent on actual retry attempts,
    /// excluding delay periods. Useful for understanding operation
    /// performance characteristics.
    #[inline]
    pub fn average_retry_time(&self) -> Duration {
        if self.total_attempts > 0 {
            self.total_retry_time / self.total_attempts
        } else {
            Duration::ZERO
        }
    }

    /// Get ratio of delay time to retry time
    ///
    /// Returns how much time was spent waiting between retries
    /// compared to time spent on actual attempts. Higher ratios
    /// indicate longer backoff periods.
    #[inline]
    pub fn delay_to_retry_ratio(&self) -> f64 {
        if self.total_retry_time > Duration::ZERO {
            self.total_delay_time.as_secs_f64() / self.total_retry_time.as_secs_f64()
        } else {
            0.0
        }
    }
}
