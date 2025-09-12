use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::jsonpath::stream_processor::types::{ProcessorStats, ProcessorStatsSnapshot};

impl ProcessorStats {
    /// Create new processor statistics tracker
    #[must_use] 
    pub fn new() -> Self {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_micros()).unwrap_or(u64::MAX))
            .unwrap_or(0);

        Self {
            chunks_processed: std::sync::Arc::new(AtomicU64::new(0)),
            bytes_processed: std::sync::Arc::new(AtomicU64::new(0)),
            objects_yielded: std::sync::Arc::new(AtomicU64::new(0)),
            processing_errors: std::sync::Arc::new(AtomicU64::new(0)),
            parse_errors: std::sync::Arc::new(AtomicU64::new(0)),
            start_time: std::sync::Arc::new(AtomicU64::new(now_micros)),
            last_process_time: std::sync::Arc::new(AtomicU64::new(now_micros)),
        }
    }

    /// Get current statistics snapshot
    #[must_use] 
    pub fn snapshot(&self) -> ProcessorStatsSnapshot {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_micros()).unwrap_or(u64::MAX))
            .unwrap_or(0);

        let start_time = self.start_time.load(Ordering::Relaxed);
        let elapsed_micros = now_micros.saturating_sub(start_time);
        // Precision loss acceptable for telemetry time calculations
        #[allow(clippy::cast_precision_loss)]
        let elapsed_seconds = (elapsed_micros as f64) / 1_000_000.0;

        let chunks = self.chunks_processed.load(Ordering::Relaxed);
        let bytes = self.bytes_processed.load(Ordering::Relaxed);
        let objects = self.objects_yielded.load(Ordering::Relaxed);
        let errors = self.processing_errors.load(Ordering::Relaxed);
        let parse_errors = self.parse_errors.load(Ordering::Relaxed);

        ProcessorStatsSnapshot {
            chunks_processed: chunks,
            bytes_processed: bytes,
            objects_yielded: objects,
            processing_errors: errors,
            parse_errors,
            throughput_objects_per_sec: if elapsed_seconds > 0.0 {
                // Precision loss acceptable for throughput statistics
                #[allow(clippy::cast_precision_loss)]
                { objects as f64 / elapsed_seconds }
            } else {
                0.0
            },
            bytes_per_object: if objects > 0 {
                // Precision loss acceptable for bytes per object statistics
                #[allow(clippy::cast_precision_loss)]
                { bytes as f64 / objects as f64 }
            } else {
                0.0
            },
            error_rate: if chunks > 0 {
                // Precision loss acceptable for error rate statistics
                #[allow(clippy::cast_precision_loss)]
                { (errors + parse_errors) as f64 / chunks as f64 }
            } else {
                0.0
            },
            elapsed_seconds,
        }
    }

    /// Record successful object yield
    pub fn record_object_yield(&self) {
        self.objects_yielded.fetch_add(1, Ordering::Relaxed);
    }

    /// Record processing error
    pub fn record_processing_error(&self) {
        self.processing_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record parse error
    pub fn record_parse_error(&self) {
        self.parse_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record chunk processing
    pub fn record_chunk_processed(&self, bytes: usize) {
        self.chunks_processed.fetch_add(1, Ordering::Relaxed);
        self.bytes_processed
            .fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// Update last processing timestamp
    pub fn update_last_process_time(&self) {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_micros()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        self.last_process_time.store(now_micros, Ordering::Relaxed);
    }
}

impl Default for ProcessorStats {
    fn default() -> Self {
        Self::new()
    }
}
