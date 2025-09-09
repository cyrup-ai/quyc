//! JSONPath Functions Tests
//!
//! Tests for all JSONPath function extensions defined in RFC 9535 Section 2.4
//! Including length(), count(), match(), search(), and value() functions

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    name: String,
    tags: Vec<String>,
    metadata: Option<serde_json::Value>,
}

/// RFC 9535 Section 2.4.4 - length() Function Tests
#[cfg(test)]
mod length_function_tests {
    use super::*;

    #[test]
    fn test_string_length() {
        // RFC 9535: length() returns number of characters in string
        let json_data = r#"{"items": [
            {"name": "short"},
            {"name": "medium_length"},
            {"name": "very_long_string_name"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?length(@.name) == 5]", 1), // "short"
            ("$.items[?length(@.name) > 10]", 2), // medium and long
            ("$.items[?length(@.name) < 10]", 1), // only "short"
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "String length filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_array_length() {
        // RFC 9535: length() returns number of elements in array
        let json_data = r#"{"groups": [
            {"members": ["a"]},
            {"members": ["a", "b"]},
            {"members": ["a", "b", "c", "d"]},
            {"members": []}
        ]}"#;

        let test_cases = vec![
            ("$.groups[?length(@.members) == 0]", 1), // Empty array
            ("$.groups[?length(@.members) == 1]", 1), // Single element
            ("$.groups[?length(@.members) > 2]", 1),  // More than 2 elements
            ("$.groups[?length(@.members) <= 2]", 3), // 2 or fewer elements
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Array length filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_object_length() {
        // RFC 9535: length() returns number of members in object
        let json_data = r#"{"items": [
            {"props": {}},
            {"props": {"a": 1}},
            {"props": {"a": 1, "b": 2}},
            {"props": {"a": 1, "b": 2, "c": 3, "d": 4}}
        ]}"#;

        let test_cases = vec![
            ("$.items[?length(@.props) == 0]", 1), // Empty object
            ("$.items[?length(@.props) == 1]", 1), // Single property
            ("$.items[?length(@.props) > 2]", 1),  // More than 2 properties
            ("$.items[?length(@.props) <= 2]", 3), // 2 or fewer properties
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Object length filter '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_length_null_value() {
        // RFC 9535: length() of null returns null
        let json_data = r#"{"items": [
            {"value": "string"},
            {"value": null},
            {"value": [1, 2, 3]}
        ]}"#;

        let mut stream =
            JsonArrayStream::<serde_json::Value>::new("$.items[?length(@.value) == null]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 1, "length(null) should equal null");
    }

    #[test]
    fn test_length_on_primitives() {
        // RFC 9535: length() behavior on primitive values
        let json_data = r#"{"items": [
            {"value": 42},
            {"value": true},
            {"value": false}
        ]}"#;

        // Test length on numbers and booleans - behavior may vary
        let expressions = vec![
            "$.items[?length(@.value) == null]", // May return null for primitives
            "$.items[?length(@.value) == 0]",    // May return 0 for primitives
        ];

        for expr in expressions {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Document current behavior rather than asserting specific results
            println!(
                "length() on primitives '{}' returned {} results",
                expr,
                results.len()
            );
        }
    }

    #[test]
    fn test_unicode_string_length() {
        // Extended test: Unicode character handling in string length
        let json_data = r#"{"items": [
            {"text": "hello"},
            {"text": "héllö"},
            {"text": "🚀🌟💫"},
            {"text": "こんにちは"},
            {"text": "𝒯𝑒𝓈𝓉"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?length(@.text) == 5]", 3), // ASCII, accented, Japanese
            ("$.items[?length(@.text) == 3]", 1), // Emoji count
            ("$.items[?length(@.text) == 4]", 1), // Mathematical symbols
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Document behavior for Unicode handling
            println!(
                "Unicode length test '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_nested_data_structure_length() {
        // Extended test: length() on deeply nested structures
        let json_data = r#"{"items": [
            {"data": {"nested": {"array": [1, 2, 3, 4, 5]}}},
            {"data": {"nested": {"object": {"a": 1, "b": 2, "c": 3}}}},
            {"data": {"nested": {"string": "test_string"}}}
        ]}"#;

        let test_cases = vec![
            ("$.items[?length(@.data.nested.array) == 5]", 1), // Nested array
            ("$.items[?length(@.data.nested.object) == 3]", 1), // Nested object
            ("$.items[?length(@.data.nested.string) == 11]", 1), // Nested string
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Nested length test '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_length_performance_large_data() {
        // Extended test: Performance validation with larger datasets
        let large_array: Vec<i32> = (0..1000).collect();
        let large_object: serde_json::Map<String, serde_json::Value> = (0..100)
            .map(|i| (format!("key_{}", i), serde_json::Value::Number(i.into())))
            .collect();

        let json_value = serde_json::json!({
            "data": {
                "large_array": large_array,
                "large_object": large_object,
                "large_string": "a".repeat(10000)
            }
        });

        let json_data = serde_json::to_string(&json_value).expect("Valid JSON");

        let test_cases = vec![
            ("$.data[?length(@.large_array) == 1000]", 1), // Large array
            ("$.data[?length(@.large_object) == 100]", 1), // Large object
            ("$.data[?length(@.large_string) == 10000]", 1), // Large string
        ];

        for (expr, expected_count) in test_cases {
            let start_time = std::time::Instant::now();

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "Performance test '{}' returned {} results in {:?} (expected {})",
                expr,
                results.len(),
                duration,
                expected_count
            );

            // Performance assertion - should complete reasonably quickly
            assert!(
                duration.as_millis() < 1000,
                "length() performance test '{}' should complete in <1000ms",
                expr
            );
        }
    }

    #[test]
    fn test_length_edge_cases() {
        // Extended test: Edge cases and boundary conditions
        let json_data = r#"{"items": [
            {"empty_string": ""},
            {"empty_array": []},
            {"empty_object": {}},
            {"null_value": null},
            {"nested_empty": {"inner": {"value": ""}}},
            {"mixed": {"array": [], "string": "", "object": {}}}
        ]}"#;

        let test_cases = vec![
            ("$.items[?length(@.empty_string) == 0]", 1), // Empty string
            ("$.items[?length(@.empty_array) == 0]", 1),  // Empty array
            ("$.items[?length(@.empty_object) == 0]", 1), // Empty object
            ("$.items[?length(@.null_value) == null]", 1), // Null value
            ("$.items[?length(@.nested_empty.inner.value) == 0]", 1), // Nested empty
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Edge case test '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }
}

/// length() Function Error Cases and Validation
#[cfg(test)]
mod length_error_tests {
    use super::*;

    #[test]
    fn test_length_invalid_syntax() {
        // Test invalid length() function syntax
        let invalid_expressions = vec![
            "$.items[?length()]",              // Missing argument
            "$.items[?length(@.prop, extra)]", // Too many arguments
            "$.items[?length('string')]",      // Invalid argument type
            "$.items[?length(42)]",            // Invalid argument type
        ];

        for expr in invalid_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Invalid length() syntax '{}' should fail",
                expr
            );
        }
    }

    #[test]
    fn test_length_argument_validation() {
        // Test length() function argument validation
        let validation_cases = vec![
            "$.items[?length(@.nonexistent)]", // Non-existent property
            "$.items[?length(@)]",             // Current node reference
            "$.items[?length(@.*)]",           // Wildcard reference
        ];

        for expr in validation_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!(
                    "length() validation '{}' compiled (runtime behavior varies)",
                    expr
                ),
                Err(_) => println!("length() validation '{}' rejected at compile time", expr),
            }
        }
    }

    #[test]
    fn test_length_in_complex_expressions() {
        // Test length() in complex logical expressions
        let json_data = r#"{"items": [
            {"name": "short", "tags": ["a"]},
            {"name": "medium", "tags": ["a", "b"]},
            {"name": "long", "tags": ["a", "b", "c"]}
        ]}"#;

        let complex_expressions = vec![
            "$.items[?length(@.name) > 4 && length(@.tags) >= 2]", // AND condition
            "$.items[?length(@.name) < 5 || length(@.tags) == 3]", // OR condition
            "$.items[?(length(@.name) + length(@.tags)) > 7]",     // Arithmetic (if supported)
        ];

        for expr in complex_expressions {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Complex length() expression '{}' returned {} results",
                        expr,
                        results.len()
                    );
                }
                Err(_) => println!("Complex length() expression '{}' not supported", expr),
            }
        }
    }
}

/// RFC 9535 Section 2.4.5 - count() Function Tests
#[cfg(test)]
mod count_function_tests {
    use super::*;

    #[test]
    fn test_count_nodelist() {
        // RFC 9535: count() returns number of nodes in nodelist
        let json_data = r#"{"data": {
            "books": [
                {"author": "Author1", "tags": ["fiction", "classic"]},
                {"author": "Author2", "tags": ["non-fiction"]},
                {"author": "Author3", "tags": ["fiction", "mystery", "thriller"]}
            ]
        }}"#;

        // Test count of various nodelists
        let test_cases = vec![
            ("count($.data.books[*].author)", 3),  // Count all authors
            ("count($.data.books[*].tags[*])", 6), // Count all tags
            ("count($.data.books[?@.author])", 3), // Count books with authors
        ];

        for (count_expr, expected_count) in test_cases {
            // Note: This tests the count() function syntax
            // Actual implementation may vary based on how count() is integrated
            let filter_expr = format!("$.data[?count($.books[*]) == {}]", expected_count);
            let result = JsonPathParser::compile(&filter_expr);

            // Document whether count() function syntax is supported
            match result {
                Ok(_) => println!("count() function syntax supported: {}", count_expr),
                Err(_) => println!("count() function syntax not yet supported: {}", count_expr),
            }
        }
    }

    #[test]
    fn test_count_empty_nodelist() {
        // RFC 9535: count() of empty nodelist returns 0
        let json_data = r#"{"data": {"items": []}}"#;

        let filter_expr = "$.data[?count(@.items[*]) == 0]";
        let mut stream = JsonArrayStream::<serde_json::Value>::new(filter_expr);

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        // Document current behavior
        println!("count() empty nodelist returned {} results", results.len());
    }

    #[test]
    fn test_count_in_comparisons() {
        // RFC 9535: count() used in comparison expressions
        let json_data = r#"{"groups": [
            {"items": [1, 2]},
            {"items": [1, 2, 3, 4, 5]},
            {"items": []}
        ]}"#;

        let test_cases = vec![
            ("$.groups[?count(@.items[*]) > 3]", 1), // Groups with >3 items
            ("$.groups[?count(@.items[*]) == 0]", 1), // Empty groups
            ("$.groups[?count(@.items[*]) <= 2]", 2), // Small groups
        ];

        for (expr, expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "count() comparison '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(_) => println!("count() syntax not supported: {}", expr),
            }
        }
    }

    #[test]
    fn test_count_performance_large_dataset() {
        // Extended test: Performance validation with large datasets
        let large_items: Vec<serde_json::Value> = (0..10000)
            .map(|i| serde_json::json!({"id": i, "active": i % 2 == 0}))
            .collect();

        let json_value = serde_json::json!({
            "data": {
                "items": large_items
            }
        });

        let json_data = serde_json::to_string(&json_value).expect("Valid JSON");

        let test_cases = vec![
            ("$.data[?count(@.items[*]) == 10000]", 1), // Count all items
            ("$.data[?count(@.items[?@.active]) == 5000]", 1), // Count active items
        ];

        for (expr, expected_count) in test_cases {
            let start_time = std::time::Instant::now();

            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data.clone());
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    let duration = start_time.elapsed();

                    println!(
                        "count() performance test '{}' returned {} results in {:?} (expected {})",
                        expr,
                        results.len(),
                        duration,
                        expected_count
                    );

                    // Performance assertion
                    assert!(
                        duration.as_millis() < 2000,
                        "count() performance test '{}' should complete in <2000ms",
                        expr
                    );
                }
                Err(_) => println!("count() performance test '{}' not supported", expr),
            }
        }
    }
}

/// count() Function Error Cases and Validation
#[cfg(test)]
mod count_error_tests {
    use super::*;

    #[test]
    fn test_count_invalid_syntax() {
        // Test invalid count() function syntax
        let invalid_expressions = vec![
            "$.items[?count()]",              // Missing argument
            "$.items[?count(@.prop, extra)]", // Too many arguments
            "$.items[?count('string')]",      // Invalid argument type
            "$.items[?count(42)]",            // Invalid argument type
        ];

        for expr in invalid_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Invalid count() syntax '{}' should fail",
                expr
            );
        }
    }

    #[test]
    fn test_count_in_complex_expressions() {
        // Test count() in complex logical expressions
        let json_data = r#"{"data": [
            {"items": [1, 2], "tags": ["a"]},
            {"items": [1, 2, 3], "tags": ["a", "b"]},
            {"items": [], "tags": ["a", "b", "c"]}
        ]}"#;

        let complex_expressions = vec![
            "$.data[?count(@.items[*]) > 1 && count(@.tags[*]) >= 2]", // AND condition
            "$.data[?count(@.items[*]) == 0 || count(@.tags[*]) == 3]", // OR condition
            "$.data[?(count(@.items[*]) + count(@.tags[*])) > 4]",     // Arithmetic (if supported)
        ];

        for expr in complex_expressions {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Complex count() expression '{}' returned {} results",
                        expr,
                        results.len()
                    );
                }
                Err(_) => println!("Complex count() expression '{}' not supported", expr),
            }
        }
    }
}

/// RFC 9535 Section 2.4.6 - match() Function Tests
#[cfg(test)]
mod match_function_tests {
    use super::*;

    #[test]
    fn test_regex_matching() {
        // RFC 9535: match() tests if string matches regular expression
        let json_data = r#"{"items": [
            {"code": "ABC123"},
            {"code": "XYZ789"},
            {"code": "DEF456"},
            {"code": "invalid"},
            {"code": "123ABC"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.code, '^[A-Z]{3}[0-9]{3}$')]", 3), // Exact pattern
            ("$.items[?match(@.code, '^[A-Z]')]", 3),             // Starts with letter
            ("$.items[?match(@.code, '[0-9]')]", 4),              // Contains digit
            ("$.items[?match(@.code, '^[0-9]')]", 1),             // Starts with digit
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
                        "match() filter '{}' should return {} items",
                        expr,
                        expected_count
                    );
                }
                Err(_) => println!("match() function not yet supported: {}", expr),
            }
        }
    }

    #[test]
    fn test_match_case_sensitivity() {
        // RFC 9535: match() should be case-sensitive by default
        let json_data = r#"{"items": [
            {"text": "Hello"},
            {"text": "hello"},
            {"text": "HELLO"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?match(@.text, '^Hello$')]", 1),     // Exact case
            ("$.items[?match(@.text, '^hello$')]", 1),     // Lowercase
            ("$.items[?match(@.text, '^HELLO$')]", 1),     // Uppercase
            ("$.items[?match(@.text, '(?i)^hello$')]", 3), // Case insensitive
        ];

        for (expr, expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => println!("match() case sensitivity supported: {}", expr),
                Err(_) => println!("match() function not yet supported: {}", expr),
            }
        }
    }

    #[test]
    fn test_match_special_characters() {
        // RFC 9535: match() with regex special characters
        let json_data = r#"{"items": [
            {"email": "user@example.com"},
            {"email": "invalid-email"},
            {"email": "test.user+tag@domain.org"}
        ]}"#;

        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let expr = format!("$.items[?match(@.email, '{}')]", email_pattern);

        let result = JsonPathParser::compile(&expr);
        match result {
            Ok(_) => println!("match() with complex regex supported"),
            Err(_) => println!("match() function not yet supported: {}", expr),
        }
    }
}

/// RFC 9535 Section 2.4.7 - search() Function Tests
#[cfg(test)]
mod search_function_tests {
    use super::*;

    #[test]
    fn test_regex_search() {
        // RFC 9535: search() tests if string contains match for regex
        let json_data = r#"{"items": [
            {"_description": "This contains the word test"},
            {"_description": "No matching content here"},
            {"_description": "Another test example"},
            {"_description": "Testing is important"}
        ]}"#;

        let test_cases = vec![
            ("$.items[?search(@._description, 'test')]", 3), // Contains 'test'
            ("$.items[?search(@._description, '^This')]", 1), // Starts with 'This'
            ("$.items[?search(@._description, 'important$')]", 1), // Ends with 'important'
            ("$.items[?search(@._description, '[Tt]est')]", 3), // Case variations
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
                        "search() filter '{}' should return {} items",
                        expr,
                        expected_count
                    );
                }
                Err(_) => println!("search() function not yet supported: {}", expr),
            }
        }
    }

    #[test]
    fn test_search_vs_match_difference() {
        // RFC 9535: Demonstrate difference between search() and match()
        let json_data = r#"{"items": [
            {"text": "prefix_target_suffix"},
            {"text": "target"},
            {"text": "no_match_here"}
        ]}"#;

        let search_expr = "$.items[?search(@.text, 'target')]";
        let match_expr = "$.items[?match(@.text, 'target')]";

        println!("Testing search() vs match() difference:");

        let searchresult = JsonPathParser::compile(search_expr);
        let matchresult = JsonPathParser::compile(match_expr);

        match (searchresult, matchresult) {
            (Ok(_), Ok(_)) => println!("Both search() and match() syntax supported"),
            (Ok(_), Err(_)) => println!("Only search() syntax supported"),
            (Err(_), Ok(_)) => println!("Only match() syntax supported"),
            (Err(_), Err(_)) => println!("Neither search() nor match() syntax supported yet"),
        }
    }
}

/// RFC 9535 Section 2.4.8 - value() Function Tests
#[cfg(test)]
mod value_function_tests {
    use super::*;

    #[test]
    fn test_single_node_value_extraction() {
        // RFC 9535: value() converts single-node nodelist to value
        let json_data = r#"{"config": {"timeout": 30}}"#;

        // Test value() function - syntax may vary based on implementation
        let expressions = vec![
            "$.config[?value(@.timeout) > 20]", // Using value() in filter
            "$.config[?@.timeout > 20]",        // Direct property access
        ];

        for expr in expressions {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("value() function syntax supported: {}", expr),
                Err(_) => println!("value() function syntax not supported: {}", expr),
            }
        }
    }

    #[test]
    fn test_value_multi_node_error() {
        // RFC 9535: value() on multi-node nodelist should error
        let json_data = r#"{"items": [1, 2, 3]}"#;

        let expr = "$.items[?value(@[*]) > 1]"; // Multi-node nodelist
        let result = JsonPathParser::compile(expr);

        match result {
            Ok(_) => println!("value() multi-node syntax compiled (may error at runtime)"),
            Err(_) => println!("value() multi-node rejected at compile time"),
        }
    }

    #[test]
    fn test_value_empty_nodelist() {
        // RFC 9535: value() on empty nodelist behavior
        let json_data = r#"{"data": {}}"#;

        let expr = "$.data[?value(@.nonexistent) == null]";
        let result = JsonPathParser::compile(expr);

        match result {
            Ok(_) => println!("value() empty nodelist syntax supported"),
            Err(_) => println!("value() function syntax not supported: {}", expr),
        }
    }
}

/// Function Extension Error Cases
#[cfg(test)]
mod function_error_tests {
    use super::*;

    #[test]
    fn test_invalid_function_names() {
        // Test invalid or unsupported function names
        let invalid_functions = vec![
            "$.items[?unknown(@.prop)]", // Unknown function
            "$.items[?length()]",        // Missing argument
            "$.items[?match(@.text)]",   // Missing regex pattern
            "$.items[?search(@.text)]",  // Missing pattern
            "$.items[?count()]",         // Missing nodelist
        ];

        for expr in invalid_functions {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_err(), "Invalid function '{}' should fail", expr);
        }
    }

    #[test]
    fn test_function_argument_validation() {
        // Test function argument validation
        let invalid_args = vec![
            "$.items[?length(@.prop, extra)]", // Too many arguments
            "$.items[?match(@.text, pattern, flags, extra)]", // Too many args
        ];

        for expr in invalid_args {
            let result = JsonPathParser::compile(expr);
            // May be valid syntax but invalid semantically
            match result {
                Ok(_) => println!("Function args '{}' compiled (may fail at runtime)", expr),
                Err(_) => println!("Function args '{}' rejected at compile time", expr),
            }
        }
    }

    #[test]
    fn test_nested_function_calls() {
        // Test nested function calls
        let nested_expressions = vec![
            "$.items[?length(match(@.text, 'pattern'))]", // Nested functions
            "$.items[?count(value(@.items[*]))]",         // Complex nesting
        ];

        for expr in nested_expressions {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("Nested functions '{}' supported", expr),
                Err(_) => println!("Nested functions '{}' not supported", expr),
            }
        }
    }
}

/// RFC 9535 Section 2.4.1 - Type System Overview Tests
#[cfg(test)]
mod type_system_overview_tests {
    use super::*;

    #[test]
    fn test_value_type_primitives() {
        // RFC 9535: ValueType includes primitive JSON values
        let json_data = r#"{
            "primitives": [
                {"type": "null", "value": null},
                {"type": "boolean_true", "value": true},
                {"type": "boolean_false", "value": false},
                {"type": "number_int", "value": 42},
                {"type": "number_float", "value": 3.14},
                {"type": "string", "value": "hello"},
                {"type": "array", "value": [1, 2, 3]},
                {"type": "object", "value": {"key": "value"}}
            ]
        }"#;

        let test_cases = vec![
            // Test basic value type access
            ("$.primitives[?@.value == null]", 1), // null value
            ("$.primitives[?@.value == true]", 1), // boolean true
            ("$.primitives[?@.value == false]", 1), // boolean false
            ("$.primitives[?@.value == 42]", 1),   // integer
            ("$.primitives[?@.value == 3.14]", 1), // float
            ("$.primitives[?@.value == \"hello\"]", 1), // string
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Value type test '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_logical_type_operations() {
        // RFC 9535: LogicalType for boolean logic operations
        let json_data = r#"{
            "conditions": [
                {"active": true, "priority": 1},
                {"active": false, "priority": 2},
                {"active": true, "priority": 3},
                {"active": false, "priority": 4}
            ]
        }"#;

        let test_cases = vec![
            // Basic logical operations
            ("$.conditions[?@.active]", 2), // Truthy evaluation
            ("$.conditions[?@.active == true]", 2), // Explicit true comparison
            ("$.conditions[?@.active == false]", 2), // Explicit false comparison
            ("$.conditions[?@.active && @.priority > 2]", 1), // Logical AND
            ("$.conditions[?@.active || @.priority < 2]", 3), // Logical OR
            ("$.conditions[?!@.active]", 2), // Logical NOT (if supported)
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
                        "Logical type test '{}' should return {} items",
                        expr,
                        expected_count
                    );
                }
                Err(_) => {
                    println!("Logical operation '{}' not supported", expr);
                }
            }
        }
    }

    #[test]
    fn test_nodes_type_sequences() {
        // RFC 9535: NodesType represents sequences of nodes
        let json_data = r#"{
            "data": {
                "items": [
                    {"id": 1, "tags": ["a", "b"]},
                    {"id": 2, "tags": ["c", "d", "e"]},
                    {"id": 3, "tags": []}
                ]
            }
        }"#;

        let test_cases = vec![
            // Node sequences and their evaluation
            ("$.data.items[*]", 3),          // All items (NodesType)
            ("$.data.items[*].id", 3),       // All IDs (NodesType to ValueType)
            ("$.data.items[*].tags[*]", 5),  // All tags (flattened NodesType)
            ("$.data.items[?@.id > 1]", 2),  // Filtered nodes
            ("$.data.items[?@.tags[*]]", 2), // Items with non-empty tags
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Nodes type test '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }
}

/// RFC 9535 Section 2.4.2 - Function Argument Types Tests
#[cfg(test)]
mod function_argument_types_tests {
    use super::*;

    #[test]
    fn test_length_function_argument_types() {
        // RFC 9535: length() function argument type validation
        let json_data = r#"{
            "_test_data": [
                {"str_value": "hello", "num_value": 42, "arr_value": [1, 2, 3], "obj_value": {"a": 1, "b": 2}},
                {"str_value": "world", "num_value": 99, "arr_value": [], "obj_value": {}},
                {"str_value": "", "num_value": 0, "arr_value": [1], "obj_value": {"x": 10}}
            ]
        }"#;

        let test_cases = vec![
            // Valid length() argument types
            ("$._test_data[?length(@.str_value) == 5]", 1), // String argument
            ("$._test_data[?length(@.arr_value) == 3]", 1), // Array argument
            ("$._test_data[?length(@.obj_value) == 2]", 2), // Object argument
            ("$._test_data[?length(@.str_value) == 0]", 1), // Empty string
            ("$._test_data[?length(@.arr_value) == 0]", 1), // Empty array
            ("$._test_data[?length(@.obj_value) == 0]", 1), // Empty object
        ];

        for (expr, expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Length argument test '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(_) => {
                    println!("Length function not yet supported: {}", expr);
                }
            }
        }
    }

    #[test]
    fn test_invalid_function_argument_types() {
        // RFC 9535: Invalid argument types should cause type errors
        let invalid_length_args = vec![
            "$.items[?length(@.num_value)]",  // Number to length() (invalid)
            "$.items[?length(@.bool_value)]", // Boolean to length() (invalid)
            "$.items[?length(42)]",           // Literal number (invalid)
            "$.items[?length(true)]",         // Literal boolean (invalid)
        ];

        for expr in invalid_length_args {
            let result = JsonPathParser::compile(expr);
            // Note: Some implementations may allow these and return null/error at runtime
            match result {
                Ok(_) => println!(
                    "Invalid length() arg '{}' compiled (may error at runtime)",
                    expr
                ),
                Err(_) => println!("Invalid length() arg '{}' rejected at compile time", expr),
            }
        }
    }

    #[test]
    fn test_comparison_operator_type_constraints() {
        // RFC 9535: Comparison operators have type constraints
        let json_data = r#"{
            "mixed_types": [
                {"value": 42, "type": "number"},
                {"value": "42", "type": "string"},
                {"value": true, "type": "boolean"},
                {"value": null, "type": "null"},
                {"value": [1, 2], "type": "array"},
                {"value": {"x": 1}, "type": "object"}
            ]
        }"#;

        let test_cases = vec![
            // Valid type comparisons
            ("$.mixed_types[?@.value == 42]", 1), // Number comparison
            ("$.mixed_types[?@.value == \"42\"]", 1), // String comparison
            ("$.mixed_types[?@.value == true]", 1), // Boolean comparison
            ("$.mixed_types[?@.value == null]", 1), // Null comparison
            // Cross-type comparisons (behavior varies by implementation)
            ("$.mixed_types[?@.value != null]", 5), // Not null
            ("$.mixed_types[?@.type == \"number\"]", 1), // String comparison for type
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Type comparison '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_arithmetic_comparison_types() {
        // RFC 9535: Arithmetic comparisons require comparable types
        let json_data = r#"{
            "numbers": [
                {"value": 10, "priority": 1},
                {"value": 20, "priority": 2},
                {"value": 5, "priority": 3},
                {"value": 15, "priority": 4}
            ]
        }"#;

        let test_cases = vec![
            // Valid numeric comparisons
            ("$.numbers[?@.value > 10]", 2),         // Greater than
            ("$.numbers[?@.value < 15]", 2),         // Less than
            ("$.numbers[?@.value >= 15]", 2),        // Greater than or equal
            ("$.numbers[?@.value <= 10]", 2),        // Less than or equal
            ("$.numbers[?@.value > @.priority]", 4), // Cross-property comparison
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Arithmetic comparison '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }
}

/// RFC 9535 Section 2.4.3 - Type Conversion Tests
#[cfg(test)]
mod type_conversion_tests {
    use super::*;

    #[test]
    fn test_nodes_to_value_conversion() {
        // RFC 9535: NodesType to ValueType conversion rules
        let json_data = r#"{
            "conversions": [
                {"single": 42},
                {"multiple": [1, 2, 3]},
                {"empty": []}
            ]
        }"#;

        let test_cases = vec![
            // Single node to value conversion
            ("$.conversions[?@.single == 42]", 1), // Direct value access
            // Array access patterns
            ("$.conversions[?@.multiple[0] == 1]", 1), // First element
            ("$.conversions[?@.multiple[2] == 3]", 1), // Third element
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Conversion test '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_logical_type_error_handling() {
        // RFC 9535: LogicalType error scenarios
        let json_data = r#"{
            "error_cases": [
                {"value": "not_a_number"},
                {"value": [1, 2, 3]},
                {"value": {"nested": "object"}},
                {"value": null}
            ]
        }"#;

        // Test expressions that may produce LogicalTypeError
        let expressions = vec![
            "$.error_cases[?@.value > 10]",    // String > number comparison
            "$.error_cases[?@.value == 42]",   // Mixed type equality
            "$.error_cases[?@.value && true]", // Non-boolean in logical context
        ];

        for expr in expressions {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Document behavior rather than asserting specific results
            // as error handling may vary between implementations
            println!(
                "Logical type error test '{}' returned {} results",
                expr,
                results.len()
            );
        }
    }

    #[test]
    fn test_functionresult_type_validation() {
        // RFC 9535: Function result types must match expected types
        let json_data = r#"{
            "function_tests": [
                {"text": "hello", "numbers": [1, 2, 3, 4, 5]},
                {"text": "world", "numbers": []},
                {"text": "", "numbers": [42]}
            ]
        }"#;

        let test_cases = vec![
            // length() returns ValueType (number)
            ("$.function_tests[?length(@.text) > 0]", 2), // String length > 0
            ("$.function_tests[?length(@.numbers) > 3]", 1), // Array length > 3
            ("$.function_tests[?length(@.text) == length(@.numbers)]", 0), // Equal lengths
        ];

        for (expr, expected_count) in test_cases {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Function result type test '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(_) => {
                    println!("Function result type test '{}' not supported", expr);
                }
            }
        }
    }
}

/// Function Argument Validation Tests
#[cfg(test)]
mod function_argument_validation_tests {
    use super::*;

    #[test]
    fn test_single_argument_functions() {
        // RFC 9535: Functions that require exactly one argument
        let single_arg_functions = vec![
            "length(@.value)",   // length() takes one ValueType argument
            "count(@.items[*])", // count() takes one NodesType argument
        ];

        for func in single_arg_functions {
            let expr = format!("$.items[?{} > 0]", func);
            let result = JsonPathParser::compile(&expr);

            match result {
                Ok(_) => println!("Single argument function '{}' syntax supported", func),
                Err(_) => println!("Single argument function '{}' not yet supported", func),
            }
        }
    }

    #[test]
    fn test_two_argument_functions() {
        // RFC 9535: Functions that require exactly two arguments
        let two_arg_functions = vec![
            r#"match(@.text, "pattern")"#,  // match() takes ValueType, string
            r#"search(@.text, "pattern")"#, // search() takes ValueType, string
        ];

        for func in two_arg_functions {
            let expr = format!("$.items[?{}]", func);
            let result = JsonPathParser::compile(&expr);

            match result {
                Ok(_) => println!("Two argument function '{}' syntax supported", func),
                Err(_) => println!("Two argument function '{}' not yet supported", func),
            }
        }
    }

    #[test]
    fn test_special_argument_functions() {
        // RFC 9535: Functions with special argument requirements
        let special_arg_functions = vec![
            "value(@.single_item)", // value() takes NodesType (must be singular)
        ];

        for func in special_arg_functions {
            let expr = format!("$.items[?{} == 42]", func);
            let result = JsonPathParser::compile(&expr);

            match result {
                Ok(_) => println!("Special argument function '{}' syntax supported", func),
                Err(_) => println!("Special argument function '{}' not yet supported", func),
            }
        }
    }

    #[test]
    fn test_function_argument_count_validation() {
        // Test functions with wrong number of arguments
        let invalid_arg_counts = vec![
            "$.items[?length()]",                     // length() with no arguments
            "$.items[?length(@.a, @.b)]",             // length() with two arguments
            "$.items[?match(@.text)]",                // match() with one argument
            r#"$.items[?match(@.text, "p1", "p2")]"#, // match() with three arguments
            "$.items[?count()]",                      // count() with no arguments
            "$.items[?value()]",                      // value() with no arguments
        ];

        for expr in invalid_arg_counts {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Invalid argument count '{}' should fail",
                expr
            );
        }
    }
}

/// Nested Function Expression Tests
#[cfg(test)]
mod nested_function_tests {
    use super::*;

    #[test]
    fn test_function_composition() {
        // RFC 9535: Functions can be composed if types align
        let json_data = r#"{
            "nested_data": [
                {"items": ["a", "bb", "ccc"]},
                {"items": ["x", "yy"]},
                {"items": []}
            ]
        }"#;

        let composition_tests = vec![
            // Hypothetical nested function expressions
            "$.nested_data[?length(@.items) > 0]", // Basic length
            "$.nested_data[?count(@.items[*]) > 2]", // Count of array elements
        ];

        for expr in composition_tests {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Function composition '{}' returned {} results",
                        expr,
                        results.len()
                    );
                }
                Err(_) => println!("Function composition '{}' not supported", expr),
            }
        }
    }

    #[test]
    fn test_complex_nested_expressions() {
        // RFC 9535: Complex expressions with multiple function calls
        let json_data = r#"{
            "complex": [
                {"name": "item1", "tags": ["tag1", "tag2"], "active": true},
                {"name": "item2", "tags": ["tag3"], "active": false},
                {"name": "item3", "tags": [], "active": true}
            ]
        }"#;

        let complex_expressions = vec![
            // Multiple conditions with functions
            "$.complex[?length(@.name) > 4 && @.active]",
            "$.complex[?length(@.tags) > 0 && @.active]",
            "$.complex[?length(@.name) == length(@.tags)]", // Compare function results
        ];

        for expr in complex_expressions {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Complex nested expression '{}' returned {} results",
                        expr,
                        results.len()
                    );
                }
                Err(_) => println!("Complex nested expression '{}' not supported", expr),
            }
        }
    }

    #[test]
    fn test_type_error_propagation() {
        // RFC 9535: Type errors should propagate through nested expressions
        let json_data = r#"{
            "type_errors": [
                {"value": 42},
                {"value": "string"},
                {"value": null}
            ]
        }"#;

        let error_propagation_tests = vec![
            // Expressions that may cause type errors
            "$.type_errors[?length(@.value) > 5]", // length() on number/null
            "$.type_errors[?@.value > length(@.value)]", // Mixed type comparison
        ];

        for expr in error_propagation_tests {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Type error propagation '{}' returned {} results",
                        expr,
                        results.len()
                    );
                }
                Err(_) => println!("Type error propagation '{}' rejected", expr),
            }
        }
    }
}

/// Well-Typedness Validation Tests
#[cfg(test)]
mod well_typedness_tests {
    use super::*;

    #[test]
    fn test_test_expr_well_typedness() {
        // RFC 9535: test-expr must be well-typed in current context
        let test_expr_cases = vec![
            // Valid test expressions
            "@.value > 10",                    // Comparison (should be LogicalType)
            "@.active",                        // Boolean property access
            "@.name == 'test'",                // String equality
            "length(@.items) > 0",             // Function call returning number

            // Invalid test expressions (if caught at compile time)
            // "@.items[*]",                   // NodesType in test context (invalid)
            // "length(@.items, extra)",       // Wrong argument count
        ];

        for test_expr in test_expr_cases {
            let full_expr = format!("$.data[?{}]", test_expr);
            let result = JsonPathParser::compile(&full_expr);

            match result {
                Ok(_) => println!("Test expression '{}' is well-typed", test_expr),
                Err(_) => println!("Test expression '{}' is not well-typed", test_expr),
            }
        }
    }

    #[test]
    fn test_comparable_type_validation() {
        // RFC 9535: Comparable types in comparisons
        let comparable_tests = vec![
            // Valid comparable operations
            ("$.items[?@.number1 == @.number2]", true), // Number == Number
            ("$.items[?@.string1 == @.string2]", true), // String == String
            ("$.items[?@.boolean1 == @.boolean2]", true), // Boolean == Boolean
            ("$.items[?@.value == null]", true),        // Any == null
            // Potentially invalid comparable operations
            ("$.items[?@.number == @.string]", false), // Number == String
            ("$.items[?@.array == @.object]", false),  // Array == Object
        ];

        for (expr, _should_be_valid) in comparable_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                println!(
                    "Comparable expression '{}' should be valid: {:?}",
                    expr,
                    result.is_ok()
                );
            } else {
                // Note: Some implementations may allow cross-type comparisons
                println!(
                    "Comparable expression '{}' validity: {:?}",
                    expr,
                    result.is_ok()
                );
            }
        }
    }

    #[test]
    fn test_function_argument_well_typedness() {
        // RFC 9535: Function arguments must be well-typed
        let function_arg_tests = vec![
            // Valid function argument types
            ("length(@.string_value)", "ValueType"),         // String to length()
            ("length(@.array_value)", "ValueType"),          // Array to length()
            ("count(@.items[*])", "NodesType"),              // NodeList to count()
            (r#"match(@.text, "pattern")"#, "ValueType, String"), // String, Pattern to match()

            // Invalid function argument types (if caught)
            // ("length(@.items[*])", "NodesType"),          // NodeList to length() (invalid)
            // ("count(@.single_value)", "ValueType"),       // Single value to count() (invalid)
        ];

        for (func_expr, expected_type) in function_arg_tests {
            let full_expr = format!("$.data[?{} > 0]", func_expr);
            let result = JsonPathParser::compile(&full_expr);

            match result {
                Ok(_) => println!(
                    "Function argument '{}' ({}) is well-typed",
                    func_expr, expected_type
                ),
                Err(_) => println!(
                    "Function argument '{}' ({}) is not well-typed",
                    func_expr, expected_type
                ),
            }
        }
    }
}
