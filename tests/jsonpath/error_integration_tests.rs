use std::io::{Error as IoError, ErrorKind};
use quyc_client::jsonpath::error::*;
use quyc_client::jsonpath::error::conversions::IntoJsonPathError;

#[test]
fn test_error_chain_serde_to_jsonpath() {
    let json_str = r#"{"invalid": syntax,}"#;
    let serde_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let jsonpath_error: JsonPathError = serde_error.into();

    // Should convert to appropriate JsonPathError variant
    let display = format!("{}", jsonpath_error);
    assert!(display.contains("expected") || display.contains("Deserialization"));
}

#[test]
fn test_error_chain_io_to_jsonpath() {
    let io_error = IoError::new(ErrorKind::BrokenPipe, "connection lost");
    let jsonpath_error: JsonPathError = io_error.into();

    match jsonpath_error {
        JsonPathError::StreamError {
            message,
            recoverable,
            ..
        } => {
            assert!(message.contains("connection lost"));
            assert_eq!(recoverable, false);
        }
        _ => panic!("Expected StreamError variant"),
    }
}

#[test]
fn test_constructor_and_display_integration() {
    let error = JsonPathError::invalid_expression(
        "$.users[name==",
        "incomplete comparison operator",
        Some(13),
    );

    let display = format!("{}", error);
    assert!(display.contains("Invalid JSONPath expression"));
    assert!(display.contains("$.users[name=="));
    assert!(display.contains("incomplete comparison operator"));
    assert!(display.contains("position 13"));
}#[test]
fn test_buffer_error_scenarios() {
    // Test buffer overflow
    let overflow_error = JsonPathError::buffer_overflow(2048, 1024);
    let display = format!("{}", overflow_error);
    assert!(display.contains("overflow"));
    assert!(display.contains("2048"));
    assert!(display.contains("1024"));

    // Test general buffer error
    let buffer_error = JsonPathError::buffer_error("reallocation", 4096, 2048);
    let display2 = format!("{}", buffer_error);
    assert!(display2.contains("reallocation"));
    assert!(display2.contains("4096"));
    assert!(display2.contains("2048"));
}

#[test]
fn test_stream_error_scenarios() {
    // Test unrecoverable stream failure
    let failure = JsonPathError::stream_failure("critical parser error", "parsing_object");
    match failure {
        JsonPathError::StreamError { recoverable, .. } => {
            assert_eq!(recoverable, false);
        }
        _ => panic!("Expected StreamError variant"),
    }

    // Test recoverable stream warning
    let warning = JsonPathError::stream_warning("temporary backpressure", "buffering");
    match warning {
        JsonPathError::StreamError { recoverable, .. } => {
            assert_eq!(recoverable, true);
        }
        _ => panic!("Expected StreamError variant"),
    }
}

#[test]
fn test_deserialization_scenarios() {
    // Test detailed deserialization error
    let detailed_error =
        JsonPathError::deserialization_error("expected string, found number", "42", "String");
    let display = format!("{}", detailed_error);
    assert!(display.contains("expected string, found number"));
    assert!(display.contains("42"));
    assert!(display.contains("String"));

    // Test simple deserialization error
    let simple_error = JsonPathError::deserialization("type mismatch");
    let display2 = format!("{}", simple_error);
    assert!(display2.contains("Deserialization error"));
    assert!(display2.contains("type mismatch"));
}#[test]
fn test_unsupported_feature_scenarios() {
    // Test with alternative
    let with_alt = JsonPathError::unsupported_feature(
        "recursive descent (..)",
        Some("use explicit path notation"),
    );
    let display = format!("{}", with_alt);
    assert!(display.contains("recursive descent"));
    assert!(display.contains("try: use explicit path notation"));

    // Test without alternative
    let without_alt = JsonPathError::unsupported_feature("complex filter", None::<String>);
    let display2 = format!("{}", without_alt);
    assert!(display2.contains("complex filter"));
    assert!(!display2.contains("try:"));
}

#[test]
fn test_helper_constructors() {
    // Test invalid syntax helper
    let syntax_error = JsonPathError::invalid_syntax("$.test[", 7);
    let display = format!("{}", syntax_error);
    assert!(display.contains("invalid JSONPath syntax"));

    // Test unsupported operator helper
    let op_error = JsonPathError::unsupported_operator("@@");
    let display2 = format!("{}", op_error);
    assert!(display2.contains("operator '@@'"));

    // Test parse failure helper
    let parse_error = JsonPathError::parse_failure("unexpected EOF", 100);
    let display3 = format!("{}", parse_error);
    assert!(display3.contains("unexpected EOF"));
    assert!(display3.contains("byte 100"));
}#[test]
fn test_conversion_trait_integration() {
    use quyc_client::jsonpath::error::conversions::IntoJsonPathError;

    // Test successful conversion
    let success: Result<i32, std::io::Error> = Ok(42);
    let result = success.into_jsonpath_error("test_context");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // Test error conversion with context
    let failure: Result<i32, std::io::Error> =
        Err(IoError::new(ErrorKind::NotFound, "missing"));
    let result = failure.into_jsonpath_error("file_operation");
    assert!(result.is_err());

    match result.unwrap_err() {
        JsonPathError::StreamError { state, .. } => {
            assert_eq!(state, "file_operation");
        }
        _ => panic!("Expected StreamError variant"),
    }
}

#[test]
fn test_specialized_conversions() {
    // Test serde conversion with context
    let json_str = r#"{"key": "not_an_int"}"#;
    let serde_error = serde_json::from_str::<i32>(json_str).unwrap_err();
    let jsonpath_error = JsonPathError::from_serde_with_context(serde_error, json_str, "i32");

    match jsonpath_error {
        JsonPathError::DeserializationError {
            json_fragment,
            target_type,
            ..
        } => {
            assert_eq!(json_fragment, json_str);
            assert_eq!(target_type, "i32");
        }
        _ => panic!("Expected DeserializationError variant"),
    }

    // Test IO conversion with context
    let io_error = IoError::new(ErrorKind::TimedOut, "operation timeout");
    let jsonpath_error = JsonPathError::from_io_with_context(io_error, "network_read");

    match jsonpath_error {
        JsonPathError::StreamError { state, .. } => {
            assert_eq!(state, "network_read");
        }
        _ => panic!("Expected StreamError variant"),
    }
}#[test]
fn test_error_trait_compliance() {
    let error = JsonPathError::invalid_expression("test", "test reason", None);

    // Test that it implements std::error::Error
    let error_trait: &dyn std::error::Error = &error;
    assert!(error_trait.source().is_none());

    // Test Display trait
    let display_str = format!("{}", error);
    assert!(!display_str.is_empty());

    // Test Debug trait
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("InvalidExpression"));

    // Test Clone trait
    let cloned = error.clone();
    assert_eq!(format!("{}", error), format!("{}", cloned));
}

#[test]
fn test_result_type_usage() {
    fn example_function() -> JsonPathResult<String> {
        Ok("success".to_string())
    }

    fn example_error_function() -> JsonPathResult<String> {
        Err(JsonPathError::deserialization("test error"))
    }

    // Test successful result
    let success = example_function();
    assert!(success.is_ok());
    assert_eq!(success.unwrap(), "success");

    // Test error result
    let failure = example_error_function();
    assert!(failure.is_err());
    match failure.unwrap_err() {
        JsonPathError::Deserialization(msg) => {
            assert_eq!(msg, "test error");
        }
        _ => panic!("Expected Deserialization variant"),
    }
}#[test]
fn test_comprehensive_error_coverage() {
    // Test all error variants can be created and displayed
    let errors = vec![
        JsonPathError::invalid_expression("$.test", "reason", Some(5)),
        JsonPathError::json_parse_error("parse error", 10, "context"),
        JsonPathError::deserialization_error("deser error", "json", "type"),
        JsonPathError::stream_error("stream error", "state", true),
        JsonPathError::buffer_error("buffer op", 100, 50),
        JsonPathError::unsupported_feature("feature", Some("alternative")),
        JsonPathError::deserialization("simple error"),
    ];

    for error in errors {
        // Each error should have non-empty display
        let display = format!("{}", error);
        assert!(!display.is_empty());

        // Each error should have debug representation
        let debug = format!("{:?}", error);
        assert!(!debug.is_empty());

        // Each error should be cloneable
        let _cloned = error.clone();
    }
}