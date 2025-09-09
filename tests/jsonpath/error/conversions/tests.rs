//! Tests for error conversion implementations
//!
//! Comprehensive test coverage for all conversion traits and helper methods.

use std::io::{Error as IoError, ErrorKind};
use quyc_client::jsonpath::error::types::JsonPathError;
use quyc_client::jsonpath::error::conversions::helpers::IntoJsonPathError;

#[test]
fn test_serde_json_error_conversion() {
    let json_str = r#"{"invalid": json,}"#;
    let serde_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let jsonpath_error: JsonPathError = serde_error.into();

    match jsonpath_error {
        JsonPathError::JsonParseError { message, .. } => {
            assert!(message.contains("expected"));
        }
        JsonPathError::Deserialization(message) => {
            assert!(message.contains("expected"));
        }
        _ => panic!("Expected JsonParseError or Deserialization variant"),
    }
}

#[test]
fn test_io_error_conversion_recoverable() {
    let io_error = IoError::new(ErrorKind::Interrupted, "operation interrupted");
    let jsonpath_error: JsonPathError = io_error.into();

    match jsonpath_error {
        JsonPathError::StreamError {
            message,
            state,
            recoverable,
        } => {
            assert!(message.contains("interrupted"));
            assert_eq!(state, "io_operation");
            assert_eq!(recoverable, true);
        }
        _ => panic!("Expected StreamError variant"),
    }
}

#[test]
fn test_io_error_conversion_unrecoverable() {
    let io_error = IoError::new(ErrorKind::PermissionDenied, "access denied");
    let jsonpath_error: JsonPathError = io_error.into();

    match jsonpath_error {
        JsonPathError::StreamError {
            message,
            state,
            recoverable,
        } => {
            assert!(message.contains("access denied"));
            assert_eq!(state, "io_operation");
            assert_eq!(recoverable, false);
        }
        _ => panic!("Expected StreamError variant"),
    }
}

#[test]
fn test_fmt_error_conversion() {
    let fmt_error = std::fmt::Error;
    let jsonpath_error: JsonPathError = fmt_error.into();

    match jsonpath_error {
        JsonPathError::StreamError {
            state, recoverable, ..
        } => {
            assert_eq!(state, "formatting");
            assert_eq!(recoverable, false);
        }
        _ => panic!("Expected StreamError variant"),
    }
}

#[test]
fn test_utf8_error_conversion() {
    let invalid_utf8 = b"\xFF\xFE";
    let utf8_error = std::str::from_utf8(invalid_utf8).unwrap_err();
    let jsonpath_error: JsonPathError = utf8_error.into();

    match jsonpath_error {
        JsonPathError::JsonParseError {
            message,
            offset,
            context,
        } => {
            assert!(message.contains("invalid UTF-8"));
            assert_eq!(offset, 0);
            assert_eq!(context, "UTF-8 validation");
        }
        _ => panic!("Expected JsonParseError variant"),
    }
}

#[test]
fn test_from_utf8_error_conversion() {
    let invalid_utf8 = vec![0xFF, 0xFE];
    let from_utf8_error = String::from_utf8(invalid_utf8).unwrap_err();
    let jsonpath_error: JsonPathError = from_utf8_error.into();

    match jsonpath_error {
        JsonPathError::JsonParseError {
            message,
            offset,
            context,
        } => {
            assert!(message.contains("invalid UTF-8"));
            assert_eq!(offset, 0);
            assert_eq!(context, "string conversion");
        }
        _ => panic!("Expected JsonParseError variant"),
    }
}

#[test]
fn test_parse_int_error_conversion() {
    let parse_error = "not_a_number".parse::<i32>().unwrap_err();
    let jsonpath_error: JsonPathError = parse_error.into();

    match jsonpath_error {
        JsonPathError::DeserializationError {
            message,
            json_fragment,
            target_type,
        } => {
            assert!(message.contains("invalid digit"));
            assert_eq!(json_fragment, "number");
            assert_eq!(target_type, "integer");
        }
        _ => panic!("Expected DeserializationError variant"),
    }
}

#[test]
fn test_parse_float_error_conversion() {
    let parse_error = "not_a_float".parse::<f64>().unwrap_err();
    let jsonpath_error: JsonPathError = parse_error.into();

    match jsonpath_error {
        JsonPathError::DeserializationError {
            message,
            json_fragment,
            target_type,
        } => {
            assert!(message.contains("invalid float"));
            assert_eq!(json_fragment, "number");
            assert_eq!(target_type, "float");
        }
        _ => panic!("Expected DeserializationError variant"),
    }
}

#[test]
fn test_into_jsonpath_error_trait() {
    let result: Result<i32, std::io::Error> =
        Err(IoError::new(ErrorKind::NotFound, "file not found"));
    let jsonpath_result = result.into_jsonpath_error("file_reading");

    assert!(jsonpath_result.is_err());
    match jsonpath_result.unwrap_err() {
        JsonPathError::StreamError { message, state, .. } => {
            assert!(message.contains("file not found"));
            assert_eq!(state, "file_reading");
        }
        _ => panic!("Expected StreamError variant"),
    }
}

#[test]
fn test_map_jsonpath_error_trait() {
    let result: Result<i32, &str> = Err("custom error");
    let jsonpath_result = result.map_jsonpath_error(|| {
        JsonPathError::unsupported_feature("test feature", None::<String>)
    });

    assert!(jsonpath_result.is_err());
    match jsonpath_result.unwrap_err() {
        JsonPathError::UnsupportedFeature { feature, .. } => {
            assert_eq!(feature, "test feature");
        }
        _ => panic!("Expected UnsupportedFeature variant"),
    }
}

#[test]
fn test_from_serde_with_context() {
    let json_str = r#"{"key": "not_a_number"}"#;
    let serde_error = serde_json::from_str::<i32>(json_str).unwrap_err();
    let jsonpath_error = JsonPathError::from_serde_with_context(
        serde_error,
        r#"{"key": "not_a_number"}"#,
        "i32",
    );

    match jsonpath_error {
        JsonPathError::DeserializationError {
            message,
            json_fragment,
            target_type,
        } => {
            assert!(message.contains("invalid type"));
            assert_eq!(json_fragment, r#"{"key": "not_a_number"}"#);
            assert_eq!(target_type, "i32");
        }
        _ => panic!("Expected DeserializationError variant"),
    }
}

#[test]
fn test_from_io_with_context() {
    let io_error = IoError::new(ErrorKind::UnexpectedEof, "unexpected end");
    let jsonpath_error = JsonPathError::from_io_with_context(io_error, "stream_processing");

    match jsonpath_error {
        JsonPathError::StreamError {
            message,
            state,
            recoverable,
        } => {
            assert!(message.contains("unexpected end"));
            assert_eq!(state, "stream_processing");
            assert_eq!(recoverable, false);
        }
        _ => panic!("Expected StreamError variant"),
    }
}