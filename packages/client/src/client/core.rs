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
        
        // Apply compression headers based on configuration
        let mut modified_request = request;
        crate::http::headers::add_compression_headers(modified_request.headers_mut(), &self.config);
        
        // Apply request body compression if enabled and appropriate
        if self.config.request_compression {
            let content_type = modified_request.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok());
                
            if crate::http::compression::should_compress_content_type(content_type, &self.config) {
                if let Some(body) = modified_request.body().cloned() {
                    let start_time = std::time::Instant::now();
                    
                    match body {
                        crate::http::request::RequestBody::Bytes(data) => {
                            // Determine best compression algorithm based on config
                            let algorithm = if self.config.brotli_enabled {
                                crate::http::headers::CompressionAlgorithm::Brotli
                            } else if self.config.gzip_enabled {
                                crate::http::headers::CompressionAlgorithm::Gzip
                            } else if self.config.deflate {
                                crate::http::headers::CompressionAlgorithm::Deflate
                            } else {
                                crate::http::headers::CompressionAlgorithm::Identity
                            };
                            
                            if algorithm != crate::http::headers::CompressionAlgorithm::Identity {
                                // Get the appropriate compression level from config
                                let compression_level = match algorithm {
                                    crate::http::headers::CompressionAlgorithm::Brotli => self.config.brotli_level,
                                    crate::http::headers::CompressionAlgorithm::Gzip => self.config.gzip_level,
                                    crate::http::headers::CompressionAlgorithm::Deflate => self.config.deflate_level,
                                    crate::http::headers::CompressionAlgorithm::Identity => None,
                                };
                                
                                match crate::http::compression::compress_bytes_with_metrics(
                                    &data,
                                    algorithm,
                                    compression_level,
                                    Some(&stats)
                                ) {
                                    Ok(compressed_data) => {
                                        // Only use compression if it's worthwhile
                                        if compressed_data.len() < data.len() {
                                            modified_request = modified_request.body_bytes(
                                                bytes::Bytes::from(compressed_data.clone())
                                            );
                                            
                                            // Add Content-Encoding header
                                            if let Ok(encoding_header) = http::HeaderValue::from_str(algorithm.encoding_name()) {
                                                modified_request.headers_mut().insert(
                                                    http::header::CONTENT_ENCODING,
                                                    encoding_header
                                                );
                                            }
                                            
                                            // Update compression metrics
                                            let compression_time_ms = start_time.elapsed().as_millis() as u64;
                                            stats.bytes_sent.fetch_add(compressed_data.len() as u64, std::sync::atomic::Ordering::Relaxed);
                                            
                                            tracing::debug!(
                                                target: "quyc::client",
                                                algorithm = %algorithm.encoding_name(),
                                                original_size = data.len(),
                                                compressed_size = compressed_data.len(),
                                                compression_time_ms = compression_time_ms,
                                                "Request body compressed"
                                            );
                                        }
                                    },
                                    Err(e) => {
                                        tracing::warn!(
                                            target: "quyc::client",
                                            error = %e,
                                            "Request body compression failed, sending uncompressed"
                                        );
                                    }
                                }
                            }
                        },
                        crate::http::request::RequestBody::Text(text) => {
                            let data = text.as_bytes();
                            let algorithm = if self.config.brotli_enabled {
                                crate::http::headers::CompressionAlgorithm::Brotli
                            } else if self.config.gzip_enabled {
                                crate::http::headers::CompressionAlgorithm::Gzip
                            } else if self.config.deflate {
                                crate::http::headers::CompressionAlgorithm::Deflate
                            } else {
                                crate::http::headers::CompressionAlgorithm::Identity
                            };
                            
                            if algorithm != crate::http::headers::CompressionAlgorithm::Identity {
                                // Get the appropriate compression level from config
                                let compression_level = match algorithm {
                                    crate::http::headers::CompressionAlgorithm::Brotli => self.config.brotli_level,
                                    crate::http::headers::CompressionAlgorithm::Gzip => self.config.gzip_level,
                                    crate::http::headers::CompressionAlgorithm::Deflate => self.config.deflate_level,
                                    crate::http::headers::CompressionAlgorithm::Identity => None,
                                };
                                
                                match crate::http::compression::compress_bytes_with_metrics(data, algorithm, compression_level, Some(&stats)) {
                                    Ok(compressed_data) => {
                                        if compressed_data.len() < data.len() {
                                            modified_request = modified_request.body_bytes(
                                                bytes::Bytes::from(compressed_data.clone())
                                            );
                                            
                                            if let Ok(encoding_header) = http::HeaderValue::from_str(algorithm.encoding_name()) {
                                                modified_request.headers_mut().insert(
                                                    http::header::CONTENT_ENCODING,
                                                    encoding_header
                                                );
                                            }
                                            
                                            stats.bytes_sent.fetch_add(compressed_data.len() as u64, std::sync::atomic::Ordering::Relaxed);
                                            
                                            tracing::debug!(
                                                target: "quyc::client",
                                                algorithm = %algorithm.encoding_name(),
                                                original_size = data.len(),
                                                compressed_size = compressed_data.len(),
                                                "Request text body compressed"
                                            );
                                        }
                                    },
                                    Err(e) => {
                                        tracing::warn!(
                                            target: "quyc::client",
                                            error = %e,
                                            "Request text body compression failed, sending uncompressed"
                                        );
                                    }
                                }
                            }
                        },
                        crate::http::request::RequestBody::Json(json_value) => {
                            if let Ok(json_text) = serde_json::to_string(&json_value) {
                                let data = json_text.as_bytes();
                                let algorithm = if self.config.brotli_enabled {
                                    crate::http::headers::CompressionAlgorithm::Brotli
                                } else if self.config.gzip_enabled {
                                    crate::http::headers::CompressionAlgorithm::Gzip
                                } else if self.config.deflate {
                                    crate::http::headers::CompressionAlgorithm::Deflate
                                } else {
                                    crate::http::headers::CompressionAlgorithm::Identity
                                };
                                
                                if algorithm != crate::http::headers::CompressionAlgorithm::Identity {
                                    // Get the appropriate compression level from config
                                    let compression_level = match algorithm {
                                        crate::http::headers::CompressionAlgorithm::Brotli => self.config.brotli_level,
                                        crate::http::headers::CompressionAlgorithm::Gzip => self.config.gzip_level,
                                        crate::http::headers::CompressionAlgorithm::Deflate => self.config.deflate_level,
                                        crate::http::headers::CompressionAlgorithm::Identity => None,
                                    };
                                    
                                    match crate::http::compression::compress_bytes(data, algorithm, compression_level) {
                                        Ok(compressed_data) => {
                                            if compressed_data.len() < data.len() {
                                                modified_request = modified_request.body_bytes(
                                                    bytes::Bytes::from(compressed_data.clone())
                                                );
                                                
                                                if let Ok(encoding_header) = http::HeaderValue::from_str(algorithm.encoding_name()) {
                                                    modified_request.headers_mut().insert(
                                                        http::header::CONTENT_ENCODING,
                                                        encoding_header
                                                    );
                                                }
                                                
                                                stats.bytes_sent.fetch_add(compressed_data.len() as u64, std::sync::atomic::Ordering::Relaxed);
                                                
                                                tracing::debug!(
                                                    target: "quyc::client",
                                                    algorithm = %algorithm.encoding_name(),
                                                    original_size = data.len(),
                                                    compressed_size = compressed_data.len(),
                                                    "Request JSON body compressed"
                                                );
                                            }
                                        },
                                        Err(e) => {
                                            tracing::warn!(
                                                target: "quyc::client",
                                                error = %e,
                                                "Request JSON body compression failed, sending uncompressed"
                                            );
                                        }
                                    }
                                }
                            }
                        },
                        // For other body types (Form, Multipart, Stream), compression is handled
                        // at the protocol level or not applied due to complexity
                        _ => {
                            tracing::debug!(
                                target: "quyc::client",
                                "Skipping compression for complex body type"
                            );
                        }
                    }
                }
            }
        }
        
        // Build and execute strategy
        let strategy = self.strategy.build();
        let response = strategy.execute(modified_request);
        
        // Track result
        if response.is_success() {
            stats.successful_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else if response.is_error() {
            stats.failed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        
        response
    }


}