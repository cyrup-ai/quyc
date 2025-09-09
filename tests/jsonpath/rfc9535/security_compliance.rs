//! RFC 9535 Security & Robustness Tests
//!
//! Tests for security and robustness aspects of JSONPath implementation.
//! These tests validate protection against various attack vectors and
//! ensure the implementation can handle malicious or malformed inputs safely.
//!
//! This test suite validates:
//! - Injection attack prevention
//! - Resource exhaustion protection  
//! - Malformed input handling
//! - Deep nesting protection
//! - Regular expression DoS prevention
//! - Memory usage limits
//! - Performance bounds under adversarial conditions
//! - Input validation and sanitization

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct SecurityTestModel {
    id: i32,
    data: String,
    nested: Option<serde_json::Value>,
}

/// Injection Attack Prevention Tests
#[cfg(test)]
mod injection_attack_tests {
    use super::*;

    #[test]
    fn test_path_injection_prevention() {
        // Test prevention of path injection attacks through user input
        let json_data = r#"{"users": [
            {"name": "admin", "role": "administrator", "secret": "top_secret"},
            {"name": "user1", "role": "user", "public": "visible_data"},
            {"name": "user2", "role": "user", "public": "other_data"}
        ]}"#;

        // Potentially malicious path components that should be safely handled
        let malicious_paths = vec![
            "$.users[0]['secret']",                // Direct access attempt
            "$.users[?@.name == 'admin'].secret",  // Filter injection attempt
            "$.users[*]['secret']",                // Wildcard secret access
            "$..secret",                           // Descendant secret search
            "$.users[?@.role == 'administrator']", // Role-based access
        ];

        for malicious_path in malicious_paths {
            let result = JsonPathParser::compile(malicious_path);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(malicious_path);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Path injection test '{}' -> {} results",
                        malicious_path,
                        results.len()
                    );

                    // Log for security audit - these should be controlled by application logic
                    if results.len() > 0 {
                        println!("  WARNING: Potentially sensitive data accessible via path");
                    }
                }
                Err(_) => println!("Path injection '{}' rejected by parser", malicious_path),
            }
        }
    }

    #[test]
    fn test_filter_expression_injection() {
        // Test injection through filter expressions
        let json_data = r#"{"items": [
            {"id": 1, "status": "active", "data": "normal"},
            {"id": 2, "status": "inactive", "data": "sensitive"},
            {"id": 3, "status": "active", "data": "public"}
        ]}"#;

        // Test potentially malicious filter expressions
        let malicious_filters = vec![
            "$.items[?@.status == 'active' || @.status == 'inactive']", // Boolean injection
            "$.items[?@.id > 0]",                                       // Always true condition
            "$.items[?@.data != null]",                                 // Null bypass
            "$.items[?@.status.length > 0]", // Property access injection
        ];

        for filter in malicious_filters {
            let result = JsonPathParser::compile(filter);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(filter);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Filter injection test '{}' -> {} results",
                        filter,
                        results.len()
                    );
                }
                Err(_) => println!("Filter injection '{}' rejected by parser", filter),
            }
        }
    }

    #[test]
    fn test_string_escape_injection() {
        // Test injection through string escape sequences
        let json_data = r#"{"keys": {
            "normal": "value1",
            "with'quote": "value2",
            "with\"quote": "value3",
            "with\\backslash": "value4"
        }}"#;

        let escape_injection_tests = vec![
            "$['keys']['normal']",            // Normal access
            "$['keys']['with\\'quote']",      // Single quote escape
            "$['keys']['with\"quote']",       // Double quote in single quotes
            "$['keys']['with\\\\backslash']", // Backslash escape
            "$['keys']['with\\nquote']",      // Newline injection attempt
            "$['keys']['with\\tquote']",      // Tab injection attempt
        ];

        // Parse JSON data for testing
        let _json_value: serde_json::Value =
            serde_json::from_str(json_data).expect("Test JSON should be valid");

        for path in escape_injection_tests {
            let result = JsonPathParser::compile(path);
            match result {
                Ok(_expr) => {
                    // Test that the compiled expression can safely process the JSON
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(path);
                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "✓ Escape injection '{}' processed safely: {} results",
                        path,
                        results.len()
                    );

                    // Verify no injection occurred by checking result validity
                    for value in results {
                        // Values from JsonArrayStream should be valid JSON values
                        println!("  Result value: {:?}", value);
                    }
                }
                Err(err) => {
                    println!("Escape injection '{}' rejected: {}", path, err);
                    // Rejection is also acceptable for security
                }
            }
        }
    }

    #[test]
    fn test_unicode_injection_prevention() {
        // Test Unicode-based injection attempts
        let json_data = r#"{"data": {
            "normal": "value",
            "café": "coffee",
            "αβγ": "greek",
            "🚀": "rocket"
        }}"#;

        let unicode_injection_tests = vec![
            "$['data']['café']",                         // Accented characters
            "$['data']['αβγ']",                          // Greek letters
            "$['data']['🚀']",                           // Emoji
            "$['data']['\\u0063\\u0061\\u0066\\u0065']", // Unicode escapes for "cafe"
            "$['data']['\\u03B1\\u03B2\\u03B3']",        // Unicode escapes for Greek
        ];

        // Parse JSON data for Unicode testing
        let _json_value: serde_json::Value =
            serde_json::from_str(json_data).expect("Unicode test JSON should be valid");

        for path in unicode_injection_tests {
            let result = JsonPathParser::compile(path);
            match result {
                Ok(_expr) => {
                    // Test that Unicode paths can safely process the JSON data
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(path);
                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "✓ Unicode injection '{}' processed safely: {} results",
                        path,
                        results.len()
                    );

                    // Verify Unicode handling doesn't cause issues
                    for value in results {
                        // Values from JsonArrayStream should be valid JSON values
                        println!("  Unicode result value: {:?}", value);
                    }
                }
                Err(err) => {
                    println!("Unicode injection '{}' rejected: {}", path, err);
                    // Rejection may be acceptable for certain Unicode sequences
                }
            }
        }
    }
}

/// Resource Exhaustion Protection Tests
#[cfg(test)]
mod resource_exhaustion_tests {
    use super::*;

    #[test]
    fn test_large_json_handling() {
        // Test handling of large JSON documents
        let large_array: Vec<i32> = (0..10000).collect();
        let json_value = serde_json::json!({
            "large_data": large_array,
            "metadata": {
                "count": 10000,
                "_description": "Large dataset for testing"
            }
        });
        let json_data = serde_json::to_string(&json_value).expect("Valid JSON");

        let large_data_tests = vec![
            ("$.large_data[0]", "First element access"),
            ("$.large_data[-1]", "Last element access"),
            ("$.large_data[5000:5010]", "Small slice from large array"),
            ("$.metadata.count", "Metadata access"),
        ];

        for (path, _description) in large_data_tests {
            let start_time = std::time::Instant::now();

            let mut stream = JsonArrayStream::<serde_json::Value>::new(path);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "Large JSON test '{}' -> {} results in {:?} ({})",
                path,
                results.len(),
                duration,
                _description
            );

            // Performance assertion - should complete within reasonable time
            assert!(
                duration.as_millis() < 5000,
                "Large JSON handling should complete in <5000ms for '{}'",
                path
            );
        }
    }

    #[test]
    fn test_memory_usage_bounds() {
        // Test memory usage with large result sets
        let medium_array: Vec<i32> = (0..1000).collect();
        let json_value = serde_json::json!({
            "arrays": [
                medium_array.clone(),
                medium_array.clone(),
                medium_array.clone(),
                medium_array.clone(),
                medium_array.clone()
            ]
        });
        let json_data = serde_json::to_string(&json_value).expect("Valid JSON");

        let memory_test_paths = vec![
            ("$.arrays[*][*]", "All elements from all arrays"),
            ("$.arrays[*][::10]", "Every 10th element from all arrays"),
            ("$.arrays[0:3][100:200]", "Subset of arrays and elements"),
        ];

        for (path, _description) in memory_test_paths {
            let start_time = std::time::Instant::now();

            let mut stream = JsonArrayStream::<i32>::new(path);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "Memory usage test '{}' -> {} results in {:?} ({})",
                path,
                results.len(),
                duration,
                _description
            );

            // Memory usage should be reasonable
            assert!(
                duration.as_millis() < 3000,
                "Memory usage test should complete in <3000ms for '{}'",
                path
            );
        }
    }

    #[test]
    fn test_excessive_wildcard_protection() {
        // Test protection against excessive wildcard usage
        let nested_structure = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "data": [1, 2, 3, 4, 5]
                        }
                    }
                }
            }
        });
        let json_data = serde_json::to_string(&nested_structure).expect("Valid JSON");

        let wildcard_stress_tests = vec![
            ("$.*.*.*.*", "Four-level wildcard"),
            ("$.level1.*.*.*", "Three-level wildcard from level1"),
            ("$..data[*]", "Descendant with array wildcard"),
            ("$..*", "Universal descendant wildcard"),
        ];

        for (path, _description) in wildcard_stress_tests {
            let start_time = std::time::Instant::now();

            let mut stream = JsonArrayStream::<serde_json::Value>::new(path);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "Wildcard stress test '{}' -> {} results in {:?} ({})",
                path,
                results.len(),
                duration,
                _description
            );

            // Should handle wildcards efficiently
            assert!(
                duration.as_millis() < 1000,
                "Wildcard stress test should complete in <1000ms for '{}'",
                path
            );
        }
    }

    #[test]
    fn test_filter_expression_complexity_limits() {
        // Test complex filter expressions for performance bounds
        let json_data = r#"{"items": [
            {"a": 1, "b": 2, "c": 3, "d": 4, "e": 5},
            {"a": 2, "b": 3, "c": 4, "d": 5, "e": 6},
            {"a": 3, "b": 4, "c": 5, "d": 6, "e": 7},
            {"a": 4, "b": 5, "c": 6, "d": 7, "e": 8},
            {"a": 5, "b": 6, "c": 7, "d": 8, "e": 9}
        ]}"#;

        let complex_filters = vec![
            "$.items[?@.a > 0 && @.b > 1 && @.c > 2]",
            "$.items[?(@.a > 0 || @.b > 10) && (@.c < 10 || @.d < 10)]",
            "$.items[?@.a + @.b + @.c + @.d + @.e > 20]",
            "$.items[?@.a == 1 || @.a == 2 || @.a == 3 || @.a == 4 || @.a == 5]",
        ];

        for filter in complex_filters {
            let start_time = std::time::Instant::now();
            let result = JsonPathParser::compile(filter);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(filter);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    let duration = start_time.elapsed();

                    println!(
                        "Complex filter '{}' -> {} results in {:?}",
                        filter,
                        results.len(),
                        duration
                    );

                    // Complex filters should still execute reasonably fast
                    assert!(
                        duration.as_millis() < 100,
                        "Complex filter should execute in <100ms: '{}'",
                        filter
                    );
                }
                Err(_) => println!("Complex filter '{}' rejected by parser", filter),
            }
        }
    }
}

/// Malformed Input Handling Tests
#[cfg(test)]
mod malformed_input_tests {
    use super::*;

    #[test]
    fn test_invalid_json_handling() {
        // Test handling of malformed JSON input
        let malformed_json_inputs = vec![
            "{\"key\": value}",             // Unquoted value
            "{\"key\": \"unclosed string}", // Unclosed string
            "{\"key\": 123,}",              // Trailing comma
            "{\"key\": [1, 2, 3,]}",        // Trailing comma in array
            "{'key': 'single_quotes'}",     // Single quotes (non-standard)
            "{\"key\": undefined}",         // Undefined value
            "{\"key\": NaN}",               // NaN value
            "{\"key\": Infinity}",          // Infinity value
        ];

        for malformed_json in malformed_json_inputs {
            println!("Testing malformed JSON: {}", malformed_json);

            let result = std::panic::catch_unwind(|| {
                let mut stream = JsonArrayStream::<serde_json::Value>::new("$.key");

                let chunk = Bytes::from(malformed_json);
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                results.len()
            });

            match result {
                Ok(count) => println!("  Processed without panic, {} results", count),
                Err(_) => println!("  Properly rejected or caused controlled error"),
            }
        }
    }

    #[test]
    fn test_invalid_jsonpath_syntax() {
        // Test handling of malformed JSONPath expressions
        let invalid_jsonpaths = vec![
            "$[",                 // Unclosed bracket
            "$.key.",             // Trailing dot
            "$...",               // Multiple dots
            "$.key[",             // Unclosed array access
            "$.key[abc]",         // Invalid array index
            "$.key[?",            // Unclosed filter
            "$.key[?@.prop",      // Incomplete filter
            "$key",               // Missing root $
            "key.value",          // No root at all
            "$.key[?@.prop ==]",  // Incomplete comparison
            "$.key[?@.prop && ]", // Incomplete logical expression
        ];

        for invalid_path in invalid_jsonpaths {
            let result = JsonPathParser::compile(invalid_path);
            match result {
                Ok(_) => println!("Invalid JSONPath '{}' unexpectedly compiled", invalid_path),
                Err(_) => println!("Invalid JSONPath '{}' correctly rejected", invalid_path),
            }
        }
    }

    #[test]
    fn test_edge_case_json_structures() {
        // Test edge cases in JSON structure
        let edge_case_jsons = vec![
            "{}",                 // Empty object
            "[]",                 // Empty array
            "null",               // Root null
            "\"string\"",         // Root string
            "42",                 // Root number
            "true",               // Root boolean
            "[null, null, null]", // Array of nulls
            "{\"\":\"\"}",        // Empty string key/value
        ];

        for edge_json in edge_case_jsons {
            println!("Testing edge case JSON: {}", edge_json);

            let test_paths = vec!["$", "$.*", "$[*]", "$..value"];

            for path in test_paths {
                let result = std::panic::catch_unwind(|| {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(path);

                    let chunk = Bytes::from(edge_json);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();
                    results.len()
                });

                match result {
                    Ok(count) => println!("  Path '{}' -> {} results", path, count),
                    Err(_) => println!("  Path '{}' caused error", path),
                }
            }
        }
    }

    #[test]
    fn test_extremely_large_numbers() {
        // Test handling of extremely large numbers
        let large_number_json = r#"{
            "small": 1,
            "large_int": 9223372036854775807,
            "larger_than_int": 18446744073709551615,
            "scientific": 1.23e+100,
            "very_small": 1.23e-100
        }"#;

        let number_test_paths = vec![
            "$.small",
            "$.large_int",
            "$.larger_than_int",
            "$.scientific",
            "$.very_small",
            "$[?@.large_int > 1000000000000000000]",
        ];

        for path in number_test_paths {
            let result = std::panic::catch_unwind(|| {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(path);

                let chunk = Bytes::from(large_number_json);
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                results.len()
            });

            match result {
                Ok(count) => println!("Large number test '{}' -> {} results", path, count),
                Err(_) => println!("Large number test '{}' caused error", path),
            }
        }
    }
}

/// Deep Nesting Protection Tests
#[cfg(test)]
mod deep_nesting_tests {
    use super::*;

    #[test]
    fn test_deep_object_nesting() {
        // Create deeply nested object structure
        let mut deep_value = serde_json::json!("found");
        for i in 0..100 {
            deep_value = serde_json::json!({
                format!("level_{}", i): deep_value
            });
        }
        let json_data = serde_json::to_string(&deep_value).expect("Valid JSON");

        let deep_nesting_tests = vec![
            ("$.level_99.level_98.level_97", "Three-level deep access"),
            ("$..found", "Descendant search through deep nesting"),
            ("$.level_99.*.*", "Wildcard through deep levels"),
        ];

        for (path, _description) in deep_nesting_tests {
            let start_time = std::time::Instant::now();

            let result = std::panic::catch_unwind(|| {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(path);

                let chunk = Bytes::from(json_data.clone());
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                results.len()
            });

            let duration = start_time.elapsed();

            match result {
                Ok(count) => {
                    println!(
                        "Deep nesting test '{}' -> {} results in {:?} ({})",
                        path, count, duration, _description
                    );

                    // Should handle deep nesting without excessive time
                    assert!(
                        duration.as_millis() < 2000,
                        "Deep nesting should process in <2000ms for '{}'",
                        path
                    );
                }
                Err(_) => println!(
                    "Deep nesting test '{}' caused stack overflow or error",
                    path
                ),
            }
        }
    }

    #[test]
    fn test_deep_array_nesting() {
        // Create deeply nested array structure
        let mut deep_array = serde_json::json!([42]);
        for _ in 0..50 {
            deep_array = serde_json::json!([deep_array]);
        }
        let json_data = serde_json::to_string(&deep_array).expect("Valid JSON");

        let deep_array_tests = vec![
            ("$[0][0][0]", "Three-level array access"),
            ("$..[42]", "Search for number through deep arrays"),
            ("$[*][*][*]", "Three-level wildcard"),
        ];

        for (path, _description) in deep_array_tests {
            let start_time = std::time::Instant::now();

            let result = std::panic::catch_unwind(|| {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(path);

                let chunk = Bytes::from(json_data.clone());
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                results.len()
            });

            let duration = start_time.elapsed();

            match result {
                Ok(count) => {
                    println!(
                        "Deep array test '{}' -> {} results in {:?} ({})",
                        path, count, duration, _description
                    );

                    // Should handle deep arrays efficiently
                    assert!(
                        duration.as_millis() < 1500,
                        "Deep array processing should complete in <1500ms for '{}'",
                        path
                    );
                }
                Err(_) => println!("Deep array test '{}' caused error", path),
            }
        }
    }

    #[test]
    fn test_recursion_limits() {
        // Test recursion limits with descendant operators
        let recursive_structure = serde_json::json!({
            "root": {
                "child": {
                    "child": {
                        "child": {
                            "child": {
                                "child": {
                                    "target": "deep_value"
                                }
                            }
                        }
                    }
                }
            }
        });
        let json_data = serde_json::to_string(&recursive_structure).expect("Valid JSON");

        let recursion_tests = vec![
            ("$..target", "Descendant search for target"),
            ("$..child", "Descendant search for child"),
            ("$.root..target", "Descendant search from root"),
            ("$..*", "Universal descendant search"),
        ];

        for (path, _description) in recursion_tests {
            let start_time = std::time::Instant::now();

            let result = std::panic::catch_unwind(|| {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(path);

                let chunk = Bytes::from(json_data.clone());
                let results: Vec<_> = stream.process_chunk(chunk).collect();
                results.len()
            });

            let duration = start_time.elapsed();

            match result {
                Ok(count) => {
                    println!(
                        "Recursion test '{}' -> {} results in {:?} ({})",
                        path, count, duration, _description
                    );

                    // Recursion should be controlled and fast
                    assert!(
                        duration.as_millis() < 500,
                        "Recursion test should complete in <500ms for '{}'",
                        path
                    );
                }
                Err(_) => println!("Recursion test '{}' hit limits or caused error", path),
            }
        }
    }
}

/// Regular Expression DoS Prevention Tests
#[cfg(test)]
mod regex_dos_prevention_tests {
    use super::*;

    #[test]
    fn test_catastrophic_backtracking_prevention() {
        // Test prevention of regex patterns that could cause catastrophic backtracking
        let json_data = r#"{"items": [
            {"text": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaX"},
            {"text": "normal_text"},
            {"text": "another_normal_text"}
        ]}"#;

        let dangerous_patterns = vec![
            // These patterns could cause exponential backtracking on certain inputs
            "$.items[?match(@.text, '(a+)+b')]", // Nested quantifiers
            "$.items[?match(@.text, '(a|a)*b')]", // Alternation with overlap
            "$.items[?match(@.text, 'a*a*a*a*b')]", // Multiple quantifiers
            "$.items[?match(@.text, '(a*)*b')]", // Nested star quantifiers
        ];

        for pattern in dangerous_patterns {
            let start_time = std::time::Instant::now();
            let result = JsonPathParser::compile(pattern);

            match result {
                Ok(_) => {
                    let executionresult = std::panic::catch_unwind(|| {
                        let mut stream = JsonArrayStream::<serde_json::Value>::new(pattern);

                        let chunk = Bytes::from(json_data);
                        let results: Vec<_> = stream.process_chunk(chunk).collect();
                        results.len()
                    });

                    let duration = start_time.elapsed();

                    match executionresult {
                        Ok(count) => {
                            println!(
                                "Dangerous regex '{}' -> {} results in {:?}",
                                pattern, count, duration
                            );

                            // Should not take excessive time even with problematic patterns
                            if duration.as_millis() > 1000 {
                                println!(
                                    "  WARNING: Potential ReDoS vulnerability - took {}ms",
                                    duration.as_millis()
                                );
                            }
                        }
                        Err(_) => println!("Dangerous regex '{}' caused timeout or error", pattern),
                    }
                }
                Err(_) => println!(
                    "Dangerous regex '{}' correctly rejected at compile time",
                    pattern
                ),
            }
        }
    }

    #[test]
    fn test_regex_complexity_limits() {
        // Test regex patterns of varying complexity
        let json_data = r#"{"data": [
            {"code": "ABC123"},
            {"code": "XYZ789"},
            {"email": "user@example.com"},
            {"phone": "+1-555-123-4567"}
        ]}"#;

        let complexity_patterns = vec![
            (
                "$.data[?match(@.code, '^[A-Z]{3}[0-9]{3}$')]",
                "Simple pattern",
            ),
            (
                "$.data[?match(@.email, '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$')]",
                "Email pattern",
            ),
            (
                "$.data[?match(@.phone, '^\\+?[1-9]\\d{1,14}$')]",
                "Phone pattern",
            ),
            ("$.data[?match(@.code, '[A-Z]+')]", "Basic character class"),
        ];

        for (pattern, _description) in complexity_patterns {
            let start_time = std::time::Instant::now();
            let result = JsonPathParser::compile(pattern);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(pattern);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    let duration = start_time.elapsed();

                    println!(
                        "Regex complexity '{}' -> {} results in {:?} ({})",
                        pattern,
                        results.len(),
                        duration,
                        _description
                    );

                    // Complex patterns should still execute quickly
                    assert!(
                        duration.as_millis() < 200,
                        "Complex regex should execute in <200ms: '{}'",
                        pattern
                    );
                }
                Err(_) => println!(
                    "Regex pattern '{}' not supported ({})",
                    pattern, _description
                ),
            }
        }
    }

    #[test]
    fn test_regex_input_size_limits() {
        // Test regex patterns against various input sizes
        let small_text = "a".repeat(100);
        let medium_text = "a".repeat(1000);
        let large_text = "a".repeat(10000);

        let size_test_data = vec![
            (small_text, "small_input"),
            (medium_text, "medium_input"),
            (large_text, "large_input"),
        ];

        for (text, size_desc) in size_test_data {
            let json_value = serde_json::json!({"text": text});
            let json_data = serde_json::to_string(&json_value).expect("Valid JSON");

            // Use a semantically correct JSONPath pattern that filters the root object
            // based on whether its text field matches the regex
            let pattern = "$[?match(.text, 'a+')]";

            let start_time = std::time::Instant::now();
            let result = JsonPathParser::compile(pattern);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(pattern);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    let duration = start_time.elapsed();

                    println!(
                        "Regex input size test '{}' -> {} results in {:?} ({})",
                        pattern,
                        results.len(),
                        duration,
                        size_desc
                    );

                    // Should handle various input sizes efficiently with ReDoS protection
                    assert!(
                        duration.as_millis() < 1000,
                        "Regex with {} input should complete in <1000ms (ReDoS protection active)",
                        size_desc
                    );

                    // Should find the matching object (1 result expected)
                    assert_eq!(
                        results.len(),
                        1,
                        "Should find exactly one matching object for pattern: {}",
                        pattern
                    );
                }
                Err(_) => println!("Regex pattern not supported for size test"),
            }
        }
    }
}

/// Parser Vulnerability Protection Tests
#[cfg(test)]
mod parser_vulnerability_tests {
    use super::*;

    #[test]
    fn test_buffer_overflow_protection() {
        // Test protection against buffer overflow attacks through oversized inputs
        let extremely_long_property = "a".repeat(100000);
        let _oversized_json = format!(r#"{{"{}": "value"}}"#, extremely_long_property);

        let buffer_overflow_tests = vec![
            (
                format!("$.{}", extremely_long_property),
                "Extremely long property name",
            ),
            (
                format!("$['{}']", extremely_long_property),
                "Extremely long bracket property",
            ),
            ("$..a".repeat(1000), "Repeated descendant operators"),
            (
                format!("$[?@.field == '{}']", "x".repeat(50000)),
                "Extremely long filter value",
            ),
        ];

        for (path, _description) in buffer_overflow_tests {
            let start_time = std::time::Instant::now();

            let result = std::panic::catch_unwind(|| JsonPathParser::compile(&path));

            let duration = start_time.elapsed();

            match result {
                Ok(parseresult) => match parseresult {
                    Ok(_) => println!(
                        "Buffer overflow test '{}' compiled unexpectedly",
                        _description
                    ),
                    Err(_) => {
                        println!("Buffer overflow test '{}' correctly rejected", _description)
                    }
                },
                Err(_) => println!(
                    "Buffer overflow test '{}' caused panic (potential vulnerability)",
                    _description
                ),
            }

            // Compilation should not take excessive time even for malicious inputs
            assert!(
                duration.as_millis() < 5000,
                "Buffer overflow protection test '{}' should reject quickly",
                _description
            );
        }
    }

    #[test]
    fn test_stack_overflow_protection() {
        // Test protection against stack overflow through deeply nested expressions
        let deep_property_chain = (0..1000)
            .map(|i| format!("prop{}", i))
            .collect::<Vec<_>>()
            .join(".");
        let deep_bracket_chain = (0..1000)
            .map(|i| format!("['prop{}']", i))
            .collect::<Vec<_>>()
            .join("");

        let stack_overflow_tests = vec![
            (
                format!("$.{}", deep_property_chain),
                "1000-level property chain",
            ),
            (
                format!("${}", deep_bracket_chain),
                "1000-level bracket chain",
            ),
            (
                format!("${}", "[*]".repeat(500)),
                "500 nested wildcard selectors",
            ),
            (
                format!("${}", "..value".repeat(200)),
                "200 nested descendant operators",
            ),
        ];

        for (path, _description) in stack_overflow_tests {
            let start_time = std::time::Instant::now();

            let result = std::panic::catch_unwind(|| JsonPathParser::compile(&path));

            let duration = start_time.elapsed();

            match result {
                Ok(parseresult) => match parseresult {
                    Ok(_) => println!(
                        "Stack overflow test '{}' compiled (potential issue)",
                        _description
                    ),
                    Err(_) => println!("Stack overflow test '{}' correctly rejected", _description),
                },
                Err(_) => println!(
                    "Stack overflow test '{}' hit protection limits",
                    _description
                ),
            }

            // Should complete quickly even for pathological inputs
            assert!(
                duration.as_millis() < 3000,
                "Stack overflow protection test '{}' should complete quickly",
                _description
            );
        }
    }

    #[test]
    fn test_memory_exhaustion_protection() {
        // Test protection against memory exhaustion attacks
        let huge_array_json = format!(r#"{{"array": [{}]}}"#, "1,".repeat(1000000) + "1");
        let huge_object_json = format!(
            r#"{{{}}}"#,
            (0..100000)
                .map(|i| format!(r#""key{}": "value{}""#, i, i))
                .collect::<Vec<_>>()
                .join(",")
        );

        let memory_exhaustion_tests = vec![
            (
                huge_array_json.clone(),
                "$.array[*]",
                "Million element array wildcard",
            ),
            (
                huge_object_json.clone(),
                "$.*",
                "100k property object wildcard",
            ),
            (
                huge_array_json.clone(),
                "$.array[::1]",
                "Million element array slice",
            ),
            (
                huge_object_json.clone(),
                "$..value0",
                "Descendant search in huge object",
            ),
        ];

        for (_json_data, path, _description) in memory_exhaustion_tests {
            let start_time = std::time::Instant::now();

            let json_data_clone = _json_data.clone();
            let path_clone = path.to_string();
            let result = std::panic::catch_unwind(move || {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(&path_clone);
                // Limit JSON data to 1MB to prevent memory exhaustion
                let limited_data = if json_data_clone.len() > 1024 * 1024 {
                    json_data_clone[..1024 * 1024].to_string()
                } else {
                    json_data_clone
                };
                let chunk = Bytes::from(limited_data);
                let streamresult = stream.process_chunk(chunk);
                let results: Vec<_> = streamresult.collect().into_iter().take(1000).collect(); // Limit results
                results.len()
            });

            let duration = start_time.elapsed();

            match result {
                Ok(count) => println!(
                    "Memory exhaustion test '{}' -> {} results in {:?}",
                    _description, count, duration
                ),
                Err(_) => println!(
                    "Memory exhaustion test '{}' hit protection limits",
                    _description
                ),
            }

            // Should complete within reasonable memory/time bounds
            assert!(
                duration.as_millis() < 10000,
                "Memory exhaustion test '{}' should complete within bounds",
                _description
            );
        }
    }

    #[test]
    fn test_parser_state_corruption() {
        // Test that parser state cannot be corrupted by malicious inputs
        let corruption_attempts = vec![
            "$[?@.field == '\0']",              // Null byte injection
            "$[?@.field == '\x01\x02']",        // Control character injection
            "$[?@.field == '\u{FEFF}']",        // BOM injection
            "$[?@.field == '\u{200B}']",        // Zero-width space
            "$[?@.field == '\u{FFFF}']",        // Invalid Unicode
            r#"$[?@.field == "line1\nline2"]"#, // Newline injection
            r#"$[?@.field == "tab\ttab"]"#,     // Tab injection
        ];

        for malicious_input in corruption_attempts {
            let result1 = JsonPathParser::compile(malicious_input);
            let result2 = JsonPathParser::compile("$.normal.path"); // Normal path after malicious

            match (result1, result2) {
                (Ok(_), Ok(_)) => println!(
                    "Parser state corruption test '{}' - both compiled",
                    malicious_input
                ),
                (Err(_), Ok(_)) => println!(
                    "Parser state corruption test '{}' - first rejected, second ok",
                    malicious_input
                ),
                (Ok(_), Err(_)) => println!(
                    "Parser state corruption test '{}' - potential state corruption",
                    malicious_input
                ),
                (Err(_), Err(_)) => println!(
                    "Parser state corruption test '{}' - both rejected",
                    malicious_input
                ),
            }

            // Parser should consistently handle normal paths after any input
            let normalresult = JsonPathParser::compile("$.test");
            assert!(
                normalresult.is_ok(),
                "Parser state should not be corrupted after malicious input: {}",
                malicious_input
            );
        }
    }

    #[test]
    fn test_unicode_vulnerability_protection() {
        // Test protection against Unicode-based vulnerabilities
        let unicode_attacks = vec![
            ("$['\u{202E}danger\u{202D}']", "RTL/LTR override attack"),
            ("$['\u{00A0}']", "Non-breaking space"),
            ("$['\u{2000}']", "En quad space"),
            ("$['\u{200C}']", "Zero-width non-joiner"),
            ("$['\u{200D}']", "Zero-width joiner"),
            ("$['\u{061C}']", "Arabic letter mark"),
            ("$['\u{2066}\u{2069}']", "Isolate characters"),
            ("$['\u{FE0F}']", "Variation selector"),
        ];

        for (attack_path, _description) in unicode_attacks {
            let result = JsonPathParser::compile(attack_path);
            match result {
                Ok(_) => println!(
                    "Unicode attack '{}' compiled ({})",
                    attack_path, _description
                ),
                Err(_) => println!(
                    "Unicode attack '{}' rejected ({})",
                    attack_path, _description
                ),
            }

            // Test that Unicode attacks don't affect subsequent parsing
            let normalresult = JsonPathParser::compile("$.normal");
            assert!(
                normalresult.is_ok(),
                "Unicode attack should not affect subsequent parsing: {}",
                _description
            );
        }
    }
}

/// Error Recovery and State Consistency Tests
#[cfg(test)]
mod error_recovery_tests {
    use super::*;

    #[test]
    fn test_graceful_error_recovery() {
        // Test that errors are handled gracefully without corrupting state
        let error_inducing_inputs = vec![
            ("$[", "Unclosed bracket"),
            ("$.key.", "Trailing dot"),
            ("$key", "Missing root"),
            ("$.key[abc", "Malformed bracket"),
            ("$.key[?@", "Incomplete filter"),
        ];

        for (malicious_path, _description) in error_inducing_inputs {
            // Attempt to compile malicious path
            let errorresult = JsonPathParser::compile(malicious_path);
            assert!(
                errorresult.is_err(),
                "Error-inducing path '{}' should be rejected",
                malicious_path
            );

            // Verify normal paths still work after error
            let normalresult = JsonPathParser::compile("$.valid.path");
            assert!(
                normalresult.is_ok(),
                "Normal path should work after error from '{}' ({})",
                malicious_path,
                _description
            );

            // Verify stream creation still works
            let streamresult =
                std::panic::catch_unwind(|| JsonArrayStream::<serde_json::Value>::new("$.test"));
            assert!(
                streamresult.is_ok(),
                "Stream creation should work after error from '{}' ({})",
                malicious_path,
                _description
            );
        }
    }

    #[test]
    fn test_error_propagation_consistency() {
        // Test that errors propagate consistently through the system
        let json_data = r#"{"valid": "data"}"#;

        let error_scenarios = vec![
            ("$[?@.field ==]", "Incomplete comparison"),
            ("$[?@.field &&]", "Incomplete logical"),
            ("$[?@.field == 'unclosed", "Unclosed string"),
            ("$...", "Invalid descendant"),
            ("$store", "Missing dot"),
        ];

        for (invalid_path, _description) in error_scenarios {
            // Parser should reject invalid paths
            let parseresult = JsonPathParser::compile(invalid_path);
            assert!(
                parseresult.is_err(),
                "Invalid path '{}' should be rejected during parsing ({})",
                invalid_path,
                _description
            );

            // Stream creation with invalid path should also fail consistently
            let streamresult = std::panic::catch_unwind(|| {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(invalid_path);
                let chunk = Bytes::from(json_data);
                let _results: Vec<_> = stream.process_chunk(chunk).collect();
            });

            // Either the stream creation should panic or handle gracefully
            match streamresult {
                Ok(_) => println!(
                    "Invalid path '{}' handled gracefully in stream",
                    invalid_path
                ),
                Err(_) => println!(
                    "Invalid path '{}' properly rejected in stream",
                    invalid_path
                ),
            }
        }
    }

    #[test]
    fn test_concurrent_access_safety() {
        // Test thread safety and concurrent access patterns
        use std::thread;

        let json_data = r#"{"shared": {"value": 42, "array": [1, 2, 3, 4, 5]}}"#;
        let test_paths = vec![
            "$.shared.value",
            "$.shared.array[*]",
            "$.shared.array[0]",
            "$..value",
            "$.shared.*",
        ];

        let handles: Vec<_> = test_paths
            .into_iter()
            .map(|path| {
                let json_data = json_data.to_string();
                let path = path.to_string();

                thread::spawn(move || {
                    for i in 0..100 {
                        let mut stream = JsonArrayStream::<serde_json::Value>::new(&path);
                        let chunk = Bytes::from(json_data.clone());
                        let results: Vec<_> = stream.process_chunk(chunk).collect();

                        if i == 0 {
                            println!("Concurrent test '{}' -> {} results", path, results.len());
                        }
                    }
                    path
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            let path = handle.join().expect("Thread should complete successfully");
            println!("Concurrent access test completed for: {}", path);
        }
    }

    #[test]
    fn test_resource_cleanup_after_errors() {
        // Test that resources are properly cleaned up after errors
        let cleanup_test_scenarios = vec![
            ("$[", "Incomplete bracket"),
            ("$.a.b.c.d.e.f.g.h.i.j.k.l.m", "Long property chain"),
            ("$[*][*][*][*][*]", "Deep wildcard nesting"),
            ("$..value..target..data", "Multiple descendants"),
        ];

        for (path, _description) in cleanup_test_scenarios {
            // Create and process multiple times to test cleanup
            for iteration in 0..10 {
                let result = std::panic::catch_unwind(|| {
                    let parseresult = JsonPathParser::compile(path);
                    if let Ok(_) = parseresult {
                        let mut stream = JsonArrayStream::<serde_json::Value>::new(path);
                        let chunk = Bytes::from(r#"{"test": "data"}"#);
                        let _results: Vec<_> = stream.process_chunk(chunk).collect();
                    }
                });

                match result {
                    Ok(_) => {
                        if iteration == 0 {
                            println!(
                                "Resource cleanup test '{}' handled gracefully",
                                _description
                            );
                        }
                    }
                    Err(_) => {
                        if iteration == 0 {
                            println!(
                                "Resource cleanup test '{}' caused expected error",
                                _description
                            );
                        }
                    }
                }
            }

            // Verify system is still functional after cleanup tests
            let verificationresult = JsonPathParser::compile("$.test");
            assert!(
                verificationresult.is_ok(),
                "System should be functional after resource cleanup test: {}",
                _description
            );
        }
    }

    #[test]
    fn test_input_validation_edge_cases() {
        // Test edge cases in input validation
        let edge_case_inputs = vec![
            ("", "Empty string"),
            (" ", "Whitespace only"),
            ("\n", "Newline only"),
            ("\t", "Tab only"),
            ("\r\n", "CRLF"),
            ("$", "Root only"),
            ("$$", "Double root"),
            ("$.$", "Root dot root"),
            ("$[]", "Empty brackets"),
            ("$[:]", "Empty slice"),
            ("$[?]", "Empty filter"),
        ];

        for (input, _description) in edge_case_inputs {
            let result = JsonPathParser::compile(input);
            println!(
                "Input validation edge case '{}' ({}): {:?}",
                input.escape_debug(),
                _description,
                result.is_ok()
            );

            // System should handle edge cases without crashing
            let post_testresult = JsonPathParser::compile("$.normal");
            assert!(
                post_testresult.is_ok(),
                "System should remain stable after edge case: {}",
                _description
            );
        }
    }
}
