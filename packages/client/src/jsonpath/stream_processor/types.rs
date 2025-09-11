use std::sync::{Arc, atomic::AtomicU64};

/// Circuit breaker state for error recovery
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, processing normally
    Closed,
    /// Circuit is open, failing fast due to consecutive errors
    Open,
    /// Circuit is half-open, allowing limited requests to test recovery
    HalfOpen,
}

/// Error recovery state with circuit breaker pattern
#[derive(Debug)]
pub struct ErrorRecoveryState {
    /// Current circuit breaker state
    pub(super) circuit_state: Arc<AtomicU64>, // 0=Closed, 1=Open, 2=HalfOpen
    /// Consecutive failure count
    pub(super) consecutive_failures: Arc<AtomicU64>,
    /// Timestamp of last failure (microseconds since epoch)
    pub(super) last_failure_time: Arc<AtomicU64>,
    /// Circuit breaker failure threshold
    pub(super) failure_threshold: u64,
    /// Circuit breaker timeout (microseconds)
    pub(super) circuit_timeout_micros: u64,
    /// Maximum backoff delay (microseconds)
    pub(super) max_backoff_micros: u64,
}

impl Default for ErrorRecoveryState {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecoveryState {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            circuit_state: Arc::new(AtomicU64::new(0)), // Closed
            consecutive_failures: Arc::new(AtomicU64::new(0)),
            last_failure_time: Arc::new(AtomicU64::new(0)),
            failure_threshold: 5,
            circuit_timeout_micros: 30_000_000, // 30 seconds
            max_backoff_micros: 60_000_000,     // 60 seconds
        }
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        use std::sync::atomic::Ordering;
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.circuit_state.store(0, Ordering::Relaxed); // Closed
    }

    /// Check if a request should be allowed
    #[must_use] 
    pub fn should_allow_request(&self) -> bool {
        use std::sync::atomic::Ordering;
        let state = self.circuit_state.load(Ordering::Relaxed);

        match state {
            0 => true, // Closed - allow all requests
            1 => {
                // Open - check if timeout has passed
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_micros() as u64;
                let last_failure = self.last_failure_time.load(Ordering::Relaxed);

                if now.saturating_sub(last_failure) > self.circuit_timeout_micros {
                    // Transition to half-open
                    self.circuit_state.store(2, Ordering::Relaxed);
                    true
                } else {
                    false
                }
            }
            2 => true,  // HalfOpen - allow limited requests
            _ => false, // Unknown state - be conservative
        }
    }

    /// Record a failure
    pub fn record_failure(&self) {
        use std::sync::atomic::Ordering;
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;
        self.last_failure_time.store(now, Ordering::Relaxed);

        if failures >= self.failure_threshold {
            self.circuit_state.store(1, Ordering::Relaxed); // Open
        }
    }

    /// Get backoff delay in microseconds
    #[must_use] 
    pub fn get_backoff_delay_micros(&self) -> u64 {
        use std::sync::atomic::Ordering;
        let failures = self.consecutive_failures.load(Ordering::Relaxed);

        // Exponential backoff: base_delay * 2^failures, capped at max_backoff
        let base_delay_micros = 1_000_000; // 1 second
        let delay = base_delay_micros * (1u64 << failures.min(10)); // Cap at 2^10 to prevent overflow
        delay.min(self.max_backoff_micros)
    }

    /// Get current circuit breaker state
    #[must_use] 
    pub fn get_current_state(&self) -> CircuitState {
        use std::sync::atomic::Ordering;
        match self.circuit_state.load(Ordering::Relaxed) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed, // Default to closed for unknown states
        }
    }
}

/// Lock-free performance statistics for `JsonStreamProcessor`
#[derive(Debug)]
pub struct ProcessorStats {
    /// Total chunks processed from HTTP response
    pub chunks_processed: Arc<AtomicU64>,
    /// Total bytes processed from HTTP response
    pub bytes_processed: Arc<AtomicU64>,
    /// Objects successfully deserialized and yielded
    pub objects_yielded: Arc<AtomicU64>,
    /// Processing errors encountered
    pub processing_errors: Arc<AtomicU64>,
    /// JSON parsing errors encountered
    pub parse_errors: Arc<AtomicU64>,
    /// Start time for throughput calculation
    pub start_time: Arc<AtomicU64>,
    /// Last processing timestamp for latency tracking
    pub last_process_time: Arc<AtomicU64>,
}

// ProcessorStats::new() implementation is in crate::telemetry::jsonpath::stream

/// Immutable snapshot of processor statistics
#[derive(Debug, Clone, Copy)]
pub struct ProcessorStatsSnapshot {
    /// Total chunks processed
    pub chunks_processed: u64,
    /// Total bytes processed
    pub bytes_processed: u64,
    /// Objects successfully yielded
    pub objects_yielded: u64,
    /// Processing errors encountered
    pub processing_errors: u64,
    /// JSON parsing errors
    pub parse_errors: u64,
    /// Objects processed per second
    pub throughput_objects_per_sec: f64,
    /// Average bytes per object
    pub bytes_per_object: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Total elapsed processing time in seconds
    pub elapsed_seconds: f64,
}

/// High-performance HTTP chunk processor for `JSONPath` streaming
pub struct JsonStreamProcessor<T>
where
    T: ystream::prelude::MessageChunk + Default + Send + 'static,
{
    pub(super) json_array_stream: super::super::JsonArrayStream<T>,
    pub(super) chunk_handlers: Vec<
        Box<
            dyn FnMut(
                    Result<T, super::super::JsonPathError>,
                ) -> Result<T, super::super::JsonPathError>
                + Send,
        >,
    >,
    pub(super) stats: ProcessorStats,
    pub(super) error_recovery: ErrorRecoveryState,
}

// JsonStreamProcessor implementations are in stream_processor/core.rs
