//! JSONPath error handling module
//!
//! This module provides comprehensive error types and utilities for JSONPath operations
//! in the HTTP3 streaming framework. It includes error types, constructors, conversions,
//! and utilities for error handling.

pub mod constructors;
mod conversions;
mod types;

// Re-export all error types and utilities
pub use constructors::{
    buffer_error, deserialization_error, invalid_expression_error, json_parse_error, stream_error,
};
// pub use conversions::{FromIo, FromSerde}; // These types don't exist
pub use types::{ErrorKind, JsonPathError, JsonPathResult};