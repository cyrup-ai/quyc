//! Core StreamBuffer implementation for zero-allocation JSON streaming
//!
//! Contains the main StreamBuffer struct and its fundamental operations
//! for efficient chunk management and JSON boundary detection.

use bytes::{Bytes, BytesMut};

use super::{capacity::CapacityManager, reader::BufferReader};
// Removed unused imports

/// Zero-allocation streaming buffer with efficient chunk management
///
/// Optimized for JSON parsing workflows where data arrives in chunks and needs
/// to be parsed incrementally. Uses memory pools and zero-copy techniques.
#[derive(Debug)]
pub struct StreamBuffer {
    /// Main buffer for accumulating incoming chunks
    pub(super) buffer: BytesMut,
    /// Total bytes processed (for statistics)
    pub(super) total_processed: u64,
    /// Position of last complete JSON object boundary
    pub(super) last_boundary: usize,
    /// Buffer capacity management
    pub(super) capacity_manager: CapacityManager,
}

impl StreamBuffer {
    /// Create new stream buffer with specified initial capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Initial buffer capacity in bytes (recommended: 8KB-64KB)
    ///
    /// # Performance
    ///
    /// Initial capacity should be sized based on expected chunk sizes and JSON object sizes.
    /// Larger capacities reduce reallocations but increase memory usage.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            total_processed: 0,
            last_boundary: 0,
            capacity_manager: CapacityManager::new(capacity),
        }
    }

    /// Create buffer with default capacity optimized for HTTP responses
    pub fn new() -> Self {
        Self::with_capacity(8192) // 8KB default
    }

    /// Append incoming HTTP chunk to buffer
    ///
    /// # Arguments
    ///
    /// * `chunk` - Incoming bytes from HTTP response stream
    ///
    /// # Performance
    ///
    /// Uses zero-copy techniques when possible. Automatically manages buffer
    /// capacity and growth to minimize reallocations.
    pub fn append_chunk(&mut self, chunk: Bytes) {
        self.total_processed += chunk.len() as u64;

        // Check if we need to grow the buffer
        if self.buffer.capacity() - self.buffer.len() < chunk.len() {
            self.capacity_manager
                .ensure_capacity(&mut self.buffer, chunk.len());
        }

        // Zero-copy append when possible
        self.buffer.extend_from_slice(&chunk);
    }

    /// Get a reader for the current buffer contents
    ///
    /// Returns a reader that implements `std::io::Read` for use with serde_json::StreamDeserializer.
    /// The reader tracks position and handles partial reads correctly.
    pub fn reader(&mut self) -> BufferReader<'_> {
        BufferReader::new(&self.buffer[..])
    }

    /// Create a reader for the current buffer contents (alias for reader)
    ///
    /// Returns a reader that implements both `std::io::Read` and `serde_json::de::Read`
    /// for seamless integration with serde_json streaming deserializers.
    #[inline]
    pub fn create_reader(&mut self) -> BufferReader<'_> {
        self.reader()
    }

    /// Clear buffer and reset state
    ///
    /// Useful for error recovery or when switching to a new response stream.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.last_boundary = 0;
        self.capacity_manager.reset();
    }
}
