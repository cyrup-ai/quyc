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
    #[must_use] 
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            buffer: Vec::new(),
            max_size,
        }
    }

    /// Append bytes to buffer with size checking
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError` if:
    /// - Appending data would exceed maximum buffer size limit
    /// - Memory allocation fails during buffer expansion
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
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError` if:
    /// - UTF-8 validation fails and recovery strategy cannot handle invalid bytes
    /// - Memory allocation fails during string conversion
    /// - Buffer contains unrecoverable invalid UTF-8 sequences
    #[inline]
    pub fn to_string(&self, strategy: Utf8RecoveryStrategy) -> JsonPathResult<String> {
        Utf8Handler::validate_utf8_with_recovery(&self.buffer, strategy)
    }

    /// Get current buffer size
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

    /// Clear the buffer
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}
