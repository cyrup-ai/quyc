//! Utility methods and support types for `JsonArrayStream`
//!
//! Contains helper methods, statistics tracking, and support types
//! for `JSONPath` streaming functionality.

use super::core::JsonArrayStream;

impl<T> JsonArrayStream<T> {
    /// Check if the stream has completed processing
    ///
    /// Returns `true` when the entire JSON structure has been parsed and all matching
    /// array elements have been yielded.
    #[inline]
    #[must_use] 
    pub fn is_complete(&self) -> bool {
        self.state.is_complete()
    }

    /// Get current parsing statistics for monitoring and debugging
    ///
    /// Returns metrics including bytes processed, objects yielded, and parsing errors.
    #[must_use] 
    pub fn stats(&self) -> StreamStats {
        StreamStats {
            bytes_processed: self.buffer.total_bytes_processed(),
            objects_yielded: self.state.objects_yielded(),
            parse_errors: self.state.parse_errors(),
            buffer_size: self.buffer.current_size(),
        }
    }

    /// Get the `JSONPath` expression string
    ///
    /// Returns the original `JSONPath` expression used to create this stream processor.
    #[must_use]
    pub fn jsonpath_expr(&self) -> &str {
        self.path_expression.original()
    }
}

/// Streaming performance and debugging statistics
#[derive(Debug, Clone, Copy)]
pub struct StreamStats {
    /// Total bytes processed from HTTP response
    pub bytes_processed: u64,
    /// Number of JSON objects successfully deserialized
    pub objects_yielded: u64,
    /// Count of recoverable parsing errors encountered
    pub parse_errors: u64,
    /// Current internal buffer size in bytes
    pub buffer_size: usize,
}
