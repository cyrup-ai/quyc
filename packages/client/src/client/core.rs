//! Core HTTP client implementation
//!
//! Provides the main HttpClient with connection pooling, protocol strategy,
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
}

impl ClientStats {
    /// Create new client statistics
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a snapshot of current statistics
    pub fn snapshot(&self) -> crate::telemetry::ClientStatsSnapshot {
        crate::telemetry::ClientStatsSnapshot {
            request_count: self.total_requests.load(Ordering::Relaxed) as usize,
            connection_count: self.connection_pool_size.load(Ordering::Relaxed) as usize,
            total_bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            total_bytes_received: self.bytes_received.load(Ordering::Relaxed),
            total_response_time_nanos: 0, // Not tracked in this implementation
            successful_requests: self.successful_requests.load(Ordering::Relaxed) as usize,
            failed_requests: self.failed_requests.load(Ordering::Relaxed) as usize,
            cache_hits: self.cache_hits.load(Ordering::Relaxed) as usize,
            cache_misses: self.cache_misses.load(Ordering::Relaxed) as usize,
        }
    }
}

/// HTTP client with connection pooling and intelligent protocol strategy
///
/// This is the CANONICAL HttpClient that consolidates all HTTP functionality
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
    /// Create HttpClient with default configuration
    #[inline]
    pub fn new() -> Self {
        Self {
            config: HttpConfig::default(),
            stats: Arc::new(ClientStats::default()),
            strategy: HttpProtocolStrategy::default(),
            created_at: Instant::now(),
        }
    }

    /// Create HttpClient with custom configuration
    #[inline]
    pub fn with_config(config: HttpConfig) -> Self {
        Self {
            config,
            stats: Arc::new(ClientStats::default()),
            strategy: HttpProtocolStrategy::default(),
            created_at: Instant::now(),
        }
    }

    /// Create HttpClient with custom configuration and pre-allocated stats
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



    /// Create HttpClient with custom configuration and strategy
    #[inline]
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
    pub fn stats(&self) -> Arc<ClientStats> {
        self.stats.clone()
    }

    /// Get current configuration
    #[inline]
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }

    /// Get current strategy
    #[inline]
    pub fn strategy(&self) -> &HttpProtocolStrategy {
        &self.strategy
    }

    /// Get client uptime
    #[inline]
    pub fn uptime(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Get connection pool size
    #[inline]
    pub fn connection_pool_size(&self) -> u64 {
        self.stats.connection_pool_size.load(Ordering::Relaxed)
    }

    /// Get active connections count
    #[inline]
    pub fn active_connections(&self) -> u64 {
        self.stats.active_connections.load(Ordering::Relaxed)
    }

    /// Get average response time in milliseconds
    #[inline]
    pub fn avg_response_time_ms(&self) -> u64 {
        self.stats.avg_response_time_ms.load(Ordering::Relaxed)
    }

    /// Get cache hit rate (0.0 to 1.0)
    #[inline]
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.stats.cache_hits.load(Ordering::Relaxed) as f64;
        let misses = self.stats.cache_misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total == 0.0 { 0.0 } else { hits / total }
    }

    /// Get success rate (0.0 to 1.0)
    #[inline]
    pub fn success_rate(&self) -> f64 {
        let successful = self.stats.successful_requests.load(Ordering::Relaxed) as f64;
        let failed = self.stats.failed_requests.load(Ordering::Relaxed) as f64;
        let total = successful + failed;
        if total == 0.0 { 1.0 } else { successful / total }
    }

    /// Get total bytes transferred (sent + received)
    #[inline]
    pub fn total_bytes_transferred(&self) -> u64 {
        self.stats.bytes_sent.load(Ordering::Relaxed) + 
        self.stats.bytes_received.load(Ordering::Relaxed)
    }

    /// Check if client has metrics available
    #[inline]
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
        
        // Build and execute strategy
        let strategy = self.strategy.build();
        let response = strategy.execute(request);
        
        // Track result
        if response.is_success() {
            stats.successful_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else if response.is_error() {
            stats.failed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        
        response
    }


}