//! RFC 9535 Security and Error Handling Tests
//!
//! Tests for RFC 9535 security considerations and error handling:
//! - Parser vulnerability tests for malformed inputs
//! - Comprehensive UTF-8 decode error handling tests  
//! - Memory exhaustion protection tests for deep nesting
//! - Security boundary validation
//!
//! This test suite validates:
//! - Robust error handling for malicious inputs
//! - UTF-8 encoding/decoding security
//! - Memory protection mechanisms
//! - Parser resilience under attack

use std::time::{Duration, Instant};

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};

/// Create malformed JSON for security testing
fn create_malformed_json_variants() -> Vec<(&'static str, &'static str)> {
    vec![
        // Unterminated structures
        (r#"{"incomplete": "#, "Unterminated object"),
        (r#"["incomplete""#, "Unterminated array"),
        (r#"{"key": "unterminated string"#, "Unterminated string"),
        (
            r#"{"nested": {"deep": {"incomplete": "#,
            "Deeply nested incomplete",
        ),
        // Invalid escape sequences
        (r#"{"key": "invalid\escape"}"#, "Invalid escape sequence"),
        (r#"{"key": "unicode\uGGGG"}"#, "Invalid unicode escape"),
        (r#"{"key": "short\u12"}"#, "Short unicode escape"),
        (r#"{"key": "null\0byte"}"#, "Null byte in string"),
        // Malformed numbers
        (r#"{"number": 123.}"#, "Trailing decimal point"),
        (r#"{"number": .123}"#, "Leading decimal point"),
        (r#"{"number": 123.45.67}"#, "Multiple decimal points"),
        (r#"{"number": 1e}"#, "Incomplete scientific notation"),
        (r#"{"number": --123}"#, "Double negative"),
        // Invalid JSON structure
        (r#"{"key": value}"#, "Unquoted value"),
        (r#"{'key': 'value'}"#, "Single quoted strings"),
        (r#"{"key": undefined}"#, "Undefined value"),
        (r#"{key: "value"}"#, "Unquoted key"),
        (r#"{"trailing": "comma",}"#, "Trailing comma"),
        // Control characters
        (r#"{"key": "line\nbreak"}"#, "Unescaped newline"),
        (r#"{"key": "tab\there"}"#, "Unescaped tab"),
        (
            r#"{"key": "carriage\rreturn"}"#,
            "Unescaped carriage return",
        ),
    ]
}

/// RFC 9535 Security - Parser Vulnerability Tests
#[cfg(test)]
mod parser_vulnerability_tests {
    use super::*;

    #[test]
    fn test_malformed_json_resilience() {
        // Security: Parser should handle malformed JSON gracefully
        let malformed_variants = create_malformed_json_variants();

        for (malformed_json, _description) in malformed_variants {
            let expr = "$..*"; // Simple recursive descent

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(malformed_json);

            // Should not crash or panic, even with malformed input
            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should complete quickly and not hang
            assert!(
                elapsed < Duration::from_secs(1),
                "Malformed JSON processing should not hang: {} ({})",
                _description,
                elapsed.as_millis()
            );

            // Results should be empty or error, but not crash
            println!(
                "Malformed JSON test '{}': {} results in {}ms",
                _description,
                results.len(),
                elapsed.as_millis()
            );
        }
    }

    #[test]
    fn test_deeply_nested_malformed_structures() {
        // Security: Test deeply nested malformed structures
        let deep_nesting_tests = vec![
            // Deeply nested incomplete structures
            (100, "Deep incomplete objects"),
            (500, "Very deep incomplete objects"),
            (1000, "Extremely deep incomplete objects"),
        ];

        for (depth, _description) in deep_nesting_tests {
            let mut malformed_json = String::new();

            // Create deeply nested incomplete structure
            for _ in 0..depth {
                malformed_json.push_str(r#"{"nested":"#);
            }
            malformed_json.push_str("\"incomplete");
            // Intentionally leave incomplete (no closing braces)

            let expr = "$..*";
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(malformed_json.clone());

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should not hang or crash with deep malformed nesting
            assert!(
                elapsed < Duration::from_secs(5),
                "Deep malformed nesting should not hang: {} depth {} ({}ms)",
                _description,
                depth,
                elapsed.as_millis()
            );

            println!(
                "Deep malformed test {}: {} results in {}ms",
                _description,
                results.len(),
                elapsed.as_millis()
            );
        }
    }

    #[test]
    fn test_parser_injection_attacks() {
        // Security: Test potential parser injection attacks
        let expr_overflow = format!("$.store.book[?{}true]", "@.title == 'a' && ".repeat(1000));
        let property_overflow = format!("$.{}end", "property.".repeat(1000));

        let injection_tests = vec![
            // JSONPath injection attempts
            (
                "$.store.book[?@.title == 'title' || true]",
                "Boolean injection attempt",
            ),
            ("$.store.book[?@.price == 0 or 1=1]", "SQL-style injection"),
            (
                "$.store.book[?@.author == ''; DROP TABLE books; --']",
                "SQL injection attempt",
            ),
            (
                "$.store.book[?@.title == '<script>alert(1)</script>']",
                "XSS injection attempt",
            ),
            // Parser escape attempts
            ("$.store.book[?@.title == '\\u0000']", "Null byte injection"),
            (
                "$.store.book[?@.title == '\\u001F']",
                "Control character injection",
            ),
            (
                "$.store.book[?@.title == '\\xFF\\xFE']",
                "Byte order mark injection",
            ),
            // Parser overflow attempts
            (expr_overflow.as_str(), "Expression overflow"),
            (property_overflow.as_str(), "Property chain overflow"),
        ];

        for (expr, _description) in injection_tests {
            let start_time = Instant::now();
            let result = JsonPathParser::compile(expr);
            let elapsed = start_time.elapsed();

            // Should complete quickly - no infinite loops or hangs
            assert!(
                elapsed < Duration::from_secs(1),
                "Parser injection test should complete quickly: {} ({}ms)",
                _description,
                elapsed.as_millis()
            );

            // Test execution if compilation succeeds
            if result.is_ok() {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(
                    r#"{"store":{"book":[{"title":"test","price":10,"author":"test"}]}}"#,
                );

                let exec_start = Instant::now();
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                let exec_elapsed = exec_start.elapsed();

                assert!(
                    exec_elapsed < Duration::from_secs(2),
                    "Injection test execution should not hang: {} ({}ms)",
                    _description,
                    exec_elapsed.as_millis()
                );

                println!(
                    "Injection test '{}': compiled and executed {} results in {}ms",
                    _description,
                    results.len(),
                    exec_elapsed.as_millis()
                );
            } else {
                println!(
                    "Injection test '{}': rejected at compile time",
                    _description
                );
            }
        }
    }

    #[test]
    fn test_resource_exhaustion_protection() {
        // Security: Test protection against resource exhaustion
        let large_string_expr = format!("$.store.book[?@.title == '{}']", "A".repeat(10000));
        let very_large_string_expr = format!("$.store.book[?@.author == '{}']", "B".repeat(50000));
        let complex_bool_expr = format!("$.store.book[?{}true]", "@.price > 1 && ".repeat(100));
        let large_or_expr = format!("$.store.book[?{}false]", "@.title != 'x' || ".repeat(50));
        let large_number_expr = "$.store.book[?@.price == 99999999999999999999]".to_string();
        let large_negative_expr = "$.store.book[?@.price == -99999999999999999999]".to_string();

        let exhaustion_tests = vec![
            // Large string attacks
            (&large_string_expr, "Large string comparison"),
            (&very_large_string_expr, "Very large string comparison"),
            // Large number attacks
            (&large_number_expr, "Large number comparison"),
            (&large_negative_expr, "Large negative number"),
            // Complex expression attacks
            (&complex_bool_expr, "Complex boolean expression"),
            (&large_or_expr, "Large OR expression"),
        ];

        for (expr, _description) in exhaustion_tests {
            let start_time = Instant::now();
            let result = JsonPathParser::compile(expr);
            let elapsed = start_time.elapsed();

            // Compilation should complete in reasonable time
            assert!(
                elapsed < Duration::from_secs(5),
                "Resource exhaustion test compilation should not hang: {} ({}ms)",
                _description,
                elapsed.as_millis()
            );

            if result.is_ok() {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(
                    r#"{"store":{"book":[{"title":"test","price":10,"author":"test"}]}}"#,
                );

                let exec_start = Instant::now();
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                let exec_elapsed = exec_start.elapsed();

                assert!(
                    exec_elapsed < Duration::from_secs(10),
                    "Resource exhaustion test execution should not hang: {} ({}ms)",
                    _description,
                    exec_elapsed.as_millis()
                );

                println!(
                    "Resource test '{}': {} results in {}ms",
                    _description,
                    results.len(),
                    exec_elapsed.as_millis()
                );
            }
        }
    }
}

/// RFC 9535 Security - UTF-8 Decode Error Handling Tests
#[cfg(test)]
mod utf8_decode_error_tests {
    use super::*;

    #[test]
    fn test_invalid_utf8_sequences() {
        // Security: Test handling of invalid UTF-8 sequences
        let invalid_utf8_tests = vec![
            // Invalid UTF-8 byte sequences (as raw bytes)
            (vec![0xFF, 0xFE], "Invalid BOM sequence"),
            (vec![0x80, 0x81], "Invalid continuation bytes"),
            (vec![0xC0, 0x80], "Overlong encoding"),
            (vec![0xED, 0xA0, 0x80], "Surrogate half"),
            (vec![0xF4, 0x90, 0x80, 0x80], "Code point too large"),
            (vec![0xC2], "Incomplete 2-byte sequence"),
            (vec![0xE0, 0x80], "Incomplete 3-byte sequence"),
            (vec![0xF0, 0x80, 0x80], "Incomplete 4-byte sequence"),
        ];

        for (invalid_bytes, _description) in invalid_utf8_tests {
            // Create JSON with invalid UTF-8 embedded
            let mut json_bytes = b"{\"key\": \"".to_vec();
            json_bytes.extend_from_slice(&invalid_bytes);
            json_bytes.extend_from_slice(b"\"}");

            let expr = "$.key";
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_bytes);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should handle invalid UTF-8 gracefully without crashing
            assert!(
                elapsed < Duration::from_secs(1),
                "Invalid UTF-8 test should complete quickly: {} ({}ms)",
                _description,
                elapsed.as_millis()
            );

            println!(
                "Invalid UTF-8 test '{}': {} results in {}ms",
                _description,
                results.len(),
                elapsed.as_millis()
            );
        }
    }

    #[test]
    fn test_utf8_boundary_conditions() {
        // Security: Test UTF-8 boundary conditions
        let boundary_tests = vec![
            // Valid edge cases
            ("$.key", r#"{"key": "\u0000"}"#, "Null character"),
            ("$.key", r#"{"key": "\u001F"}"#, "Last control character"),
            ("$.key", r#"{"key": "\u0020"}"#, "First printable character"),
            ("$.key", r#"{"key": "\u007F"}"#, "DEL character"),
            ("$.key", r#"{"key": "\u0080"}"#, "First extended ASCII"),
            ("$.key", r#"{"key": "\u00FF"}"#, "Last Latin-1"),
            ("$.key", r#"{"key": "\u0100"}"#, "First beyond Latin-1"),
            ("$.key", r#"{"key": "\uFFFF"}"#, "Last BMP character"),
            // Surrogate pairs
            (
                "$.key",
                r#"{"key": "\uD800\uDC00"}"#,
                "First surrogate pair",
            ),
            ("$.key", r#"{"key": "\uDBFF\uDFFF"}"#, "Last surrogate pair"),
            // Special Unicode ranges
            ("$.key", r#"{"key": "café"}"#, "Latin extended"),
            ("$.key", r#"{"key": "中文"}"#, "CJK characters"),
            ("$.key", r#"{"key": "العربية"}"#, "Arabic script"),
            ("$.key", r#"{"key": "🚀🎉"}"#, "Emoji characters"),
        ];

        for (expr, _json_data, _description) in boundary_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(_json_data);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should handle UTF-8 boundary cases correctly
            assert!(
                elapsed < Duration::from_millis(100),
                "UTF-8 boundary test should be fast: {} ({}ms)",
                _description,
                elapsed.as_millis()
            );

            assert_eq!(
                results.len(),
                1,
                "UTF-8 boundary test should find the value: {} ({})",
                _description,
                _json_data
            );
        }
    }

    #[test]
    fn test_utf8_normalization_attacks() {
        // Security: Test UTF-8 normalization attacks
        let normalization_tests = vec![
            // Different representations of same character
            ("$.café", r#"{"café": "value1"}"#, "NFC normalization"),
            ("$.café", r#"{"cafe\u0301": "value2"}"#, "NFD normalization"),
            // Case folding attacks
            ("$.Café", r#"{"café": "value"}"#, "Case sensitivity test"),
            ("$.CAFÉ", r#"{"café": "value"}"#, "Uppercase vs lowercase"),
            // Lookalike character attacks
            (
                "$.test",
                r#"{"test": "value1", "te\u0455t": "value2"}"#,
                "Cyrillic lookalike",
            ),
            (
                "$.admin",
                r#"{"admin": "real", "adm\u0131n": "fake"}"#,
                "Dotless i attack",
            ),
            // Zero-width character attacks
            (
                "$.property",
                r#"{"property": "value1", "prop\u200Berty": "value2"}"#,
                "Zero-width space",
            ),
            (
                "$.key",
                r#"{"key": "value1", "k\uFEFFey": "value2"}"#,
                "Zero-width no-break space",
            ),
        ];

        for (expr, _json_data, _description) in normalization_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(_json_data);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should handle normalization consistently
            assert!(
                elapsed < Duration::from_millis(100),
                "UTF-8 normalization test should be fast: {} ({}ms)",
                _description,
                elapsed.as_millis()
            );

            println!(
                "UTF-8 normalization test '{}': {} results in {}ms",
                _description,
                results.len(),
                elapsed.as_millis()
            );
        }
    }
}

/// RFC 9535 Security - Memory Exhaustion Protection Tests  
#[cfg(test)]
mod memory_exhaustion_tests {
    use super::*;

    #[test]
    fn test_deep_nesting_memory_protection() {
        // Security: Test memory protection for deeply nested structures
        let deep_nesting_tests = vec![
            (100, "Moderate nesting"),
            (500, "Deep nesting"),
            (1000, "Very deep nesting"),
            (2000, "Extreme nesting"),
        ];

        for (depth, _description) in deep_nesting_tests {
            // Create deeply nested JSON
            let mut json = String::from("{");
            for i in 0..depth {
                json.push_str(&format!("\"level{}\":{{", i));
            }
            json.push_str("\"value\":\"found\"");
            for _ in 0..depth {
                json.push('}');
            }
            json.push('}');

            let expr = "$..*"; // Recursive descent through all levels
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should not consume excessive memory or time
            assert!(
                elapsed < Duration::from_secs(10),
                "Deep nesting memory test should not hang: {} depth {} ({}ms)",
                _description,
                depth,
                elapsed.as_millis()
            );

            println!(
                "Deep nesting memory test {}: {} results in {}ms",
                _description,
                results.len(),
                elapsed.as_millis()
            );
        }
    }

    #[test]
    fn test_wide_structure_memory_protection() {
        // Security: Test memory protection for wide structures
        let wide_structure_tests = vec![
            (1000, "Wide object"),
            (5000, "Very wide object"),
            (10000, "Extremely wide object"),
        ];

        for (width, _description) in wide_structure_tests {
            // Create wide JSON object
            let mut json = String::from("{");
            for i in 0..width {
                if i > 0 {
                    json.push(',');
                }
                json.push_str(&format!("\"prop{}\":\"value{}\"", i, i));
            }
            json.push('}');

            let expr = "$..*"; // Access all properties
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should handle wide structures efficiently
            assert!(
                elapsed < Duration::from_secs(5),
                "Wide structure memory test should not hang: {} width {} ({}ms)",
                _description,
                width,
                elapsed.as_millis()
            );

            let expectedresults = width * 2; // property names + values
            assert_eq!(
                results.len(),
                expectedresults,
                "Wide structure should find all {} properties: {} ({})",
                expectedresults,
                _description,
                results.len()
            );
        }
    }

    #[test]
    fn test_large_string_memory_protection() {
        // Security: Test memory protection for large strings
        let large_string_tests = vec![
            (10000, "Large string"),
            (100000, "Very large string"),
            (1000000, "Massive string"),
        ];

        for (size, _description) in large_string_tests {
            // Create JSON with large string value
            let large_value = "A".repeat(size);
            let json = format!(r#"{{"key": "{}"}}"#, large_value);

            let expr = "$.key";
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should handle large strings without memory issues
            assert!(
                elapsed < Duration::from_secs(2),
                "Large string memory test should not hang: {} size {} ({}ms)",
                _description,
                size,
                elapsed.as_millis()
            );

            assert_eq!(
                results.len(),
                1,
                "Large string test should find the value: {} ({})",
                _description,
                results.len()
            );
        }
    }

    #[test]
    fn test_complex_recursive_memory_protection() {
        // Security: Test memory protection for complex recursive patterns
        let recursive_tests = vec![
            ("$..*", 50, "Recursive descent"),
            ("$..*..*", 20, "Double recursive descent"),
            ("$..property..*", 30, "Property recursive descent"),
            ("$..array[*]..*", 25, "Array recursive descent"),
        ];

        for (expr, data_scale, _description) in recursive_tests {
            // Create complex nested structure
            let mut json = String::from(r#"{"root":{"#);
            for i in 0..data_scale {
                json.push_str(&format!(
                    r#""property{}": {{"nested": {{"array": [1, 2, 3], "value": "item{}"}}}},"#,
                    i, i
                ));
            }
            json.push_str(r#""end": "marker"}}"#);

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should handle complex recursion with memory protection
            assert!(
                elapsed < Duration::from_secs(15),
                "Complex recursive memory test should not hang: {} scale {} ({}ms)",
                _description,
                data_scale,
                elapsed.as_millis()
            );

            println!(
                "Complex recursive memory test '{}': {} results in {}ms",
                _description,
                results.len(),
                elapsed.as_millis()
            );
        }
    }
}
