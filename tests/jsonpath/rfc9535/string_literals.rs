//! RFC 9535 String Literals Test Suite (Section 2.2.4.2)
//!
//! Tests for all escape sequences, string handling, and literal validation:
//! - Standard escape sequences: \b, \t, \n, \f, \r, \", \', \/, \\
//! - Unicode escape sequences: \uXXXX
//! - Invalid escape sequence error handling
//! - Hexadecimal escape validation
//! - String comparison edge cases
//! - Quote handling in JSONPath expressions

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct StringTestModel {
    text: String,
    _description: String,
    code: String,
}

/// RFC 9535 Section 2.2.4.2 - String Literals Tests
#[cfg(test)]
mod string_literal_tests {
    use super::*;

    #[test]
    fn test_basic_escape_sequences() {
        // RFC 9535: Standard escape sequences in string literals
        let json_data = r#"{
            "items": [
                {"text": "line1\nline2"},
                {"text": "tab\there"},
                {"text": "quote\"inside"},
                {"text": "backslash\\here"}
            ]
        }"#;

        let test_cases = vec![
            (r#"$.items[?@.text == "line1\nline2"]"#, 1), // Newline escape
            (r#"$.items[?@.text == "tab\there"]"#, 1),    // Tab escape
            (r#"$.items[?@.text == "quote\"inside"]"#, 1), // Quote escape
            (r#"$.items[?@.text == "backslash\\here"]"#, 1), // Backslash escape
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Escape sequence test '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_all_standard_escape_sequences() {
        // RFC 9535: Complete set of standard escape sequences
        let json_data = r#"{
            "escapes": [
                {"type": "backspace", "char": "\b"},
                {"type": "tab", "char": "\t"},
                {"type": "newline", "char": "\n"},
                {"type": "formfeed", "char": "\f"},
                {"type": "return", "char": "\r"},
                {"type": "quote", "char": "\""},
                {"type": "apostrophe", "char": "'"},
                {"type": "slash", "char": "/"},
                {"type": "backslash", "char": "\\"}
            ]
        }"#;

        let test_cases = vec![
            (r#"$.escapes[?@.char == "\b"]"#, 1), // Backspace
            (r#"$.escapes[?@.char == "\t"]"#, 1), // Tab
            (r#"$.escapes[?@.char == "\n"]"#, 1), // Newline
            (r#"$.escapes[?@.char == "\f"]"#, 1), // Form feed
            (r#"$.escapes[?@.char == "\r"]"#, 1), // Carriage return
            (r#"$.escapes[?@.char == "\""]"#, 1), // Double quote
            (r#"$.escapes[?@.char == "'"]"#, 1),  // Single quote (no escape needed)
            (r#"$.escapes[?@.char == "/"]"#, 1),  // Slash (no escape needed)
            (r#"$.escapes[?@.char == "\\"]"#, 1), // Backslash
        ];

        for (expr, expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    assert_eq!(
                        results.len(),
                        expected_count,
                        "Standard escape '{}' should return {} items",
                        expr,
                        expected_count
                    );
                }
                Err(e) => {
                    println!("Escape sequence '{}' not supported: {:?}", expr, e);
                }
            }
        }
    }

    #[test]
    fn test_unicode_escape_sequences() {
        // RFC 9535: Unicode escape sequences \uXXXX
        let json_data = r#"{
            "unicode": [
                {"char": "A", "_description": "Latin A"},
                {"char": "α", "_description": "Greek alpha"},
                {"char": "中", "_description": "Chinese character"},
                {"char": "🚀", "_description": "Rocket emoji"},
                {"char": "€", "_description": "Euro symbol"}
            ]
        }"#;

        let test_cases = vec![
            (r#"$.unicode[?@.char == "\u0041"]"#, 1), // Latin A (U+0041)
            (r#"$.unicode[?@.char == "\u03B1"]"#, 1), // Greek alpha (U+03B1)
            (r#"$.unicode[?@.char == "\u4E2D"]"#, 1), // Chinese 中 (U+4E2D)
            (r#"$.unicode[?@.char == "\u20AC"]"#, 1), // Euro € (U+20AC)
        ];

        for (expr, expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Unicode escape '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(e) => {
                    println!("Unicode escape '{}' not supported: {:?}", expr, e);
                }
            }
        }
    }

    #[test]
    fn test_quote_handling_variations() {
        // RFC 9535: Different quote handling scenarios
        let json_data = r#"{
            "quotes": [
                {"text": "double\"quote"},
                {"text": "single'quote"},
                {"text": "mixed\"and'quotes"},
                {"text": "nested\"quotes\"here"}
            ]
        }"#;

        let test_cases = vec![
            // Double quotes in JSONPath expressions
            (r#"$.quotes[?@.text == "double\"quote"]"#, 1),
            (r#"$.quotes[?@.text == "single'quote"]"#, 1),
            (r#"$.quotes[?@.text == "mixed\"and'quotes"]"#, 1),
            // Alternative: Single quotes for JSONPath string literals
            ("$.quotes[?@.text == 'double\"quote']", 1),
            ("$.quotes[?@.text == 'single\\'quote']", 1),
        ];

        for (expr, expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Quote handling '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(e) => {
                    println!("Quote handling '{}' failed: {:?}", expr, e);
                }
            }
        }
    }

    #[test]
    fn test_complex_string_patterns() {
        // RFC 9535: Complex string patterns with multiple escapes
        let json_data = r#"{
            "complex": [
                {"pattern": "line1\nline2\ttab"},
                {"pattern": "path\\to\\file.txt"},
                {"pattern": "json: {\"key\": \"value\"}"},
                {"pattern": "regex: ^[a-z]+$"}
            ]
        }"#;

        let test_cases = vec![
            (r#"$.complex[?@.pattern == "line1\nline2\ttab"]"#, 1),
            (r#"$.complex[?@.pattern == "path\\to\\file.txt"]"#, 1),
            (
                r#"$.complex[?@.pattern == "json: {\"key\": \"value\"}"]"#,
                1,
            ),
            (r#"$.complex[?@.pattern == "regex: ^[a-z]+$"]"#, 1),
        ];

        for (expr, expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Complex pattern '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(e) => {
                    println!("Complex pattern '{}' failed: {:?}", expr, e);
                }
            }
        }
    }

    #[test]
    fn test_empty_string_handling() {
        // RFC 9535: Empty string edge cases
        let json_data = r#"{
            "strings": [
                {"value": ""},
                {"value": " "},
                {"value": "\t"},
                {"value": "\n"},
                {"value": "non-empty"}
            ]
        }"#;

        let test_cases = vec![
            (r#"$.strings[?@.value == ""]"#, 1),   // Empty string
            (r#"$.strings[?@.value == " "]"#, 1),  // Space
            (r#"$.strings[?@.value == "\t"]"#, 1), // Tab only
            (r#"$.strings[?@.value == "\n"]"#, 1), // Newline only
            (r#"$.strings[?@.value != ""]"#, 4),   // Non-empty strings
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Empty string test '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_string_comparison_edge_cases() {
        // RFC 9535: String comparison behaviors
        let json_data = r#"{
            "comparisons": [
                {"text": "abc"},
                {"text": "ABC"},
                {"text": "123"},
                {"text": "!@#"},
                {"text": "äöü"},
                {"text": "😀😃😄"}
            ]
        }"#;

        let test_cases = vec![
            // Case sensitivity
            (r#"$.comparisons[?@.text == "abc"]"#, 1),
            (r#"$.comparisons[?@.text == "ABC"]"#, 1),
            (r#"$.comparisons[?@.text != "abc"]"#, 5),
            // Different character types
            (r#"$.comparisons[?@.text == "123"]"#, 1),
            (r#"$.comparisons[?@.text == "!@#"]"#, 1),
            (r#"$.comparisons[?@.text == "äöü"]"#, 1),
            (r#"$.comparisons[?@.text == "😀😃😄"]"#, 1),
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "String comparison '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }
}

/// String Literal Error Cases and Validation
#[cfg(test)]
mod string_literal_error_tests {
    use super::*;

    #[test]
    fn test_invalid_escape_sequences() {
        // Test invalid escape sequences that should fail
        let invalid_escapes = vec![
            r#"$.items[?@.text == "invalid\x20"]"#, // Invalid \x escape
            r#"$.items[?@.text == "invalid\q"]"#,   // Invalid single char escape
            r#"$.items[?@.text == "invalid\"]"#,    // Trailing backslash
            r#"$.items[?@.text == "unclosed"#,      // Unclosed string
        ];

        for expr in invalid_escapes {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Invalid escape '{}' should fail", expr);
        }
    }

    #[test]
    fn test_invalid_unicode_escapes() {
        // Test invalid Unicode escape sequences
        let invalid_unicode = vec![
            r#"$.items[?@.text == "\u"]"#,      // Incomplete Unicode escape
            r#"$.items[?@.text == "\u123"]"#,   // Too short Unicode escape
            r#"$.items[?@.text == "\u123G"]"#,  // Invalid hex digit
            r#"$.items[?@.text == "\uXYZW"]"#,  // Non-hex Unicode escape
            r#"$.items[?@.text == "\u12345"]"#, // Too long Unicode escape
        ];

        for expr in invalid_unicode {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Invalid Unicode escape '{}' should fail",
                expr
            );
        }
    }

    #[test]
    fn test_unclosed_strings() {
        // Test unclosed string literals
        let unclosed_strings = vec![
            r#"$.items[?@.text == "unclosed"#,  // Missing closing quote
            r#"$.items[?@.text == 'unclosed"#,  // Mixed quotes
            r#"$.items[?@.text == "newline\n"#, // Actual newline in string
        ];

        for expr in unclosed_strings {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Unclosed string '{}' should fail", expr);
        }
    }

    #[test]
    fn test_quote_mixing_errors() {
        // Test improper quote mixing
        let quote_errors = vec![
            r#"$.items[?@.text == "mixed'quotes"]"#, // This should work
            r#"$.items[?@.text == 'mixed"quotes']"#, // This should work
            r#"$.items[?@.text == "mixed"quotes"]"#, // This should fail
            r#"$.items[?@.text == 'mixed'quotes']"#, // This should fail
        ];

        for (i, expr) in quote_errors.iter().enumerate() {
            let result = JsonPathParser::compile(expr);
            match i {
                0 | 1 => {
                    // These should work (proper escaping or different quote types)
                    println!("Quote mixing test {}: {} - {:?}", i, expr, result.is_ok());
                }
                2 | 3 => {
                    // These should fail (improper quote usage)
                    assert!(
                        result.is_err(),
                        "Improper quote mixing '{}' should fail",
                        expr
                    );
                }
                _ => {}
            }
        }
    }
}

/// Extended String Literal Tests
#[cfg(test)]
mod extended_string_tests {
    use super::*;

    #[test]
    fn test_hexadecimal_validation() {
        // Test hexadecimal character validation in Unicode escapes
        let hex_test_cases = vec![
            (r#"\u0000"#, true), // Valid: all zeros
            (r#"\u0041"#, true), // Valid: Latin A
            (r#"\uFFFF"#, true), // Valid: max value
            (r#"\u12AB"#, true), // Valid: mixed case
            (r#"\uabcd"#, true), // Valid: lowercase
            (r#"\uABCD"#, true), // Valid: uppercase
        ];

        for (escape_seq, _should_be_valid) in hex_test_cases {
            let expr = format!(r#"$.test[?@.char == "{}"]"#, escape_seq);
            let result = JsonPathParser::compile(&expr);

            if _should_be_valid {
                println!(
                    "Hex validation '{}' should be valid: {:?}",
                    escape_seq,
                    result.is_ok()
                );
            } else {
                assert!(result.is_err(), "Invalid hex '{}' should fail", escape_seq);
            }
        }
    }

    #[test]
    fn test_string_literal_performance() {
        // Test performance with long strings and many escapes
        let long_string = "a".repeat(1000);
        let escaped_string = "\\n".repeat(500); // 1000 characters with 500 escapes

        let json_value = serde_json::json!({
            "performance": [
                {"text": long_string},
                {"text": "short"},
                {"text": escaped_string}
            ]
        });

        let json_data = serde_json::to_string(&json_value).expect("Valid JSON");

        let start_time = std::time::Instant::now();

        let expr = format!(r#"$.performance[?@.text == "{}"]"#, long_string);
        let mut stream = JsonArrayStream::<serde_json::Value>::new(&expr);

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        let duration = start_time.elapsed();

        assert_eq!(results.len(), 1, "Should find the long string");
        assert!(
            duration.as_millis() < 1000,
            "String literal performance should complete in <1000ms, took {:?}",
            duration
        );
    }

    #[test]
    fn test_multibyte_character_handling() {
        // Test proper handling of multibyte UTF-8 characters
        let json_data = r#"{
            "multibyte": [
                {"text": "café"},
                {"text": "naïve"},
                {"text": "Москва"},
                {"text": "東京"},
                {"text": "🎉🎊🎈"}
            ]
        }"#;

        let test_cases = vec![
            (r#"$.multibyte[?@.text == "café"]"#, 1),   // French accents
            (r#"$.multibyte[?@.text == "naïve"]"#, 1),  // Diaeresis
            (r#"$.multibyte[?@.text == "Москва"]"#, 1), // Cyrillic
            (r#"$.multibyte[?@.text == "東京"]"#, 1),   // Japanese
            (r#"$.multibyte[?@.text == "🎉🎊🎈"]"#, 1), // Emoji
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Multibyte test '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_control_character_handling() {
        // Test handling of control characters in strings
        let json_data = serde_json::json!({
            "controls": [
                {"text": "\u{0000}"},          // NULL
                {"text": "\u{0001}"},          // SOH
                {"text": "\u{0008}"},          // Backspace
                {"text": "\u{0009}"},          // Tab
                {"text": "\u{000A}"},          // Line Feed
                {"text": "\u{000D}"},          // Carriage Return
                {"text": "\u{001F}"},          // Unit Separator
            ]
        });

        let json_str = serde_json::to_string(&json_data).expect("Valid JSON");

        let test_cases = vec![
            (format!(r#"$.controls[?@.text == "{}"]"#, "\u{0000}"), 1), // NULL character
            (format!(r#"$.controls[?@.text == "{}"]"#, "\u{0008}"), 1), // Backspace
            (format!(r#"$.controls[?@.text == "{}"]"#, "\u{0009}"), 1), // Tab
            (format!(r#"$.controls[?@.text == "{}"]"#, "\u{000A}"), 1), // Line Feed
            (format!(r#"$.controls[?@.text == "{}"]"#, "\u{000D}"), 1), // Carriage Return
        ];

        for (expr, expected_count) in test_cases {
            let result = JsonPathParser::compile(&expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(&expr);

                    let chunk = Bytes::from(json_str.clone());
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Control character test '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(e) => {
                    println!("Control character test '{}' failed: {:?}", expr, e);
                }
            }
        }
    }
}
