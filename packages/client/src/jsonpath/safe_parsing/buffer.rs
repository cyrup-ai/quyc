//! Memory-safe string buffer with size limits
//!
//! Provides controlled buffer operations that prevent memory exhaustion
//! while maintaining efficient string building capabilities.

use super::utf8::{Utf8Handler, Utf8RecoveryStrategy};
use crate::jsonpath::error::{JsonPathResult, buffer_error};

/// Memory-safe string buffer with size limits
pub struct SafeStringBuffer {
    buffer: Vec<u8>,
    max_size: usize,
}

impl SafeStringBuffer {
    /// Create new safe string buffer with size limit
    #[inline]
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            buffer: Vec::new(),
            max_size,
        }
    }

    /// Append bytes to buffer with size checking
    #[inline]
    pub fn append(&mut self, data: &[u8]) -> JsonPathResult<()> {
        if self.buffer.len() + data.len() > self.max_size {
            return Err(buffer_error(
                "string buffer append",
                data.len(),
                self.max_size - self.buffer.len(),
            ));
        }

        self.buffer.extend_from_slice(data);
        Ok(())
    }

    /// Convert buffer to UTF-8 string with recovery
    #[inline]
    pub fn to_string(&self, strategy: Utf8RecoveryStrategy) -> JsonPathResult<String> {
        Utf8Handler::validate_utf8_with_recovery(&self.buffer, strategy)
    }

    /// Get current buffer size
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clear the buffer
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}
