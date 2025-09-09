//! Standard library type conversions to JsonPathError
//!
//! From trait implementations for converting common std types into JsonPathError variants.

use super::super::types::JsonPathError;

/// Conversion from serde_json::Error to JsonPathError
impl From<serde_json::Error> for JsonPathError {
    fn from(error: serde_json::Error) -> Self {
        // Extract useful information from serde_json::Error
        let message = error.to_string();

        // Try to extract line/column information if available
        let line = error.line();
        if line > 0 {
            let context = format!("line {}, column {}", line, error.column());
            JsonPathError::new(
                super::super::types::ErrorKind::InvalidJson,
                format!("{} ({})", message, context),
            )
        } else {
            // Fallback to simple deserialization error
            JsonPathError::new(super::super::types::ErrorKind::SerdeError, message)
        }
    }
}

/// Conversion from std::io::Error to JsonPathError
impl From<std::io::Error> for JsonPathError {
    fn from(error: std::io::Error) -> Self {
        JsonPathError::new(
            super::super::types::ErrorKind::IoError,
            format!("IO operation error: {}", error.to_string()),
        )
    }
}

/// Conversion from std::fmt::Error to JsonPathError
impl From<std::fmt::Error> for JsonPathError {
    fn from(error: std::fmt::Error) -> Self {
        JsonPathError::new(
            super::super::types::ErrorKind::ProcessingError,
            format!("Formatting error: {}", error.to_string()),
        )
    }
}

/// Conversion from std::str::Utf8Error to JsonPathError
impl From<std::str::Utf8Error> for JsonPathError {
    fn from(error: std::str::Utf8Error) -> Self {
        JsonPathError::new(
            super::super::types::ErrorKind::InvalidJson,
            format!(
                "invalid UTF-8 sequence: {} at offset {}",
                error,
                error.valid_up_to()
            ),
        )
    }
}

/// Conversion from std::string::FromUtf8Error to JsonPathError
impl From<std::string::FromUtf8Error> for JsonPathError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        let utf8_error = error.utf8_error();
        JsonPathError::new(
            super::super::types::ErrorKind::InvalidJson,
            format!(
                "invalid UTF-8 in string conversion: {} at offset {}",
                utf8_error,
                utf8_error.valid_up_to()
            ),
        )
    }
}

/// Conversion from std::num::ParseIntError to JsonPathError
impl From<std::num::ParseIntError> for JsonPathError {
    fn from(error: std::num::ParseIntError) -> Self {
        JsonPathError::new(
            super::super::types::ErrorKind::SerdeError,
            format!("Parse integer error: {}", error.to_string()),
        )
    }
}

/// Conversion from std::num::ParseFloatError to JsonPathError
impl From<std::num::ParseFloatError> for JsonPathError {
    fn from(error: std::num::ParseFloatError) -> Self {
        JsonPathError::new(
            super::super::types::ErrorKind::SerdeError,
            format!("Parse float error: {}", error.to_string()),
        )
    }
}
