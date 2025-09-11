//! JSON Path Error Types
//!
//! Core error types for JSON path processing and streaming operations.

use std::error::Error;
use std::fmt;

/// JSON Path processing error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// Invalid JSON syntax
    InvalidJson,
    /// Invalid JSON path expression
    InvalidPath,
    /// IO operation failed
    IoError,
    /// Serialization/deserialization error
    SerdeError,
    /// Deserialization error
    Deserialization,
    /// Generic processing error
    ProcessingError,
}

/// Main JSON Path error type
#[derive(Debug, Clone)]
pub struct JsonPathError {
    pub kind: ErrorKind,
    pub message: String,
}

impl fmt::Display for JsonPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON Path Error: {}", self.message)
    }
}

impl Error for JsonPathError {}

/// Result type for JSON Path operations
pub type JsonPathResult<T> = Result<T, JsonPathError>;

impl JsonPathError {
    #[must_use] 
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }

    pub fn invalid_json(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidJson, msg.into())
    }

    pub fn invalid_path(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidPath, msg.into())
    }

    pub fn io_error(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::IoError, msg.into())
    }

    pub fn serde_error(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::SerdeError, msg.into())
    }

    pub fn processing_error(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::ProcessingError, msg.into())
    }

    pub fn invalid_index(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidPath, msg.into())
    }

    #[must_use] 
    pub fn invalid_expression_simple(kind: &str, msg: &str, position: Option<usize>) -> Self {
        let message = match position {
            Some(pos) => format!("{kind} at position {pos}: {msg}"),
            None => format!("{kind}: {msg}"),
        };
        Self::new(ErrorKind::InvalidPath, message)
    }

    #[must_use] 
    pub fn buffer_underflow() -> Self {
        Self::new(ErrorKind::ProcessingError, "Buffer underflow during JSON parsing".into())
    }

    #[must_use] 
    pub fn unexpected_byte(byte: char) -> Self {
        Self::new(ErrorKind::InvalidJson, format!("Unexpected byte: '{byte}'"))
    }

    #[must_use] 
    pub fn invalid_utf8() -> Self {
        Self::new(ErrorKind::InvalidJson, "Invalid UTF-8 sequence".into())
    }

    #[must_use] 
    pub fn unexpected_end_of_input() -> Self {
        Self::new(ErrorKind::InvalidJson, "Unexpected end of input".into())
    }

    #[must_use] 
    pub fn invalid_number() -> Self {
        Self::new(ErrorKind::InvalidJson, "Invalid number format".into())
    }
}
