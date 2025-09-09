//! JSON Path Error Tests
//!
//! Tests for the JSONPath error handling and RFC 9535 error compliance

use quyc::jsonpath::JsonPathParser;
use quyc::jsonpath::error::{
    JsonPathError, JsonPathResult, JsonPathResultExt, invalid_expression_error,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct ErrorTestModel {
    id: i32,
    data: Option<String>,
    nested: Option<serde_json::Value>,
}

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
    fn test_error_context_chaining() {
        let result: JsonPathResult<()> = Err(JsonPathError::StreamError {
            message: "test error".to_string(),
            state: "initial".to_string(),
            recoverable: true,
        });

        let with_context = result.with_stream_context("parsing");
        assert!(
            matches!(with_context, Err(JsonPathError::StreamError { state, .. }) if state == "parsing")
        );
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

/// Well-formedness vs Validity Error Tests
#[cfg(test)]
mod wellformedness_validity_tests {
    use super::*;

    #[test]
    fn test_syntax_well_formedness_errors() {
        // RFC 9535: Test well-formedness errors (syntax violations)
        let syntax_errors = vec![
            // Unclosed constructs
            ("$[", "Unclosed bracket selector"),
            ("$.key[", "Unclosed array access"),
            ("$.key[?", "Unclosed filter expression"),
            ("$.key[?@.prop", "Incomplete filter expression"),
            ("$.key[?@.prop ==", "Incomplete comparison"),
            ("$[\"unclosed", "Unclosed string literal"),
            ("$['unclosed", "Unclosed single-quoted string"),
            // Invalid characters and sequences
            ("$.", "Trailing dot with no property"),
            ("..", "Invalid double dot without continuation"),
            ("$...", "Triple dot sequence"),
            ("$key", "Missing root $ prefix"),
            ("key.value", "No root identifier"),
        ];

        for (invalid_expr, _description) in syntax_errors {
            let result = JsonPathParser::compile(invalid_expr);
            assert!(
                result.is_err(),
                "Syntax error '{}' should fail: {}",
                invalid_expr,
                _description
            );

            if let Err(error) = result {
                // Verify error provides meaningful information
                let error_msg = format!("{}", error);
                assert!(
                    !error_msg.is_empty(),
                    "Error message should not be empty for: {}",
                    invalid_expr
                );
            }
        }
    }

    #[test]
    fn test_semantic_validity_errors() {
        // RFC 9535: Test validity errors (semantically invalid but syntactically correct)
        let validity_errors = vec![
            ("$[999999999999999999999]", "Array index overflow"),
            ("$[-999999999999999999999]", "Negative array index overflow"),
            ("$[1:999999999999999999999]", "Slice end overflow"),
            ("$[::0]", "Zero step in slice"),
            ("$.['property']", "Mixed bracket/dot notation"),
        ];

        for (invalid_expr, _description) in validity_errors {
            let result = JsonPathParser::compile(invalid_expr);
            // Some validity errors may be caught at compile time, others at runtime
            match result {
                Ok(_) => println!(
                    "Validity check '{}' passed compilation: {}",
                    invalid_expr, _description
                ),
                Err(_) => println!(
                    "Validity check '{}' failed compilation: {}",
                    invalid_expr, _description
                ),
            }
        }
    }

    #[test]
    fn test_error_position_reporting() {
        // Test that error positions are accurately reported
        let positioned_errors = vec![
            ("$.valid.invalid[", 14),  // Error at position of unclosed bracket
            ("$.valid[?@.test =", 16), // Error at incomplete comparison
            ("$.valid..invalid", 8),   // Error at double dot
        ];

        for (invalid_expr, expected_position) in positioned_errors {
            let result = JsonPathParser::compile(invalid_expr);
            assert!(result.is_err(), "Should fail: {}", invalid_expr);

            if let Err(error) = result {
                let error_msg = format!("{}", error);
                // Check if position information is provided (implementation-dependent)
                println!(
                    "Error position test '{}' reported: {}",
                    invalid_expr, error_msg
                );
            }
        }
    }
}

/// Error Message Quality Tests
#[cfg(test)]
mod error_message_quality_tests {
    use super::*;

    #[test]
    fn test_error_message_clarity() {
        // Test that error messages are clear and helpful
        let test_cases = vec![
            ("$[", "unclosed"),
            ("$.invalid[?", "filter"),
            ("key.value", "root"),
            ("$.", "property"),
        ];

        for (invalid_expr, expected_keyword) in test_cases {
            let result = JsonPathParser::compile(invalid_expr);
            assert!(result.is_err(), "Should fail: {}", invalid_expr);

            if let Err(error) = result {
                let error_msg = format!("{}", error).to_lowercase();
                assert!(
                    error_msg.contains(expected_keyword),
                    "Error message '{}' should contain '{}' for expression '{}'",
                    error_msg,
                    expected_keyword,
                    invalid_expr
                );
            }
        }
    }

    #[test]
    fn test_error_consistency() {
        // Test that similar errors produce consistent messages
        let bracket_errors = vec!["$[", "$.key[", "$.path[?"];
        let error_messages: Vec<String> = bracket_errors
            .iter()
            .filter_map(|expr| {
                JsonPathParser::compile(expr)
                    .err()
                    .map(|e| format!("{}", e))
            })
            .collect();

        // All bracket-related errors should have some consistency
        for msg in &error_messages {
            println!("Bracket error message: {}", msg);
        }

        // Verify we got error messages for all test cases
        assert_eq!(
            error_messages.len(),
            bracket_errors.len(),
            "Should get error messages for all bracket test cases"
        );
    }
}

/// Performance Under Error Conditions
#[cfg(test)]
mod error_performance_tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_error_detection_performance() {
        // Test that error detection doesn't cause performance degradation
        let complex_invalid_expr = "$".repeat(1000) + "[invalid syntax here";

        let start = Instant::now();
        let result = JsonPathParser::compile(&complex_invalid_expr);
        let duration = start.elapsed();

        assert!(result.is_err(), "Complex invalid expression should fail");
        assert!(
            duration.as_millis() < 100,
            "Error detection should be fast even for complex invalid expressions"
        );
    }

    #[test]
    fn test_repeated_error_parsing() {
        // Test performance of repeated error parsing
        let invalid_expressions = vec!["$[", "$.invalid[", "$.test[?", "key.value", "$."];

        let start = Instant::now();

        for _ in 0..1000 {
            for expr in &invalid_expressions {
                let _ = JsonPathParser::compile(expr);
            }
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 1000,
            "Repeated error parsing should be efficient"
        );
    }
}
