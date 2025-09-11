//! Error constructor functions module
//!
//! Provides convenient factory functions for creating `JSONPath` error types
//! with proper context and formatting.

mod core;
mod helpers;

// Re-export all constructor functions for public API compatibility

use super::types::JsonPathError;

// Function aliases for backward compatibility
pub fn invalid_expression_error(
    expression: impl Into<String>,
    reason: impl Into<String>,
    position: Option<usize>,
) -> JsonPathError {
    JsonPathError::invalid_expression(expression, reason, position)
}

pub fn deserialization_error(
    message: impl Into<String>,
    json_fragment: impl Into<String>,
    target_type: &'static str,
) -> JsonPathError {
    JsonPathError::deserialization_error(message, json_fragment, target_type)
}

pub fn stream_error(
    message: impl Into<String>,
    state: impl Into<String>,
    recoverable: bool,
) -> JsonPathError {
    JsonPathError::stream_error(message, state, recoverable)
}

pub fn buffer_error(
    operation: impl Into<String>,
    requested_size: usize,
    available_capacity: usize,
) -> JsonPathError {
    JsonPathError::buffer_error(operation, requested_size, available_capacity)
}

pub fn json_parse_error(
    message: impl Into<String>,
    position: usize,
    context: impl Into<String>,
) -> JsonPathError {
    JsonPathError::json_parse_error(message, position, context)
}
