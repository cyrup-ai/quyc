//! Helper functions for common error scenarios
//!
//! Convenience methods that wrap core constructors for frequently used error patterns.

use super::super::types::JsonPathError;

/// Helper functions for common error scenarios
impl JsonPathError {
    /// Creates an error for invalid JSONPath syntax
    pub fn invalid_syntax(expression: &str, position: usize) -> Self {
        Self::invalid_expression(expression, "invalid JSONPath syntax", Some(position))
    }

    /// Creates an error for unsupported JSONPath operators
    pub fn unsupported_operator(operator: &str) -> Self {
        Self::unsupported_feature(
            format!("operator '{}'", operator),
            Some("check JSONPath specification for supported operators"),
        )
    }

    /// Creates an error for JSON parsing failures with minimal context
    pub fn parse_failure(message: &str, offset: usize) -> Self {
        Self::json_parse_error(message, offset, "JSON parsing")
    }

    /// Creates an error for stream buffer overflow
    pub fn buffer_overflow(requested: usize, available: usize) -> Self {
        Self::buffer_error("overflow", requested, available)
    }

    /// Creates an error for unrecoverable stream processing failures
    pub fn stream_failure(message: &str, state: &str) -> Self {
        Self::stream_error(message, state, false)
    }

    /// Creates an error for recoverable stream processing issues
    pub fn stream_warning(message: &str, state: &str) -> Self {
        Self::stream_error(message, state, true)
    }
}
