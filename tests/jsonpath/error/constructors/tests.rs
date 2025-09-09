//! Tests for error constructor functions
//!
//! Comprehensive test coverage for all error factory methods and helpers.

use quyc_client::jsonpath::error::types::JsonPathError;

#[test]
fn test_invalid_expression_constructor() {
    let error = JsonPathError::invalid_expression("$.test[", "unclosed bracket", Some(7));

    match error {
        JsonPathError::InvalidExpression {
            expression,
            reason,
            position,
        } => {
            assert_eq!(expression, "$.test[");
            assert_eq!(reason, "unclosed bracket");
            assert_eq!(position, Some(7));
        }
        _ => panic!("Expected InvalidExpression variant"),
    }
}

#[test]
fn test_json_parse_error_constructor() {
    let error = JsonPathError::json_parse_error("expected comma", 42, "parsing array");

    match error {
        JsonPathError::JsonParseError {
            message,
            offset,
            context,
        } => {
            assert_eq!(message, "expected comma");
            assert_eq!(offset, 42);
            assert_eq!(context, "parsing array");
        }
        _ => panic!("Expected JsonParseError variant"),
    }
}

#[test]
fn test_deserialization_error_constructor() {
    let error =
        JsonPathError::deserialization_error("type mismatch", r#"{"key": "value"}"#, "i32");

    match error {
        JsonPathError::DeserializationError {
            message,
            json_fragment,
            target_type,
        } => {
            assert_eq!(message, "type mismatch");
            assert_eq!(json_fragment, r#"{"key": "value"}"#);
            assert_eq!(target_type, "i32");
        }
        _ => panic!("Expected DeserializationError variant"),
    }
}

#[test]
fn test_stream_error_constructor() {
    let error = JsonPathError::stream_error("buffer overflow", "processing", true);

    match error {
        JsonPathError::StreamError {
            message,
            state,
            recoverable,
        } => {
            assert_eq!(message, "buffer overflow");
            assert_eq!(state, "processing");
            assert_eq!(recoverable, true);
        }
        _ => panic!("Expected StreamError variant"),
    }
}

#[test]
fn test_buffer_error_constructor() {
    let error = JsonPathError::buffer_error("allocation", 1024, 512);

    match error {
        JsonPathError::BufferError {
            operation,
            requested_size,
            available_capacity,
        } => {
            assert_eq!(operation, "allocation");
            assert_eq!(requested_size, 1024);
            assert_eq!(available_capacity, 512);
        }
        _ => panic!("Expected BufferError variant"),
    }
}

#[test]
fn test_unsupported_feature_constructor_with_alternative() {
    let error =
        JsonPathError::unsupported_feature("recursive descent", Some("use explicit paths"));

    match error {
        JsonPathError::UnsupportedFeature {
            feature,
            alternative,
        } => {
            assert_eq!(feature, "recursive descent");
            assert_eq!(alternative, Some("use explicit paths".to_string()));
        }
        _ => panic!("Expected UnsupportedFeature variant"),
    }
}

#[test]
fn test_unsupported_feature_constructor_no_alternative() {
    let error = JsonPathError::unsupported_feature("advanced filter", None::<String>);

    match error {
        JsonPathError::UnsupportedFeature {
            feature,
            alternative,
        } => {
            assert_eq!(feature, "advanced filter");
            assert_eq!(alternative, None);
        }
        _ => panic!("Expected UnsupportedFeature variant"),
    }
}

#[test]
fn test_deserialization_constructor() {
    let error = JsonPathError::deserialization("simple error");

    match error {
        JsonPathError::Deserialization(message) => {
            assert_eq!(message, "simple error");
        }
        _ => panic!("Expected Deserialization variant"),
    }
}

#[test]
fn test_helper_invalid_syntax() {
    let error = JsonPathError::invalid_syntax("$.test[", 7);
    let display = format!("{}", error);
    assert!(display.contains("invalid JSONPath syntax"));
    assert!(display.contains("position 7"));
}

#[test]
fn test_helper_unsupported_operator() {
    let error = JsonPathError::unsupported_operator("@@");
    let display = format!("{}", error);
    assert!(display.contains("operator '@@'"));
    assert!(display.contains("check JSONPath specification"));
}

#[test]
fn test_helper_parse_failure() {
    let error = JsonPathError::parse_failure("unexpected token", 15);
    let display = format!("{}", error);
    assert!(display.contains("unexpected token"));
    assert!(display.contains("byte 15"));
}

#[test]
fn test_helper_buffer_overflow() {
    let error = JsonPathError::buffer_overflow(2048, 1024);
    let display = format!("{}", error);
    assert!(display.contains("overflow"));
    assert!(display.contains("2048"));
    assert!(display.contains("1024"));
}

#[test]
fn test_helper_stream_failure() {
    let error = JsonPathError::stream_failure("critical error", "parsing");
    let display = format!("{}", error);
    assert!(display.contains("critical error"));
    assert!(display.contains("parsing"));
    assert!(display.contains("recoverable: false"));
}

#[test]
fn test_helper_stream_warning() {
    let error = JsonPathError::stream_warning("minor issue", "processing");
    let display = format!("{}", error);
    assert!(display.contains("minor issue"));
    assert!(display.contains("processing"));
    assert!(display.contains("recoverable: true"));
}