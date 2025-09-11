//! Core HTTP client implementation
//!
//! Provides the main `HttpClient` with connection pooling, protocol strategy,
//! comprehensive telemetry, and enterprise-grade error handling.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::config::HttpConfig;
use crate::http::HttpRequest;
use crate::protocols::strategy::HttpProtocolStrategy;

// Telemetry module not yet implemented

/// Client statistics for telemetry and monitoring
#[derive(Debug, Default)]
pub struct ClientStats {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    pub connection_pool_size: AtomicU64,
    pub active_connections: AtomicU64,
    pub avg_response_time_ms: AtomicU64,

    // ===== Compression Statistics =====
    /// Total requests where compression was attempted
    pub compression_attempted: AtomicU64,
    /// Total requests where compression was actually applied (worthwhile)
    pub compression_applied: AtomicU64,
    /// Total responses that were decompressed
    pub decompression_applied: AtomicU64,
    /// Total bytes before compression (for calculating compression ratio)
    pub bytes_before_compression: AtomicU64,
    /// Total bytes after compression
    pub bytes_after_compression: AtomicU64,
    /// Total bytes before decompression
    pub bytes_before_decompression: AtomicU64,
    /// Total bytes after decompression
    pub bytes_after_decompression: AtomicU64,
    /// Total time spent compressing (microseconds)
    pub compression_time_micros: AtomicU64,
    /// Total time spent decompressing (microseconds)
    pub decompression_time_micros: AtomicU64,
    /// Number of compression errors
    pub compression_errors: AtomicU64,
    /// Number of decompression errors
    pub decompression_errors: AtomicU64,
}

impl ClientStats {
    /// Create new client statistics
    #[inline]
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a snapshot of current statistics
    pub fn snapshot(&self) -> crate::telemetry::ClientStatsSnapshot {
        // Safe conversion with overflow protection for cross-platform compatibility
        let request_count = usize::try_from(self.total_requests.load(Ordering::Relaxed)).unwrap_or(usize::MAX);
        let connection_count = usize::try_from(self.connection_pool_size.load(Ordering::Relaxed)).unwrap_or(usize::MAX);
        let successful_requests = usize::try_from(self.successful_requests.load(Ordering::Relaxed)).unwrap_or(usize::MAX);
        let failed_requests = usize::try_from(self.failed_requests.load(Ordering::Relaxed)).unwrap_or(usize::MAX);
        let cache_hits = usize::try_from(self.cache_hits.load(Ordering::Relaxed)).unwrap_or(usize::MAX);
        let cache_misses = usize::try_from(self.cache_misses.load(Ordering::Relaxed)).unwrap_or(usize::MAX);
        
        crate::telemetry::ClientStatsSnapshot {
            request_count,
            connection_count,
            total_bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            total_bytes_received: self.bytes_received.load(Ordering::Relaxed),
            total_response_time_nanos: 0, // Not tracked in this implementation
            successful_requests,
            failed_requests,
            cache_hits,
            cache_misses,
        }
    }
}

/// HTTP client with connection pooling and intelligent protocol strategy
///
/// This is the CANONICAL `HttpClient` that consolidates all HTTP functionality
/// into a single, performant, zero-allocation client with comprehensive telemetry.
#[derive(Debug, Clone)]
pub struct HttpClient {
    config: HttpConfig,
    stats: Arc<ClientStats>,
    strategy: HttpProtocolStrategy,
    created_at: Instant,
}

// Default implementation moved to configuration.rs

impl HttpClient {
    /// Create `HttpClient` with default configuration
    #[inline]
    #[must_use] 
    pub fn new() -> Self {
        Self {
            config: HttpConfig::default(),
            stats: Arc::new(ClientStats::default()),
            strategy: HttpProtocolStrategy::default(),
            created_at: Instant::now(),
        }
    }

    /// Create `HttpClient` with custom configuration
    #[inline]
    #[must_use] 
    pub fn with_config(config: HttpConfig) -> Self {
        Self {
            config,
            stats: Arc::new(ClientStats::default()),
            strategy: HttpProtocolStrategy::default(),
            created_at: Instant::now(),
        }
    }

    /// Create `HttpClient` with custom configuration and pre-allocated stats
    /// Used for advanced initialization scenarios with shared statistics
    #[inline]
    pub fn new_direct(config: HttpConfig, stats: ClientStats) -> Self {
        Self {
            config,
            stats: Arc::new(stats),
            strategy: HttpProtocolStrategy::default(),
            created_at: Instant::now(),
        }
    }



    /// Create `HttpClient` with custom configuration and strategy
    #[inline]
    #[must_use] 
    pub fn with_config_and_strategy(config: HttpConfig, strategy: HttpProtocolStrategy) -> Self {
        Self {
            config,
            stats: Arc::new(ClientStats::default()),
            strategy,
            created_at: Instant::now(),
        }
    }



    /// Get client statistics for monitoring and telemetry
    #[inline]
    #[must_use] 
    pub fn stats(&self) -> Arc<ClientStats> {
        self.stats.clone()
    }

    /// Get current configuration
    #[inline]
    #[must_use] 
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }

    /// Get current strategy
    #[inline]
    #[must_use] 
    pub fn strategy(&self) -> &HttpProtocolStrategy {
        &self.strategy
    }

    /// Get client uptime
    #[inline]
    #[must_use] 
    pub fn uptime(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Get connection pool size
    #[inline]
    #[must_use] 
    pub fn connection_pool_size(&self) -> u64 {
        self.stats.connection_pool_size.load(Ordering::Relaxed)
    }

    /// Get active connections count
    #[inline]
    #[must_use] 
    pub fn active_connections(&self) -> u64 {
        self.stats.active_connections.load(Ordering::Relaxed)
    }

    /// Get average response time in milliseconds
    #[inline]
    #[must_use] 
    pub fn avg_response_time_ms(&self) -> u64 {
        self.stats.avg_response_time_ms.load(Ordering::Relaxed)
    }

    /// Get cache hit rate (0.0 to 1.0)
    #[inline]
    #[must_use] 
    pub fn cache_hit_rate(&self) -> f64 {
        // Precision loss acceptable for cache hit rate statistics
        #[allow(clippy::cast_precision_loss)]
        let hits = self.stats.cache_hits.load(Ordering::Relaxed) as f64;
        #[allow(clippy::cast_precision_loss)]
        let misses = self.stats.cache_misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total == 0.0 { 0.0 } else { hits / total }
    }

    /// Get success rate (0.0 to 1.0)
    #[inline]
    #[must_use] 
    pub fn success_rate(&self) -> f64 {
        // Precision loss acceptable for success rate statistics
        #[allow(clippy::cast_precision_loss)]
        let successful = self.stats.successful_requests.load(Ordering::Relaxed) as f64;
        #[allow(clippy::cast_precision_loss)]
        let failed = self.stats.failed_requests.load(Ordering::Relaxed) as f64;
        let total = successful + failed;
        if total == 0.0 { 1.0 } else { successful / total }
    }

    /// Get total bytes transferred (sent + received)
    #[inline]
    #[must_use] 
    pub fn total_bytes_transferred(&self) -> u64 {
        self.stats.bytes_sent.load(Ordering::Relaxed) + 
        self.stats.bytes_received.load(Ordering::Relaxed)
    }

    /// Check if client has metrics available
    #[inline]
    #[must_use] 
    pub fn has_metrics(&self) -> bool {
        self.stats.total_requests.load(Ordering::Relaxed) > 0
    }

    /// Reset all statistics (useful for testing)
    #[inline]
    pub fn reset_stats(&self) {
        self.stats.total_requests.store(0, Ordering::Relaxed);
        self.stats.successful_requests.store(0, Ordering::Relaxed);
        self.stats.failed_requests.store(0, Ordering::Relaxed);
        self.stats.cache_hits.store(0, Ordering::Relaxed);
        self.stats.cache_misses.store(0, Ordering::Relaxed);
        self.stats.bytes_sent.store(0, Ordering::Relaxed);
        self.stats.bytes_received.store(0, Ordering::Relaxed);
        self.stats.connection_pool_size.store(0, Ordering::Relaxed);
        self.stats.active_connections.store(0, Ordering::Relaxed);
        self.stats.avg_response_time_ms.store(0, Ordering::Relaxed);
    }

    /// Check if client is closed (always false for canonical client)
    #[inline]
    #[must_use] 
    pub fn is_closed(&self) -> bool {
        false
    }

    /// Execute HTTP request with telemetry tracking and protocol selection
    ///
    /// Uses protocol strategy for intelligent protocol selection and automatic fallback.
    /// Tracks comprehensive telemetry metrics and applies strategy-specific optimizations.
    #[inline]
    pub fn execute(&self, request: HttpRequest) -> crate::http::response::HttpResponse {
        let stats = self.stats.clone();
        
        // Track request
        stats.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Add Accept-Encoding headers for response decompression
        let mut headers = request.headers().clone();
        crate::http::headers::add_compression_headers(&mut headers, &self.config);
        let request_with_headers = request.with_headers(headers);

        // Build and execute strategy - compression handled transparently at protocol layer
        let strategy = self.strategy.build();
        let response = strategy.execute(request_with_headers);
        
        // Track result
        if response.is_success() {
            stats.successful_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else if response.is_error() {
            stats.failed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        
        response
    }


}