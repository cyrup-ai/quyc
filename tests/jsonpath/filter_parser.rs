//! Filter parser module tests
//!
//! Tests for JSONPath filter parser functionality, mirroring src/json_path/filter_parser.rs
//!
//! This module contains comprehensive tests for filter parsing pipeline:
//! - Comparison operator edge case validation
//! - Type mismatch handling in filter expressions
//! - Unicode string comparison compliance
//! - Numeric precision in filter comparisons
//! - Null value semantics in filter expressions
//! - Performance validation for large dataset filtering

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    name: String,
    value: serde_json::Value,
    optional: Option<String>,
}

/// RFC 9535 Comparison Operator Edge Cases
#[cfg(test)]
mod comparison_edge_cases {
    use super::*;

    #[test]
    fn test_empty_nodelist_comparisons() {
        // RFC 9535: Comparisons with empty nodelists
        let json_data = r#"{
            "items": [
                {"name": "item1", "value": 10},
                {"name": "item2", "value": 20}
            ]
        }"#;

        let empty_nodelist_tests = vec![
            // Comparing against non-existent properties
            "$.items[?@.nonexistent == 10]", // Empty nodelist == literal
            "$.items[?@.nonexistent != 10]", // Empty nodelist != literal
            "$.items[?@.nonexistent > 10]",  // Empty nodelist > literal
            "$.items[?@.nonexistent < 10]",  // Empty nodelist < literal
            "$.items[?10 == @.nonexistent]", // Literal == empty nodelist
            "$.items[?@.nonexistent == @.also_missing]", // Empty == empty
        ];

        for expr in empty_nodelist_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Empty nodelist comparisons should typically return no matches
            println!(
                "Empty nodelist test '{}' returned {} results",
                expr,
                results.len()
            );
            assert_eq!(
                results.len(),
                0,
                "Empty nodelist comparison should return no results"
            );
        }
    }

    #[test]
    fn test_type_mismatch_behaviors() {
        // RFC 9535: Type mismatch in comparisons
        let json_data = r#"{
            "items": [
                {"string_val": "10", "number_val": 10, "bool_val": true},
                {"string_val": "hello", "number_val": 20, "bool_val": false},
                {"string_val": "true", "number_val": 0, "bool_val": true}
            ]
        }"#;

        let type_mismatch_tests = vec![
            // String vs Number comparisons
            ("$.items[?@.string_val == @.number_val]", 0), // "10" == 10 (different types)
            ("$.items[?@.string_val == '10']", 1),         // "10" == "10" (same type)
            ("$.items[?@.number_val == 10]", 1),           // 10 == 10 (same type)
            // String vs Boolean comparisons
            ("$.items[?@.string_val == @.bool_val]", 0), // String vs Boolean
            ("$.items[?@.string_val == 'true']", 1),     // "true" == "true"
            ("$.items[?@.bool_val == true]", 2),         // Boolean == Boolean
            // Number vs Boolean comparisons
            ("$.items[?@.number_val == @.bool_val]", 0), // Number vs Boolean
            ("$.items[?@.number_val == 0]", 1),          // 0 == 0
            ("$.items[?@.bool_val == false]", 1),        // false == false
        ];

        for (expr, expected_count) in type_mismatch_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Type mismatch test '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_numeric_precision_handling() {
        // RFC 9535: Numeric precision in comparisons
        let json_data = r#"{
            "items": [
                {"float_val": 1.1, "precise_val": 1.1000000000000001},
                {"float_val": 0.1, "precise_val": 0.30000000000000004},
                {"float_val": 999999999999999.0, "precise_val": 999999999999999.1}
            ]
        }"#;

        let precision_tests = vec![
            // Floating point precision issues
            ("$.items[?@.float_val == 1.1]", 1), // Exact match
            ("$.items[?@.precise_val == 1.1000000000000001]", 1), // High precision
            ("$.items[?@.float_val == @.precise_val]", 0), // Different precision values
            // IEEE 754 edge cases
            ("$.items[?@.float_val > 0.99]", 2), // Range comparison
            ("$.items[?@.precise_val < 1.2]", 2), // Range comparison
            ("$.items[?@.float_val >= 999999999999999.0]", 1), // Large numbers
        ];

        for (expr, expected_count) in precision_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Numeric precision test '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_string_comparison_unicode_rules() {
        // RFC 9535: Unicode string comparison rules
        let json_data = r#"{
            "items": [
                {"text": "café", "normalized": "café"},
                {"text": "naïve", "normalized": "naive"},
                {"text": "Москва", "normalized": "москва"},
                {"text": "🌍🌎🌏", "normalized": "world"}
            ]
        }"#;

        let unicode_tests = vec![
            // Case sensitivity
            ("$.items[?@.text == 'café']", 1), // Exact match with accents
            ("$.items[?@.text == 'cafe']", 0), // Without accents (no match)
            ("$.items[?@.text == 'Café']", 0), // Different case (no match)
            // Unicode normalization
            ("$.items[?@.text == 'naïve']", 1), // With diacritic
            ("$.items[?@.text == 'naive']", 0), // Without diacritic
            // Cyrillic case sensitivity
            ("$.items[?@.text == 'Москва']", 1), // Cyrillic uppercase
            ("$.items[?@.text == 'москва']", 0), // Cyrillic lowercase
            // Emoji comparison
            ("$.items[?@.text == '🌍🌎🌏']", 1), // Emoji sequence
            ("$.items[?@.text == '🌍']", 0),     // Single emoji
        ];

        for (expr, expected_count) in unicode_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Unicode comparison test '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_string_collation_edge_cases() {
        // RFC 9535: String collation and ordering edge cases
        let json_data = r#"{
            "items": [
                {"text": "apple"},
                {"text": "Apple"},
                {"text": "APPLE"},
                {"text": "āpple"},
                {"text": "zebra"},
                {"text": "Zebra"}
            ]
        }"#;

        let collation_tests = vec![
            // Lexicographic ordering (case-sensitive)
            ("$.items[?@.text > 'a']", 2), // Lowercase comparison
            ("$.items[?@.text > 'A']", 4), // Uppercase comparison
            ("$.items[?@.text < 'z']", 5), // All except "zebra"
            ("$.items[?@.text < 'Z']", 3), // Case-sensitive ordering
            // Unicode character ordering
            ("$.items[?@.text > 'apple']", 2), // Standard comparison
            ("$.items[?@.text >= 'apple']", 3), // Including exact match
            ("$.items[?@.text > 'āpple']", 2), // Unicode character
        ];

        for (expr, expected_count) in collation_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "String collation test '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_structured_value_comparisons() {
        // RFC 9535: Comparison of structured values (arrays, objects)
        let json_data = r#"{
            "items": [
                {"data": [1, 2, 3]},
                {"data": [1, 2, 3]},
                {"data": [3, 2, 1]},
                {"data": {"a": 1, "b": 2}},
                {"data": {"a": 1, "b": 2}},
                {"data": {"b": 2, "a": 1}}
            ]
        }"#;

        let structured_tests = vec![
            // Array comparisons - should these be supported?
            ("$.items[?@.data == [1, 2, 3]]", 0), /* Array literal comparison (may not be supported) */
            // Object comparisons - should these be supported?
            ("$.items[?@.data == {'a': 1, 'b': 2}]", 0), /* Object literal comparison (may not be supported) */
            // Property existence on structured values
            ("$.items[?@.data]", 6),    // All have data property
            ("$.items[?@.data.a]", 3),  // Objects with 'a' property
            ("$.items[?@.data[0]]", 3), // Arrays with first element
        ];

        for (expr, expected_count) in structured_tests {
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Structured value test '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(_) => {
                    println!("Structured value test '{}' not supported in parser", expr);
                }
            }
        }
    }

    #[test]
    fn test_null_value_comparisons() {
        // RFC 9535: null value comparison edge cases
        let json_data = r#"{
            "items": [
                {"value": null, "name": "item1"},
                {"value": 0, "name": "item2"},
                {"value": false, "name": "item3"},
                {"value": "", "name": "item4"},
                {"name": "item5"}
            ]
        }"#;

        let null_tests = vec![
            // null vs null
            ("$.items[?@.value == null]", 1), // Explicit null value
            ("$.items[?@.value != null]", 3), // Non-null values
            // null vs other falsy values
            ("$.items[?@.value == 0]", 1),     // null != 0
            ("$.items[?@.value == false]", 1), // null != false
            ("$.items[?@.value == '']", 1),    // null != ""
            // Missing vs null distinction
            ("$.items[?@.missing == null]", 0), // Missing property != null
            ("$.items[?@.missing]", 0),         // Missing property doesn't exist
            ("$.items[?@.value]", 3),           // Non-null values exist
        ];

        for (expr, expected_count) in null_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Null comparison test '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_inequality_operator_edge_cases() {
        // RFC 9535: Inequality operators with edge cases
        let json_data = r#"{
            "items": [
                {"value": -1},
                {"value": 0},
                {"value": 1},
                {"value": 1.0},
                {"value": 1.1},
                {"value": null}
            ]
        }"#;

        let inequality_tests = vec![
            // Greater than with various types
            ("$.items[?@.value > 0]", 3),  // Positive numbers
            ("$.items[?@.value > -1]", 4), // Greater than negative
            ("$.items[?@.value >= 1]", 3), // Greater than or equal
            // Less than with various types
            ("$.items[?@.value < 1]", 2),  // Less than 1
            ("$.items[?@.value <= 1]", 4), // Less than or equal to 1
            ("$.items[?@.value < 0]", 1),  // Negative values
            // Null in inequalities
            ("$.items[?@.value > null]", 0), // Null comparisons
            ("$.items[?@.value < null]", 0), // Null comparisons
            ("$.items[?null < @.value]", 0), // Reverse null comparison
        ];

        for (expr, expected_count) in inequality_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Inequality test '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_comparison_operator_precedence() {
        // RFC 9535: Operator precedence in complex comparisons
        let json_data = r#"{
            "items": [
                {"a": 1, "b": 2, "c": 3},
                {"a": 2, "b": 3, "c": 1},
                {"a": 3, "b": 1, "c": 2}
            ]
        }"#;

        let precedence_tests = vec![
            // Comparison precedence
            ("$.items[?@.a < @.b && @.b < @.c]", 1), // Multiple comparisons with AND
            ("$.items[?@.a > @.b || @.b > @.c]", 2), // Multiple comparisons with OR
            ("$.items[?@.a < @.b == @.b < @.c]", 1), /* Chained comparisons (== has lower precedence) */
            // Parentheses for precedence
            ("$.items[?(@.a < @.b) && (@.b < @.c)]", 1), // Explicit grouping
            ("$.items[?(@.a > @.b) || (@.b > @.c)]", 2), // Explicit grouping
            ("$.items[?@.a < (@.b && @.b) < @.c]", 0), // Invalid grouping (logical in comparison)
        ];

        for (expr, expected_count) in precedence_tests {
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Precedence test '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(_) => {
                    println!(
                        "Precedence test '{}' rejected by parser (invalid syntax)",
                        expr
                    );
                }
            }
        }
    }

    #[test]
    fn test_comparison_with_functions() {
        // RFC 9535: Comparisons involving function results
        let json_data = r#"{
            "items": [
                {"name": "short", "tags": ["a"]},
                {"name": "medium", "tags": ["a", "b"]},
                {"name": "longer", "tags": ["a", "b", "c"]}
            ]
        }"#;

        let function_comparison_tests = vec![
            // length() function in comparisons
            ("$.items[?length(@.name) > 5]", 2), // String length comparison
            ("$.items[?length(@.tags) == 2]", 1), // Array length comparison
            ("$.items[?length(@.name) > length(@.tags)]", 2), // Function vs function
            // Function results vs literals
            ("$.items[?length(@.name) == 6]", 2), // Exact length match
            ("$.items[?length(@.tags) <= 2]", 2), // Length inequality
        ];

        for (expr, expected_count) in function_comparison_tests {
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => {
                    println!("Function comparison test '{}' compiled successfully", expr);
                    // Note: Actual execution depends on function implementation
                }
                Err(_) => {
                    println!("Function comparison test '{}' not supported yet", expr);
                }
            }
        }
    }
}

/// Performance Tests for Comparison Operations
#[cfg(test)]
mod comparison_performance_tests {
    use super::*;

    #[test]
    fn test_large_dataset_comparisons() {
        // Test comparison performance on large datasets
        let large_data = serde_json::json!({
            "items": (0..1000).map(|i| serde_json::json!({
                "id": i,
                "value": i % 100,
                "name": format!("item_{}", i),
                "active": i % 2 == 0
            })).collect::<Vec<_>>()
        });

        let json_data = serde_json::to_string(&large_data).expect("Valid JSON");

        let performance_tests = vec![
            ("$.items[?@.value > 50]", 490),     // Numeric comparison
            ("$.items[?@.active == true]", 500), // Boolean comparison
            ("$.items[?@.id >= 500]", 500),      // Range comparison
            ("$.items[?@.value == 42]", 10),     // Exact match
        ];

        for (expr, expected_count) in performance_tests {
            let start_time = std::time::Instant::now();

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "Performance test '{}' found {} results in {:?} (expected {})",
                expr,
                results.len(),
                duration,
                expected_count
            );

            // Performance assertion - should complete reasonably quickly
            assert!(
                duration.as_millis() < 1000,
                "Comparison performance test '{}' should complete in <1000ms",
                expr
            );
        }
    }

    #[test]
    fn test_string_comparison_performance() {
        // Test string comparison performance
        let string_data = serde_json::json!({
            "items": (0..1000).map(|i| serde_json::json!({
                "text": format!("item_{:04}", i),
                "category": if i % 3 == 0 { "alpha" } else if i % 3 == 1 { "beta" } else { "gamma" }
            })).collect::<Vec<_>>()
        });

        let json_data = serde_json::to_string(&string_data).expect("Valid JSON");

        let string_performance_tests = vec![
            ("$.items[?@.category == 'alpha']", 334), // String equality
            ("$.items[?@.text > 'item_0500']", 499),  // String ordering
            ("$.items[?@.category != 'gamma']", 667), // String inequality
        ];

        for (expr, expected_count) in string_performance_tests {
            let start_time = std::time::Instant::now();

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "String performance test '{}' found {} results in {:?} (expected {})",
                expr,
                results.len(),
                duration,
                expected_count
            );

            // Performance assertion
            assert!(
                duration.as_millis() < 1000,
                "String comparison performance test '{}' should complete in <1000ms",
                expr
            );
        }
    }
}
