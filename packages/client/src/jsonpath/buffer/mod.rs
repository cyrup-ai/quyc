//! Zero-allocation buffer management for JSON streaming
//!
//! This module provides high-performance buffer management for streaming JSON parsing.
//! Uses memory pools, zero-copy techniques, and efficient chunk aggregation to minimize
//! allocations and maximize throughput.

mod capacity;
mod core;
mod reader;

pub use core::StreamBuffer;

use bytes::Buf;
pub use reader::BufferReader;

use crate::telemetry::jsonpath::{BufferStats, CapacityStats};

impl StreamBuffer {
    /// Mark bytes as consumed after successful JSON parsing
    ///
    /// # Arguments
    ///
    /// * `bytes_consumed` - Number of bytes that were successfully parsed
    ///
    /// # Performance
    ///
    /// Uses efficient buffer advance operations to avoid copying remaining data.
    pub fn consume(&mut self, bytes_consumed: usize) {
        if bytes_consumed <= self.buffer.len() {
            self.buffer.advance(bytes_consumed);
            self.last_boundary = 0; // Reset boundary tracking

            // Shrink buffer if it's grown too large and is mostly empty
            self.capacity_manager.maybe_shrink(&mut self.buffer);
        }
    }

    /// Get current buffer size in bytes
    #[inline]
    #[must_use] 
    pub fn current_size(&self) -> usize {
        self.buffer.len()
    }

    /// Get current buffer capacity in bytes
    #[inline]
    #[must_use] 
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Get current buffer contents as bytes slice
    ///
    /// Returns a byte slice view of the current buffer contents.
    /// Useful for direct parsing operations.
    #[inline]
    #[must_use] 
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[..]
    }

    /// Get total bytes processed since creation
    #[inline]
    #[must_use] 
    pub fn total_bytes_processed(&self) -> u64 {
        self.total_processed
    }

    /// Check if buffer has enough data for JSON parsing attempt
    ///
    /// # Arguments
    ///
    /// * `min_bytes` - Minimum bytes needed for parsing attempt
    ///
    /// Returns `true` if buffer contains at least `min_bytes` of data.
    #[inline]
    #[must_use] 
    pub fn has_data(&self, min_bytes: usize) -> bool {
        self.buffer.len() >= min_bytes
    }

    /// Find likely JSON object boundaries in buffer
    ///
    /// Scans for complete JSON objects (balanced braces) to optimize parsing attempts.
    /// Returns positions of potential object boundaries for batch processing.
    #[must_use] 
    pub fn find_object_boundaries(&self) -> Vec<usize> {
        let mut boundaries = Vec::new();
        let mut brace_depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        let data = &self.buffer[..];
        for (i, &byte) in data.iter().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match byte {
                b'\\' if in_string => escape_next = true,
                b'"' => in_string = !in_string,
                b'{' if !in_string => brace_depth += 1,
                b'}' if !in_string => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        boundaries.push(i + 1); // Position after closing brace
                    }
                }
                _ => {}
            }
        }

        boundaries
    }

    /// Get byte at specific position in buffer
    ///
    /// Returns None if position is beyond buffer length
    #[inline]
    #[must_use] 
    pub fn get_byte_at(&self, position: usize) -> Option<u8> {
        self.buffer.get(position).copied()
    }

    /// Get current buffer length for bounds checking
    #[inline]
    #[must_use] 
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    #[inline]
    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get buffer utilization statistics for monitoring
    #[must_use] 
    pub fn stats(&self) -> BufferStats {
        BufferStats {
            current_size: self.buffer.len(),
            capacity: self.buffer.capacity(),
            total_processed: self.total_processed,
            // Precision loss acceptable for buffer utilization statistics
            #[allow(clippy::cast_precision_loss)]
            utilization_ratio: self.buffer.len() as f64 / self.buffer.capacity() as f64,
        }
    }

    /// Get detailed capacity management statistics for advanced monitoring
    #[must_use] 
    pub fn capacity_stats(&self) -> CapacityStats {
        CapacityStats {
            initial_capacity: self.capacity_manager.initial_capacity,
            max_capacity: self.capacity_manager.max_capacity,
            current_capacity: self.buffer.capacity(),
            growth_operations: self.capacity_manager.growth_operations,
            last_shrink_size: self.capacity_manager.last_shrink_size,
            can_shrink: self.capacity_manager.growth_operations
                >= self.capacity_manager.hysteresis_threshold,
        }
    }
}

impl Default for StreamBuffer {
    fn default() -> Self {
        Self::new()
    }
}


