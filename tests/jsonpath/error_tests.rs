//! JSON Path Error Tests
//!
//! Tests for the JSONPath error handling, moved from src/json_path/error.rs

use quyc::jsonpath::error::invalid_expression_error;
use quyc::jsonpath::{JsonPathError, JsonPathResult, JsonPathResultExt};

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_error_display_formatting() {
        let err = invalid_expression_error("$.invalid[", "unclosed bracket", Some(10));
        let display = format!("{}", err);
        assert!(display.contains("Invalid JSONPath expression"));
        assert!(display.contains("$.invalid["));
        assert!(display.contains("position 10"));
    }

    #[test]
    fn test_errorresult_extension_methods() {
        let result: JsonPathResult<String> = Err(JsonPathError::StreamError {
            message: "test error".to_string(),
            state: "initial".to_string(),
            recoverable: true,
        });

        // Test handle_or_default method
        let default_value = result.handle_or_default("fallback".to_string());
        assert_eq!(default_value, "fallback");

        // Test handle_or_log method
        let errorresult: JsonPathResult<i32> = Err(JsonPathError::InvalidExpression {
            expression: "$.invalid".to_string(),
            reason: "test".to_string(),
            position: None,
        });
        let logged_value = errorresult.handle_or_log("test context", 42);
        assert_eq!(logged_value, 42);
    }

    #[test]
    fn test_serde_json_error_conversion() {
        let json_err = serde_json::from_str::<i32>("invalid json")
            .err()
            .expect("Should error");
        let path_err: JsonPathError = json_err.into();

        assert!(matches!(path_err, JsonPathError::JsonParseError { .. }));
    }
}
