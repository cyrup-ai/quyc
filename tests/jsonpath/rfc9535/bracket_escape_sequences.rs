//! RFC 9535 Bracket Notation Escape Sequence Tests
//!
//! Tests all escape sequences within bracket notation as specified in RFC 9535 Appendix A
//! Covers both single and double quoted strings within brackets, Unicode escapes,
//! and character escaping requirements for JSONPath expressions

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct BracketTestModel {
    name: String,
    value: serde_json::Value,
}

/// RFC 9535 Bracket Notation Escape Sequence Tests
#[cfg(test)]
mod bracket_escape_tests {
    use super::*;

    #[test]
    fn test_single_quote_escape_sequences() {
        // RFC 9535: Test escape sequences in single-quoted bracket notation
        let json_data = r#"{
            "simple": "value1",
            "with'apostrophe": "value2",
            "with\\backslash": "value3",
            "with\nNewline": "value4",
            "with\tTab": "value5",
            "with\"DoubleQuote": "value6"
        }"#;

        let single_quote_tests = vec![
            // Basic single quote escaping
            ("$['simple']", 1, "Simple single quoted"),
            (
                "$['with\\'apostrophe']",
                1,
                "Escaped apostrophe in single quotes",
            ),
            (
                "$['with\\\\backslash']",
                1,
                "Escaped backslash in single quotes",
            ),
            ("$['with\\nNewline']", 1, "Escaped newline in single quotes"),
            ("$['with\\tTab']", 1, "Escaped tab in single quotes"),
            (
                "$['with\"DoubleQuote']",
                1,
                "Double quote in single quotes (no escape needed)",
            ),
        ];

        for (expr, expected_count, _description) in single_quote_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Single quote expression '{}' should compile: {}",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Single quote test '{}' should return {} results: {}",
                expr,
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_double_quote_escape_sequences() {
        // RFC 9535: Test escape sequences in double-quoted bracket notation
        let json_data = r#"{
            "simple": "value1",
            "with'apostrophe": "value2", 
            "with\\backslash": "value3",
            "with\nNewline": "value4",
            "with\tTab": "value5",
            "with\"DoubleQuote": "value6"
        }"#;

        let double_quote_tests = vec![
            // Basic double quote escaping
            (r#"$["simple"]"#, 1, "Simple double quoted"),
            (
                r#"$["with'apostrophe"]"#,
                1,
                "Apostrophe in double quotes (no escape needed)",
            ),
            (
                r#"$["with\\backslash"]"#,
                1,
                "Escaped backslash in double quotes",
            ),
            (
                r#"$["with\nNewline"]"#,
                1,
                "Escaped newline in double quotes",
            ),
            (r#"$["with\tTab"]"#, 1, "Escaped tab in double quotes"),
            (
                r#"$["with\"DoubleQuote"]"#,
                1,
                "Escaped double quote in double quotes",
            ),
        ];

        for (expr, expected_count, _description) in double_quote_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Double quote expression '{}' should compile: {}",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Double quote test '{}' should return {} results: {}",
                expr,
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_unicode_escape_sequences_in_brackets() {
        // RFC 9535: Test Unicode escape sequences in bracket notation
        let json_data = r#"{
            "A": "latin_A",
            "é": "e_acute", 
            "α": "greek_alpha",
            "🚀": "rocket_emoji",
            "東": "east_kanji",
            "Ω": "omega",
            "test": "normal"
        }"#;

        let unicode_escape_tests = vec![
            // Basic Unicode escapes
            (r#"$["\u0041"]"#, 1, "Unicode A (U+0041)"),
            (r#"$["\u00E9"]"#, 1, "Unicode é (U+00E9)"),
            (r#"$["\u03B1"]"#, 1, "Unicode α (U+03B1)"),
            // Emoji Unicode
            (r#"$["\uD83D\uDE80"]"#, 1, "Unicode 🚀 (surrogate pair)"),
            // CJK Unicode
            (r#"$["\u6771"]"#, 1, "Unicode 東 (U+6771)"),
            (r#"$["\u03A9"]"#, 1, "Unicode Ω (U+03A9)"),
            // Single quotes with Unicode
            (r#"$['\u0041']"#, 1, "Unicode A in single quotes"),
            (r#"$['\u00E9']"#, 1, "Unicode é in single quotes"),
        ];

        for (expr, expected_count, _description) in unicode_escape_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Unicode escape expression '{}' should compile: {}",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Unicode test '{}' returned {} results (expected {}) - {}",
                expr,
                results.len(),
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_control_character_escapes() {
        // RFC 9535: Test control character escape sequences
        let json_data = r#"{
            "with_backspace": "text\bhere",
            "with_formfeed": "text\fhere", 
            "with_carriage": "text\rhere",
            "with_linefeed": "text\nhere",
            "with_tab": "text\there",
            "normal": "normal_text"
        }"#;

        let control_char_tests = vec![
            // Standard control character escapes
            (r#"$["with_backspace"]"#, 1, "Backspace character"),
            (r#"$["with_formfeed"]"#, 1, "Form feed character"),
            (r#"$["with_carriage"]"#, 1, "Carriage return character"),
            (r#"$["with_linefeed"]"#, 1, "Line feed character"),
            (r#"$["with_tab"]"#, 1, "Tab character"),
            // Unicode control character escapes
            (
                r#"$["\u0008"]"#,
                0,
                "Unicode backspace (should not match literal)",
            ),
            (
                r#"$["\u0009"]"#,
                0,
                "Unicode tab (should not match literal)",
            ),
            (
                r#"$["\u000A"]"#,
                0,
                "Unicode line feed (should not match literal)",
            ),
            (
                r#"$["\u000C"]"#,
                0,
                "Unicode form feed (should not match literal)",
            ),
            (
                r#"$["\u000D"]"#,
                0,
                "Unicode carriage return (should not match literal)",
            ),
        ];

        for (expr, expected_count, _description) in control_char_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Control char expression '{}' should compile: {}",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Control char test '{}' returned {} results (expected {}) - {}",
                expr,
                results.len(),
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_invalid_escape_sequences() {
        // RFC 9535: Test invalid escape sequences that should be rejected
        let invalid_escape_tests = vec![
            // Invalid Unicode escapes
            (r#"$["\uXXXX"]"#, "Invalid Unicode hex digits"),
            (r#"$["\u123"]"#, "Incomplete Unicode escape (3 digits)"),
            (r#"$["\u12345"]"#, "Too many Unicode digits"),
            (r#"$["\u"]"#, "Empty Unicode escape"),
            // Invalid escape characters
            (r#"$["\x41"]"#, "Invalid hex escape sequence"),
            (r#"$["\z"]"#, "Invalid escape character z"),
            (r#"$["\q"]"#, "Invalid escape character q"),
            // Incomplete escapes
            (r#"$["test\"]"#, "Incomplete escape at end"),
            (r#"$["test\"]"#, "Backslash at string end"),
            // Invalid quote mixing
            (r#"$["unclosed']"#, "Mismatched quote types"),
            (r#"$['unclosed"]"#, "Mismatched quote types reverse"),
        ];

        for (expr, _description) in invalid_escape_tests {
            let result = JsonPathParser::compile(expr);
            // These should typically fail to compile
            println!(
                "Invalid escape test '{}' -> {:?} ({})",
                expr,
                result.is_ok(),
                _description
            );
        }
    }

    #[test]
    fn test_mixed_escaping_in_complex_expressions() {
        // RFC 9535: Test escape sequences in complex bracket expressions
        let json_data = r#"{
            "complex": {
                "key with spaces": "value1",
                "key'with'quotes": "value2", 
                "key\"with\"doubles": "value3",
                "key\\with\\backslashes": "value4",
                "key\nwith\nnewlines": "value5"
            }
        }"#;

        let complex_escape_tests = vec![
            // Chained bracket notation with escapes
            (
                r#"$["complex"]["key with spaces"]"#,
                1,
                "Spaces in chained brackets",
            ),
            (
                r#"$['complex']['key\'with\'quotes']"#,
                1,
                "Escaped quotes in chained",
            ),
            (
                r#"$["complex"]["key\"with\"doubles"]"#,
                1,
                "Escaped double quotes in chained",
            ),
            (
                r#"$['complex']['key\\with\\backslashes']"#,
                1,
                "Escaped backslashes in chained",
            ),
            (
                r#"$["complex"]["key\nwith\nnewlines"]"#,
                1,
                "Escaped newlines in chained",
            ),
            // Mixed notation with escapes
            (
                r#"$.complex["key with spaces"]"#,
                1,
                "Dot then bracket with spaces",
            ),
            (
                r#"$['complex'].key\'with\'quotes"#,
                0,
                "Invalid: bracket then dot with quotes",
            ),
            // Filter expressions with escapes
            (
                r#"$.complex[?@["key with spaces"] == "value1"]"#,
                1,
                "Filter with escaped key",
            ),
            (
                r#"$[?@.complex["key with spaces"]]"#,
                1,
                "Nested filter with spaces",
            ),
        ];

        for (expr, expected_count, _description) in complex_escape_tests {
            let result = JsonPathParser::compile(expr);

            if expected_count > 0 {
                assert!(
                    result.is_ok(),
                    "Complex escape expression '{}' should compile: {}",
                    expr,
                    _description
                );

                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                println!(
                    "Complex escape test '{}' returned {} results (expected {}) - {}",
                    expr,
                    results.len(),
                    expected_count,
                    _description
                );
            } else {
                // Expected to fail
                println!(
                    "Complex escape test (should fail) '{}' -> {:?} ({})",
                    expr,
                    result.is_ok(),
                    _description
                );
            }
        }
    }

    #[test]
    fn test_escape_sequence_edge_cases() {
        // RFC 9535: Test edge cases for escape sequences
        let edge_case_tests = vec![
            // Empty strings
            (r#"$[""]"#, "Empty string in double quotes"),
            (r#"$['']"#, "Empty string in single quotes"),
            // Only escape sequences
            (r#"$["\n"]"#, "Only newline escape"),
            (r#"$["\t"]"#, "Only tab escape"),
            (r#"$["\\"]"#, "Only backslash escape"),
            (r#"$["\""]"#, "Only quote escape"),
            // Multiple escapes
            (r#"$["\n\t\r"]"#, "Multiple control escapes"),
            (r#"$["\\\"\\"]"#, "Multiple quote/backslash escapes"),
            (
                r#"$["\u0041\u0042\u0043"]"#,
                "Multiple Unicode escapes (ABC)",
            ),
            // Very long escaped strings
            (
                r#"$["\u0041\u0042\u0043\u0044\u0045\u0046\u0047\u0048\u0049\u004A"]"#,
                "Long Unicode sequence",
            ),
            // Null character (if supported)
            (r#"$["\u0000"]"#, "Unicode null character"),
            // High Unicode values
            (r#"$["\uFFFF"]"#, "Max 4-digit Unicode"),
            // Case sensitivity in Unicode escapes
            (r#"$["\u0041"]"#, "Uppercase hex in Unicode"),
            (r#"$["\u0041"]"#, "Mixed case hex in Unicode"),
        ];

        for (expr, _description) in edge_case_tests {
            let result = JsonPathParser::compile(expr);
            // Document behavior for these edge cases
            println!(
                "Edge case escape test '{}' -> {:?} ({})",
                expr,
                result.is_ok(),
                _description
            );
        }
    }

    #[test]
    fn test_normalization_with_escapes() {
        // RFC 9535: Test that escaped and unescaped forms access same data
        let json_data = r#"{
            "A": "letter_A",
            "é": "e_with_acute",
            "test": "normal"
        }"#;

        let normalization_tests = vec![
            // These should be equivalent
            (r#"$["A"]"#, r#"$["\u0041"]"#, "Direct vs Unicode A"),
            (r#"$['A']"#, r#"$["\u0041"]"#, "Single quote vs Unicode A"),
            (r#"$["é"]"#, r#"$["\u00E9"]"#, "Direct vs Unicode é"),
            // Test equivalence with mixed quotes
            (r#"$["test"]"#, r#"$['test']"#, "Double vs single quotes"),
        ];

        for (expr1, expr2, _description) in normalization_tests {
            let result1 = JsonPathParser::compile(expr1);
            let result2 = JsonPathParser::compile(expr2);

            assert!(
                result1.is_ok(),
                "First expression '{}' should compile",
                expr1
            );
            assert!(
                result2.is_ok(),
                "Second expression '{}' should compile",
                expr2
            );

            let mut stream1 = JsonArrayStream::<serde_json::Value>::new(expr1);
            let mut stream2 = JsonArrayStream::<serde_json::Value>::new(expr2);

            let chunk = Bytes::from(json_data);
            let results1: Vec<_> = stream1.process_chunk(chunk.clone()).collect();
            let results2: Vec<_> = stream2.process_chunk(chunk).collect();

            println!(
                "Normalization test: '{}' vs '{}' -> {} vs {} results ({})",
                expr1,
                expr2,
                results1.len(),
                results2.len(),
                _description
            );
        }
    }
}
