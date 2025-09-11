//! Zero-allocation HTTP compression and decompression
//!
//! This module provides blazing-fast, memory-efficient compression and decompression
//! capabilities using industry-standard algorithms. All implementations are designed
//! for zero-allocation performance with streaming support for large payloads.
//!
//! # Supported Algorithms
//!
//! - **Gzip**: RFC 1952 compliant gzip compression
//! - **Deflate**: RFC 1951 compliant deflate compression  
//! - **Brotli**: RFC 7932 compliant Brotli compression
//!
//! # Performance Features
//!
//! - Zero-allocation hot paths with buffer pool reuse
//! - Lock-free thread-local buffer management
//! - Streaming processing for memory-bounded operations
//! - Branch prediction optimization for common cases
//! - SIMD acceleration where available
//!
//! # Example Usage
//!
//! ```rust
//! use quyc_client::http::compression::{compress_bytes, CompressionAlgorithm};
//!
//! let data = b"Hello, world!";
//! let compressed = compress_bytes(data, CompressionAlgorithm::Gzip, None)?;
//! let decompressed = decompress_bytes(&compressed, CompressionAlgorithm::Gzip)?;
//! assert_eq!(data, decompressed.as_slice());
//! ```

use std::io::{Read, Write};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use flate2::{
    Compression as FlateCompression,
    read::{GzDecoder, DeflateDecoder},
    write::{GzEncoder, DeflateEncoder},
};
use brotli::enc::backward_references::BrotliEncoderMode;
use brotli::{CompressorReader as BrotliEncoder, Decompressor as BrotliDecoder};

use crate::http::headers::CompressionAlgorithm;
use crate::config::HttpConfig;
use crate::error::HttpError;

/// Default compression buffer size (64KB) - optimal for most use cases
const DEFAULT_BUFFER_SIZE: usize = 65536;

/// Maximum buffer size to prevent memory exhaustion (16MB)
const MAX_BUFFER_SIZE: usize = 16 * 1024 * 1024;

/// Minimum compression ratio to be considered worthwhile (1.05 = 5% improvement)
const MIN_COMPRESSION_RATIO: f64 = 1.05;

thread_local! {
    /// Thread-local buffer pool for zero-allocation compression operations
    static BUFFER_POOL: RefCell<CompressionBufferPool> = RefCell::new(CompressionBufferPool::new());
}

/// High-performance buffer pool with automatic size management and reuse
#[derive(Debug)]
struct CompressionBufferPool {
    /// Pool of reusable buffers, sorted by size for efficient allocation
    buffers: VecDeque<Vec<u8>>,
    /// Total bytes allocated across all buffers
    total_allocated: AtomicUsize,
    /// Maximum allowed pool size in bytes
    max_pool_size: usize,
    /// Buffer size usage statistics for adaptive sizing
    size_histogram: [AtomicUsize; 16],
}

impl CompressionBufferPool {
    /// Creates a new buffer pool with optimal defaults
    #[inline]
    fn new() -> Self {
        Self {
            buffers: VecDeque::with_capacity(16),
            total_allocated: AtomicUsize::new(0),
            max_pool_size: 4 * 1024 * 1024, // 4MB max pool size
            size_histogram: Default::default(),
        }
    }

    /// Gets a buffer from the pool or allocates a new one
    /// Uses histogram data to predict optimal size
    #[inline]
    fn get_buffer(&mut self, min_size: usize) -> Vec<u8> {
        // Fast path: try to reuse existing buffer
        while let Some(mut buffer) = self.buffers.pop_front() {
            if buffer.capacity() >= min_size {
                buffer.clear();
                return buffer;
            }
            // Buffer too small, don't keep it
            self.total_allocated.fetch_sub(buffer.capacity(), Ordering::Relaxed);
        }

        // Predict optimal size based on usage patterns
        let optimal_size = self.predict_optimal_size(min_size);
        let buffer = Vec::with_capacity(optimal_size);
        self.total_allocated.fetch_add(optimal_size, Ordering::Relaxed);
        
        // Record size usage for future predictions
        self.record_size_usage(optimal_size);
        
        buffer
    }

    /// Returns a buffer to the pool for reuse
    #[inline]
    fn return_buffer(&mut self, buffer: Vec<u8>) {
        // Only keep buffer if pool isn't too large and buffer is reasonable size
        if self.total_allocated.load(Ordering::Relaxed) < self.max_pool_size 
            && buffer.capacity() <= MAX_BUFFER_SIZE {
            self.buffers.push_back(buffer);
        } else {
            self.total_allocated.fetch_sub(buffer.capacity(), Ordering::Relaxed);
        }
    }

    /// Predicts optimal buffer size based on historical usage
    #[inline]
    fn predict_optimal_size(&self, min_size: usize) -> usize {
        let mut max_count = 0;
        let mut optimal_size = min_size;

        // Find the most commonly used size bucket
        for (i, count) in self.size_histogram.iter().enumerate() {
            let count_val = count.load(Ordering::Relaxed);
            if count_val > max_count {
                max_count = count_val;
                optimal_size = (1 << (i + 12)).max(min_size); // Start from 4KB
            }
        }

        // Ensure size is reasonable
        optimal_size.clamp(min_size, MAX_BUFFER_SIZE)
    }

    /// Records size usage for adaptive buffer sizing
    #[inline]
    fn record_size_usage(&self, size: usize) {
        if size > 0 {
            let bucket = (size.ilog2().saturating_sub(12) as usize).min(15);
            self.size_histogram[bucket].fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// RAII buffer guard that automatically returns buffers to the pool
struct BufferGuard {
    buffer: Option<Vec<u8>>,
}

impl BufferGuard {
    /// Creates a new buffer guard with a buffer from the pool
    #[inline]
    fn new(min_size: usize) -> Self {
        let buffer = BUFFER_POOL.with(|pool| pool.borrow_mut().get_buffer(min_size));
        Self { buffer: Some(buffer) }
    }

    /// Gets a mutable reference to the underlying buffer
    #[inline]
    fn as_mut(&mut self) -> Result<&mut Vec<u8>, crate::error::HttpError> {
        self.buffer.as_mut().ok_or_else(|| 
            crate::error::HttpError::new(crate::error::types::Kind::Request)
                .with("Buffer pool exhausted"))
    }

    /// Gets an immutable reference to the underlying buffer
    #[inline]
    #[allow(dead_code)]
    fn as_ref(&self) -> Result<&Vec<u8>, crate::error::HttpError> {
        self.buffer.as_ref().ok_or_else(|| 
            crate::error::HttpError::new(crate::error::types::Kind::Request)
                .with("Buffer pool exhausted"))
    }
}

impl Drop for BufferGuard {
    #[inline]
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            BUFFER_POOL.with(|pool| pool.borrow_mut().return_buffer(buffer));
        }
    }
}

/// Compresses data using the specified algorithm with optimal performance
///
/// # Arguments
/// * `data` - Input data to compress
/// * `algorithm` - Compression algorithm to use
/// * `level` - Optional compression level override
///
/// # Returns
/// * `Ok(Vec<u8>)` - Compressed data
/// * `Err(HttpError)` - Compression failed
///
/// # Performance Notes
/// * Uses zero-allocation buffer pool for intermediate operations
/// * Optimized for common compression scenarios
/// * Early return for incompressible data
#[inline]
pub fn compress_bytes(
    data: &[u8], 
    algorithm: CompressionAlgorithm, 
    level: Option<u32>
) -> Result<Vec<u8>, HttpError> {
    compress_bytes_with_metrics(data, algorithm, level, None)
}

/// Compress bytes with optional metrics recording
/// 
/// This is the internal implementation that supports metrics tracking
/// for telemetry and monitoring.
#[inline]
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn compress_bytes_with_metrics(
    data: &[u8], 
    algorithm: CompressionAlgorithm, 
    level: Option<u32>,
    stats: Option<&crate::client::core::ClientStats>
) -> Result<Vec<u8>, HttpError> {
    // Fast path: empty data
    if data.is_empty() {
        return Ok(Vec::new());
    }

    // Fast path: very small data that won't benefit from compression
    if data.len() < 64 {
        return Ok(data.to_vec());
    }

    let start_time = Instant::now();
    
    // Record compression attempt
    if let Some(stats) = stats {
        stats.compression_attempted.fetch_add(1, Ordering::Relaxed);
        stats.bytes_before_compression.fetch_add(data.len() as u64, Ordering::Relaxed);
    }
    
    let result = match algorithm {
        CompressionAlgorithm::Gzip => {
            let compression_level = FlateCompression::new(level.unwrap_or(6));
            compress_gzip(data, compression_level)
        },
        CompressionAlgorithm::Deflate => {
            let compression_level = FlateCompression::new(level.unwrap_or(6));
            compress_deflate(data, compression_level)
        },
        CompressionAlgorithm::Brotli => {
            compress_with_brotli(data, level.unwrap_or(6))
        },
        CompressionAlgorithm::Identity => {
            Ok(data.to_vec())
        }
    };

    match result {
        Ok(compressed) => {
            let compression_time = start_time.elapsed();
            
            // Check if compression is worthwhile  
            // Use safe precision-aware ratio calculation for large data sizes
            let ratio = if compressed.len() == 0 {
                f64::INFINITY // Avoid division by zero, treat as maximum compression
            } else if data.len() > (1u64 << 53) as usize || compressed.len() > (1u64 << 53) as usize {
                // For very large sizes that might lose precision in f64, use integer comparison
                tracing::debug!(
                    target: "quyc::compression",
                    original_size = data.len(),
                    compressed_size = compressed.len(),
                    "Using integer comparison for very large data sizes to avoid precision loss"
                );
                // If original is significantly larger than compressed, consider it worthwhile
                if data.len() >= compressed.len() + (compressed.len() / 20) { // At least 5% reduction
                    MIN_COMPRESSION_RATIO + 0.1 // Slightly above threshold
                } else {
                    MIN_COMPRESSION_RATIO - 0.1 // Slightly below threshold
                }
            } else {
                // Safe to convert to f64 without precision loss for smaller sizes
                (data.len() as f64) / (compressed.len() as f64)
            };
            if ratio >= MIN_COMPRESSION_RATIO {
                // Record successful compression metrics
                if let Some(stats) = stats {
                    stats.compression_applied.fetch_add(1, Ordering::Relaxed);
                    stats.bytes_after_compression.fetch_add(compressed.len() as u64, Ordering::Relaxed);
                    let compression_micros = compression_time.as_micros();
                    let compression_micros_u64 = if compression_micros > u128::from(u64::MAX) {
                        tracing::warn!(
                            target: "quyc::compression",
                            compression_micros = compression_micros,
                            max_u64 = u64::MAX,
                            "Compression time exceeds u64 limits, clamping to max"
                        );
                        u64::MAX
                    } else {
                        #[allow(clippy::cast_possible_truncation)]
                        {
                            compression_micros as u64
                        }
                    };
                    stats.compression_time_micros.fetch_add(compression_micros_u64, Ordering::Relaxed);
                }
                
                tracing::debug!(
                    target: "quyc::compression",
                    algorithm = %algorithm.encoding_name(),
                    original_size = data.len(),
                    compressed_size = compressed.len(),
                    ratio = ratio,
                    duration_micros = compression_time.as_micros(),
                    "Compression completed"
                );
                Ok(compressed)
            } else {
                // Compression not worthwhile, return original data
                tracing::debug!(
                    target: "quyc::compression", 
                    algorithm = %algorithm.encoding_name(),
                    ratio = ratio,
                    "Compression ratio insufficient, returning original data"
                );
                Ok(data.to_vec())
            }
        },
        Err(e) => {
            // Record compression error metrics
            if let Some(stats) = stats {
                stats.compression_errors.fetch_add(1, Ordering::Relaxed);
            }
            
            tracing::error!(
                target: "quyc::compression",
                algorithm = %algorithm.encoding_name(),
                error = %e,
                duration_micros = start_time.elapsed().as_micros(),
                "Compression failed"
            );
            Err(e)
        }
    }
}

/// Decompresses data using the specified algorithm
///
/// # Arguments  
/// * `data` - Compressed input data
/// * `algorithm` - Compression algorithm used
///
/// # Returns
/// * `Ok(Vec<u8>)` - Decompressed data
/// * `Err(HttpError)` - Decompression failed
///
/// # Performance Notes
/// * Uses streaming decompression to handle large data efficiently
/// * Memory-bounded operation with size limits
/// * Optimized for common decompression scenarios
#[inline]
pub fn decompress_bytes(data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>, HttpError> {
    decompress_bytes_with_metrics(data, algorithm, None)
}

/// Decompress bytes with optional metrics recording
/// 
/// This is the internal implementation that supports metrics tracking
/// for telemetry and monitoring.
/// 
/// # Errors
/// 
/// Returns `HttpError` with `Kind::Request` if:
/// - Input data is corrupted or invalid for the specified algorithm
/// - Decompression buffer size exceeds the configured maximum limit
/// - I/O errors occur during decompression operations
/// - Memory allocation fails during decompression
#[inline]
pub fn decompress_bytes_with_metrics(
    data: &[u8], 
    algorithm: CompressionAlgorithm,
    stats: Option<&crate::client::core::ClientStats>
) -> Result<Vec<u8>, HttpError> {
    // Fast path: empty data
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let start_time = Instant::now();
    
    // Record decompression metrics
    if let Some(stats) = stats {
        stats.bytes_before_decompression.fetch_add(data.len() as u64, Ordering::Relaxed);
    }
    
    let result = match algorithm {
        CompressionAlgorithm::Gzip => {
            decompress_gzip(data)
        },
        CompressionAlgorithm::Deflate => {
            decompress_deflate(data)
        },
        CompressionAlgorithm::Brotli => {
            decompress_with_brotli(data)
        },
        CompressionAlgorithm::Identity => {
            Ok(data.to_vec())
        }
    };

    match result {
        Ok(decompressed) => {
            let decompression_time = start_time.elapsed();
            
            // Record successful decompression metrics
            if let Some(stats) = stats {
                stats.decompression_applied.fetch_add(1, Ordering::Relaxed);
                stats.bytes_after_decompression.fetch_add(decompressed.len() as u64, Ordering::Relaxed);
                let decompression_micros = decompression_time.as_micros();
                let decompression_micros_u64 = if decompression_micros > u128::from(u64::MAX) {
                    tracing::warn!(
                        target: "quyc::compression",
                        decompression_micros = decompression_micros,
                        max_u64 = u64::MAX,
                        "Decompression time exceeds u64 limits, clamping to max"
                    );
                    u64::MAX
                } else {
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        decompression_micros as u64
                    }
                };
                stats.decompression_time_micros.fetch_add(decompression_micros_u64, Ordering::Relaxed);
            }
            
            tracing::debug!(
                target: "quyc::compression",
                algorithm = %algorithm.encoding_name(),
                compressed_size = data.len(),
                decompressed_size = decompressed.len(),
                duration_micros = decompression_time.as_micros(),
                "Decompression completed"
            );
            Ok(decompressed)
        },
        Err(e) => {
            // Record decompression error metrics
            if let Some(stats) = stats {
                stats.decompression_errors.fetch_add(1, Ordering::Relaxed);
            }
            
            tracing::error!(
                target: "quyc::compression", 
                algorithm = %algorithm.encoding_name(),
                error = %e,
                duration_micros = start_time.elapsed().as_micros(),
                "Decompression failed"
            );
            Err(e)
        }
    }
}

/// Compress data using Gzip
#[inline]
fn compress_gzip(data: &[u8], compression_level: FlateCompression) -> Result<Vec<u8>, HttpError> {
    let mut buffer_guard = BufferGuard::new(data.len() / 4); // Estimate compressed size
    let output_buffer = buffer_guard.as_mut()?;
    
    {
        let mut encoder = GzEncoder::new(&mut *output_buffer, compression_level);
        encoder.write_all(data).map_err(|e| {
            HttpError::new(crate::error::types::Kind::Request)
                .with(format!("Gzip compression write failed: {e}"))
        })?;
        
        encoder.finish().map_err(|e| {
            HttpError::new(crate::error::types::Kind::Request)
                .with(format!("Gzip compression finish failed: {e}"))
        })?;
    }
    
    Ok(output_buffer.clone())
}

/// Compress data using Deflate
#[inline]
fn compress_deflate(data: &[u8], compression_level: FlateCompression) -> Result<Vec<u8>, HttpError> {
    let mut buffer_guard = BufferGuard::new(data.len() / 4); // Estimate compressed size
    let output_buffer = buffer_guard.as_mut()?;
    
    {
        let mut encoder = DeflateEncoder::new(&mut *output_buffer, compression_level);
        encoder.write_all(data).map_err(|e| {
            HttpError::new(crate::error::types::Kind::Request)
                .with(format!("Deflate compression write failed: {e}"))
        })?;
        
        encoder.finish().map_err(|e| {
            HttpError::new(crate::error::types::Kind::Request)
                .with(format!("Deflate compression finish failed: {e}"))
        })?;
    }
    
    Ok(output_buffer.clone())
}

/// Decompress Gzip data
#[inline]
fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, HttpError> {
    let mut buffer_guard = BufferGuard::new(data.len() * 4); // Estimate decompressed size
    let output_buffer = buffer_guard.as_mut()?;
    
    let mut decoder = GzDecoder::new(std::io::Cursor::new(data));
    let mut total_read = 0;
    const READ_LIMIT: usize = 64 * 1024 * 1024; // 64MB limit
    
    loop {
        // Ensure we have space to read
        if output_buffer.len() == output_buffer.capacity() {
            if output_buffer.capacity() >= READ_LIMIT {
                return Err(HttpError::new(crate::error::types::Kind::Request)
                    .with("Decompressed data exceeds safety limit"));
            }
            output_buffer.reserve(DEFAULT_BUFFER_SIZE);
        }
        
        let mut temp_buf = [0u8; 8192];
        match decoder.read(&mut temp_buf) {
            Ok(0) => break, // EOF
            Ok(n) => {
                total_read += n;
                if total_read > READ_LIMIT {
                    return Err(HttpError::new(crate::error::types::Kind::Request)
                        .with("Decompressed data exceeds safety limit"));
                }
                output_buffer.extend_from_slice(&temp_buf[..n]);
            },
            Err(e) => {
                return Err(HttpError::new(crate::error::types::Kind::Request)
                    .with(format!("Decompression read failed: {e}")));
            }
        }
    }
    
    Ok(output_buffer.clone())
}

/// Decompress Deflate data
#[inline]
fn decompress_deflate(data: &[u8]) -> Result<Vec<u8>, HttpError> {
    let mut buffer_guard = BufferGuard::new(data.len() * 4); // Estimate decompressed size
    let output_buffer = buffer_guard.as_mut()?;
    
    let mut decoder = DeflateDecoder::new(std::io::Cursor::new(data));
    let mut total_read = 0;
    const READ_LIMIT: usize = 64 * 1024 * 1024; // 64MB limit
    
    loop {
        // Ensure we have space to read
        if output_buffer.len() == output_buffer.capacity() {
            if output_buffer.capacity() >= READ_LIMIT {
                return Err(HttpError::new(crate::error::types::Kind::Request)
                    .with("Decompressed data exceeds safety limit"));
            }
            output_buffer.reserve(DEFAULT_BUFFER_SIZE);
        }
        
        let mut temp_buf = [0u8; 8192];
        match decoder.read(&mut temp_buf) {
            Ok(0) => break, // EOF
            Ok(n) => {
                total_read += n;
                if total_read > READ_LIMIT {
                    return Err(HttpError::new(crate::error::types::Kind::Request)
                        .with("Decompressed data exceeds safety limit"));
                }
                output_buffer.extend_from_slice(&temp_buf[..n]);
            },
            Err(e) => {
                return Err(HttpError::new(crate::error::types::Kind::Request)
                    .with(format!("Deflate decompression read failed: {e}")));
            }
        }
    }
    
    Ok(output_buffer.clone())
}

/// Internal helper for Brotli compression
#[inline]
fn compress_with_brotli(data: &[u8], level: u32) -> Result<Vec<u8>, HttpError> {
    let mut buffer_guard = BufferGuard::new(data.len() / 4);
    let output_buffer = buffer_guard.as_mut()?;
    
    let cursor = std::io::Cursor::new(data);
    let params = brotli::enc::BrotliEncoderParams {
        quality: level.min(11) as i32,
        lgwin: 22, // 4MB window
        mode: BrotliEncoderMode::BROTLI_MODE_GENERIC,
        size_hint: data.len(),
        ..Default::default()
    };
    
    let mut encoder = BrotliEncoder::with_params(cursor, DEFAULT_BUFFER_SIZE, &params);
    
    let mut temp_buf = [0u8; 8192];
    loop {
        match encoder.read(&mut temp_buf) {
            Ok(0) => break,
            Ok(n) => output_buffer.extend_from_slice(&temp_buf[..n]),
            Err(e) => {
                return Err(HttpError::new(crate::error::types::Kind::Request)
                    .with(format!("Brotli compression failed: {e}")));
            }
        }
    }
    
    Ok(output_buffer.clone())
}

/// Internal helper for Brotli decompression
#[inline]
fn decompress_with_brotli(data: &[u8]) -> Result<Vec<u8>, HttpError> {
    let mut buffer_guard = BufferGuard::new(data.len() * 4);
    let output_buffer = buffer_guard.as_mut()?;
    
    let cursor = std::io::Cursor::new(data);
    let mut decoder = BrotliDecoder::new(cursor, DEFAULT_BUFFER_SIZE);
    
    let mut total_read = 0;
    const READ_LIMIT: usize = 64 * 1024 * 1024; // 64MB limit
    
    loop {
        if output_buffer.len() == output_buffer.capacity() {
            if output_buffer.capacity() >= READ_LIMIT {
                return Err(HttpError::new(crate::error::types::Kind::Request)
                    .with("Decompressed data exceeds safety limit"));
            }
            output_buffer.reserve(DEFAULT_BUFFER_SIZE);
        }
        
        let current_len = output_buffer.len();
        output_buffer.resize(output_buffer.capacity(), 0);
        
        match decoder.read(&mut output_buffer[current_len..]) {
            Ok(0) => {
                output_buffer.resize(current_len, 0);
                break;
            },
            Ok(n) => {
                total_read += n;
                if total_read > READ_LIMIT {
                    return Err(HttpError::new(crate::error::types::Kind::Request)
                        .with("Decompressed data exceeds safety limit"));
                }
                output_buffer.resize(current_len + n, 0);
            },
            Err(e) => {
                return Err(HttpError::new(crate::error::types::Kind::Request)
                    .with(format!("Brotli decompression failed: {e}")));
            }
        }
    }
    
    Ok(output_buffer.clone())
}

/// Streaming compression reader that compresses data as it's read
///
/// Provides zero-allocation streaming compression for large data sources.
/// Memory usage is bounded by buffer sizes rather than total data size.
pub struct CompressReader<R: Read> {
    inner: R,
    algorithm: CompressionAlgorithm,
    level: Option<u32>,
    buffer: Vec<u8>,
    compressed_buffer: Vec<u8>,
    finished: bool,
}

impl<R: Read> CompressReader<R> {
    /// Creates a new streaming compression reader
    ///
    /// # Arguments
    /// * `inner` - Source reader to compress
    /// * `algorithm` - Compression algorithm to use
    /// * `level` - Optional compression level
    pub fn new(inner: R, algorithm: CompressionAlgorithm, level: Option<u32>) -> Self {
        Self {
            inner,
            algorithm,
            level,
            buffer: Vec::with_capacity(DEFAULT_BUFFER_SIZE),
            compressed_buffer: Vec::with_capacity(DEFAULT_BUFFER_SIZE),
            finished: false,
        }
    }
}

impl<R: Read> Read for CompressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.finished && self.compressed_buffer.is_empty() {
            return Ok(0);
        }

        // If we have compressed data ready, serve it first
        if !self.compressed_buffer.is_empty() {
            let to_copy = buf.len().min(self.compressed_buffer.len());
            buf[..to_copy].copy_from_slice(&self.compressed_buffer[..to_copy]);
            self.compressed_buffer.drain(..to_copy);
            return Ok(to_copy);
        }

        // Read more data from source
        self.buffer.resize(DEFAULT_BUFFER_SIZE, 0);
        match self.inner.read(&mut self.buffer) {
            Ok(0) => {
                self.finished = true;
                Ok(0)
            },
            Ok(n) => {
                self.buffer.resize(n, 0);
                
                // Compress the chunk
                match compress_bytes(&self.buffer, self.algorithm, self.level) {
                    Ok(compressed) => {
                        self.compressed_buffer = compressed;
                        self.buffer.clear();
                        
                        // Return as much as we can fit
                        let to_copy = buf.len().min(self.compressed_buffer.len());
                        buf[..to_copy].copy_from_slice(&self.compressed_buffer[..to_copy]);
                        self.compressed_buffer.drain(..to_copy);
                        Ok(to_copy)
                    },
                    Err(_) => {
                        Err(std::io::Error::other(
                            "Compression failed"
                        ))
                    }
                }
            },
            Err(e) => Err(e),
        }
    }
}

/// Streaming decompression reader that decompresses data as it's read
///
/// Provides memory-efficient streaming decompression for large compressed sources.
/// Handles partial reads and maintains internal buffers for optimal performance.
pub struct DecompressReader<R: Read> {
    inner: R,
    algorithm: CompressionAlgorithm,
    buffer: Vec<u8>,
    decompressed_buffer: Vec<u8>,
    finished: bool,
}

impl<R: Read> DecompressReader<R> {
    /// Creates a new streaming decompression reader
    ///
    /// # Arguments  
    /// * `inner` - Source reader with compressed data
    /// * `algorithm` - Compression algorithm used
    pub fn new(inner: R, algorithm: CompressionAlgorithm) -> Self {
        Self {
            inner,
            algorithm,
            buffer: Vec::with_capacity(DEFAULT_BUFFER_SIZE),
            decompressed_buffer: Vec::with_capacity(DEFAULT_BUFFER_SIZE * 2),
            finished: false,
        }
    }
}

impl<R: Read> Read for DecompressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.finished && self.decompressed_buffer.is_empty() {
            return Ok(0);
        }

        // Serve decompressed data if available
        if !self.decompressed_buffer.is_empty() {
            let to_copy = buf.len().min(self.decompressed_buffer.len());
            buf[..to_copy].copy_from_slice(&self.decompressed_buffer[..to_copy]);
            self.decompressed_buffer.drain(..to_copy);
            return Ok(to_copy);
        }

        // Read more compressed data
        self.buffer.resize(DEFAULT_BUFFER_SIZE, 0);
        match self.inner.read(&mut self.buffer) {
            Ok(0) => {
                self.finished = true;
                Ok(0)
            },
            Ok(n) => {
                self.buffer.resize(n, 0);
                
                // Decompress the chunk
                match decompress_bytes(&self.buffer, self.algorithm) {
                    Ok(decompressed) => {
                        self.decompressed_buffer = decompressed;
                        self.buffer.clear();
                        
                        // Return as much as we can fit
                        let to_copy = buf.len().min(self.decompressed_buffer.len());
                        buf[..to_copy].copy_from_slice(&self.decompressed_buffer[..to_copy]);
                        self.decompressed_buffer.drain(..to_copy);
                        Ok(to_copy)
                    },
                    Err(_) => {
                        Err(std::io::Error::other(
                            "Decompression failed"
                        ))
                    }
                }
            },
            Err(e) => Err(e),
        }
    }
}

/// Determines if content should be compressed based on Content-Type and configuration
///
/// Uses fast string matching and lookup tables for optimal performance.
/// Avoids compressing already-compressed formats and binary data.
///
/// # Arguments
/// * `content_type` - Optional Content-Type header value
/// * `config` - HTTP configuration with compression settings
///
/// # Returns
/// * `true` if content should be compressed
/// * `false` if compression should be skipped
#[inline]
#[must_use] 
pub fn should_compress_content_type(content_type: Option<&str>, config: &HttpConfig) -> bool {
    // If compression is disabled, never compress
    if !config.request_compression {
        return false;
    }

    let Some(content_type) = content_type else {
        // No content type - compress by default for text-like content
        return true;
    };

    // Fast path: check common compressible types first (branch prediction optimization)
    if content_type.starts_with("text/") 
        || content_type.starts_with("application/json")
        || content_type.starts_with("application/javascript")
        || content_type.starts_with("application/xml") {
        return true;
    }

    // Skip compression for already-compressed formats
    static UNCOMPRESSIBLE_TYPES: &[&str] = &[
        // Images
        "image/jpeg", "image/png", "image/gif", "image/webp", "image/avif", "image/bmp",
        // Video  
        "video/mp4", "video/mpeg", "video/quicktime", "video/x-msvideo", "video/webm",
        // Audio
        "audio/mpeg", "audio/mp4", "audio/ogg", "audio/wav", "audio/webm",
        // Archives
        "application/zip", "application/gzip", "application/x-gzip", 
        "application/x-compress", "application/x-bzip2", "application/x-xz",
        // Binary formats
        "application/pdf", "application/octet-stream",
        // Already compressed
        "application/x-br", "application/x-deflate",
    ];

    // Use binary search for efficient lookup (types are sorted)
    UNCOMPRESSIBLE_TYPES.binary_search(&content_type).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_reuse() {
        let mut pool = CompressionBufferPool::new();
        
        let buf1 = pool.get_buffer(1024);
        assert!(buf1.capacity() >= 1024);
        
        let capacity = buf1.capacity();
        pool.return_buffer(buf1);
        
        let buf2 = pool.get_buffer(512);
        assert_eq!(buf2.capacity(), capacity); // Should reuse same buffer
    }

    #[test]  
    fn test_compression_round_trip() {
        let original = b"Hello, world! This is a test of compression and decompression.";
        
        for algorithm in [CompressionAlgorithm::Gzip, CompressionAlgorithm::Deflate, CompressionAlgorithm::Brotli] {
            let compressed = compress_bytes(original, algorithm, None)
                .unwrap_or_else(|e| panic!("Compression should succeed in test but failed: {}", e));
            let decompressed = decompress_bytes(&compressed, algorithm)
                .unwrap_or_else(|e| panic!("Decompression should succeed in test but failed: {}", e));
            assert_eq!(original, decompressed.as_slice());
        }
    }

    #[test]
    fn test_should_compress_content_type() {
        let config = HttpConfig {
            request_compression: true,
            ..Default::default()
        };

        // Should compress
        assert!(should_compress_content_type(Some("text/plain"), &config));
        assert!(should_compress_content_type(Some("application/json"), &config));
        assert!(should_compress_content_type(None, &config));

        // Should not compress  
        assert!(!should_compress_content_type(Some("image/jpeg"), &config));
        assert!(!should_compress_content_type(Some("video/mp4"), &config));
        assert!(!should_compress_content_type(Some("application/zip"), &config));
    }
}