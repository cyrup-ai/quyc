//! `BufferReader` implementation for `serde_json` integration
//!
//! Provides `std::io::Read` implementation for seamless integration with
//! `serde_json` streaming deserializers while maintaining zero-allocation principles.

use std::io::{self, Read};

use serde_json::de::IoRead;

/// Buffer reader implementing `std::io::Read` for `serde_json` integration
pub struct BufferReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> BufferReader<'a> {
    /// Create a new buffer reader from a byte slice
    pub(super) fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    /// Create an `IoRead` wrapper for use with `serde_json::StreamDeserializer`
    ///
    /// This method returns an `IoRead` wrapper that can be used directly with
    /// `serde_json`'s streaming deserializer while maintaining zero-allocation principles.
    #[must_use] 
    pub fn into_io_read(self) -> IoRead<Self> {
        IoRead::new(self)
    }

    /// Get current position for debugging and monitoring
    #[inline]
    #[must_use] 
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get remaining bytes available for reading
    #[inline]
    #[must_use] 
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    /// Check if reader has reached end of buffer
    #[inline]
    #[must_use] 
    pub fn is_eof(&self) -> bool {
        self.position >= self.buffer.len()
    }
}

impl Read for BufferReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = &self.buffer[self.position..];
        let to_copy = std::cmp::min(buf.len(), remaining.len());

        if to_copy > 0 {
            buf[..to_copy].copy_from_slice(&remaining[..to_copy]);
            self.position += to_copy;
        }

        Ok(to_copy)
    }
}
