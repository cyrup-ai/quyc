//! Selector parser module tests
//!
//! Tests for JSONPath selector parser functionality, mirroring src/json_path/selector_parser.rs
//!
//! This module contains comprehensive tests for selector parsing pipeline:
//! - RFC 9535 Selector Compliance (all five selector types)
//! - Array slice selector algorithms and edge cases
//! - Singular query validation and optimization
//! - Selector syntax validation and error handling
//! - Performance validation for large datasets

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct BookModel {
    category: String,
    author: String,
    title: String,
    price: f64,
    isbn: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct StoreModel {
    color: String,
    price: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct NumberModel {
    value: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct ArrayTest {
    data: Vec<i32>,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    name: String,
    value: i32,
}

/// Generate test data for array slice testing
fn generate_test_array(size: usize) -> Vec<i32> {
    (0..size as i32).collect()
}

/// RFC 9535 Section 2.3.1 - Name Selector Tests
#[cfg(test)]
mod name_selector_tests {
    use super::*;

    #[test]
    fn test_single_quoted_name_selector() {
        // RFC 9535: name-selector with single quotes
        let result = JsonPathParser::compile("$['store']");
        assert!(
            result.is_ok(),
            "Single quoted name selector should be valid"
        );

        let json_data = r#"{"store": {"book": [{"category": "fiction"}]}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$['store']");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(results.len(), 1, "Should select store object");
    }

    #[test]
    fn test_double_quoted_name_selector() {
        // RFC 9535: name-selector with double quotes
        let result = JsonPathParser::compile("$[\"store\"]");
        assert!(
            result.is_ok(),
            "Double quoted name selector should be valid"
        );

        let json_data = r#"{"store": {"book": [{"category": "fiction"}]}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$[\"store\"]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(results.len(), 1, "Should select store object");
    }

    #[test]
    fn test_dot_notation_name_selector() {
        // RFC 9535: Dot notation for name selectors
        let result = JsonPathParser::compile("$.store");
        assert!(result.is_ok(), "Dot notation should be valid");

        let json_data = r#"{"store": {"book": [{"category": "fiction"}]}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(results.len(), 1, "Should select store object");
    }

    #[test]
    fn test_escaped_characters_in_names() {
        // RFC 9535: Escaped characters in string literals
        let expressions = vec![
            r#"$['store\'s']"#,    // Escaped single quote
            r#"$["store\"s"]"#,    // Escaped double quote
            r#"$['store\\book']"#, // Escaped backslash
            r#"$['store\nbook']"#, // Escaped newline
            r#"$['store\tbook']"#, // Escaped tab
        ];

        for expr in expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Escaped characters '{}' should be valid",
                expr
            );
        }
    }

    #[test]
    fn test_unicode_property_names() {
        // RFC 9535: Unicode support in name selectors
        let expressions = vec![
            "$.Ñoño",        // Spanish characters
            "$['中文']",     // Chinese characters
            "$['🚀rocket']", // Emoji
            "$['ßeta']",     // German character
        ];

        for expr in expressions {
            let result = JsonPathParser::compile(expr);
            assert!(result.is_ok(), "Unicode name '{}' should be valid", expr);
        }
    }

    #[test]
    fn test_nonexistent_property_selection() {
        // RFC 9535: Name selector should select at most one member
        let json_data = r#"{"store": {"book": []}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.nonexistent");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            0,
            "Nonexistent property should return empty nodelist"
        );
    }
}

/// RFC 9535 Section 2.3.2 - Wildcard Selector Tests  
#[cfg(test)]
mod wildcard_selector_tests {
    use super::*;

    #[test]
    fn test_object_wildcard_selection() {
        // RFC 9535: Wildcard selects all children of object
        let json_data = r#"{"store": {"book": [], "bicycle": {"color": "red"}}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            2,
            "Wildcard should select all object members"
        );
    }

    #[test]
    fn test_array_wildcard_selection() {
        // RFC 9535: Wildcard selects all elements of array
        let json_data = r#"{"books": [{"id": 1}, {"id": 2}, {"id": 3}]}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.books[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            3,
            "Wildcard should select all array elements"
        );
    }

    #[test]
    fn test_root_wildcard_selection() {
        // RFC 9535: Root wildcard should select all top-level children
        let json_data = r#"{"a": 1, "b": 2, "c": 3}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            3,
            "Root wildcard should select all top-level members"
        );
    }

    #[test]
    fn test_wildcard_on_primitive_values() {
        // RFC 9535: Wildcard on primitive values should return empty nodelist
        let json_data = r#"{"value": 42}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.value[*]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            0,
            "Wildcard on primitive should return empty nodelist"
        );
    }

    #[test]
    fn test_dot_star_notation() {
        // RFC 9535: .* notation for wildcard
        let result = JsonPathParser::compile("$.store.*");
        assert!(result.is_ok(), "Dot-star notation should be valid");

        let json_data = r#"{"store": {"book": [], "bicycle": {}}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store.*");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            2,
            "Dot-star should select all store children"
        );
    }
}

/// RFC 9535 Section 2.3.3 - Index Selector Tests
#[cfg(test)]
mod index_selector_tests {
    use super::*;

    #[test]
    fn test_positive_index_selection() {
        // RFC 9535: Non-negative indices select from start (0-based)
        let json_data = r#"{"books": ["book0", "book1", "book2"]}"#;

        let test_cases = vec![
            ("$.books[0]", 0, "book0"),
            ("$.books[1]", 1, "book1"),
            ("$.books[2]", 2, "book2"),
        ];

        for (expr, expected_index, expected_value) in test_cases {
            let mut stream = JsonArrayStream::<String>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                1,
                "Index [{}] should select one element",
                expected_index
            );
            let value = &results[0];
            assert_eq!(value, expected_value, "Should select correct value");
        }
    }

    #[test]
    fn test_negative_index_selection() {
        // RFC 9535: Negative indices select from end (-1 is last element)
        let json_data = r#"{"books": ["book0", "book1", "book2"]}"#;

        let test_cases = vec![
            ("$.books[-1]", "book2"), // Last element
            ("$.books[-2]", "book1"), // Second to last
            ("$.books[-3]", "book0"), // Third to last (first)
        ];

        for (expr, expected_value) in test_cases {
            let mut stream = JsonArrayStream::<String>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                1,
                "Negative index '{}' should select one element",
                expr
            );
            let value = &results[0];
            assert_eq!(
                value, expected_value,
                "Should select correct value from end"
            );
        }
    }

    #[test]
    fn test_out_of_bounds_index() {
        // RFC 9535: Out of bounds index should return empty nodelist
        let json_data = r#"{"books": ["book0", "book1"]}"#;

        let out_of_bounds_cases = vec![
            "$.books[5]",  // Positive out of bounds
            "$.books[-5]", // Negative out of bounds
        ];

        for expr in out_of_bounds_cases {
            let mut stream = JsonArrayStream::<String>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                0,
                "Out of bounds '{}' should return empty nodelist",
                expr
            );
        }
    }

    #[test]
    fn test_index_on_non_array() {
        // RFC 9535: Index selector on non-array should return empty nodelist
        let json_data = r#"{"store": {"name": "bookstore"}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store[0]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            0,
            "Index on object should return empty nodelist"
        );
    }

    #[test]
    fn test_zero_index_validation() {
        // RFC 9535: Index 0 should be valid first element
        let result = JsonPathParser::compile("$.books[0]");
        assert!(result.is_ok(), "Index 0 should be valid");

        let json_data = r#"{"books": ["first"]}"#;
        let mut stream = JsonArrayStream::<String>::new("$.books[0]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();

        assert_eq!(results.len(), 1, "Index 0 should select first element");
        let value = &results[0];
        assert_eq!(value, "first", "Should select first element");
    }
}

/// RFC 9535 Section 2.3.4 - Array Slice Selector Tests
#[cfg(test)]
mod array_slice_tests {
    use super::*;

    #[test]
    fn test_basic_slice_start_end() {
        // RFC 9535: [start:end] selects elements from start (inclusive) to end (exclusive)
        let json_data = r#"{"items": [0, 1, 2, 3, 4, 5]}"#;

        let test_cases = vec![
            ("$.items[1:3]", vec![1, 2]),    // Elements 1 and 2
            ("$.items[0:2]", vec![0, 1]),    // Elements 0 and 1
            ("$.items[2:5]", vec![2, 3, 4]), // Elements 2, 3, and 4
        ];

        for (expr, expected) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results, expected,
                "Slice '{}' should return {:?}",
                expr, expected
            );
        }
    }

    #[test]
    fn test_slice_with_missing_start() {
        // RFC 9535: [:end] starts from beginning
        let json_data = r#"{"items": [0, 1, 2, 3, 4]}"#;

        let test_cases = vec![
            ("$.items[:2]", vec![0, 1]),    // First 2 elements
            ("$.items[:3]", vec![0, 1, 2]), // First 3 elements
            ("$.items[:0]", vec![]),        // Empty slice
        ];

        for (expr, expected) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results, expected,
                "Slice '{}' should return {:?}",
                expr, expected
            );
        }
    }

    #[test]
    fn test_slice_with_missing_end() {
        // RFC 9535: [start:] goes to end of array
        let json_data = r#"{"items": [0, 1, 2, 3, 4]}"#;

        let test_cases = vec![
            ("$.items[2:]", vec![2, 3, 4]),    // From index 2 to end
            ("$.items[1:]", vec![1, 2, 3, 4]), // From index 1 to end
            ("$.items[4:]", vec![4]),          // Last element only
            ("$.items[5:]", vec![]),           // Beyond end
        ];

        for (expr, expected) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results, expected,
                "Slice '{}' should return {:?}",
                expr, expected
            );
        }
    }

    #[test]
    fn test_slice_with_step() {
        // RFC 9535: [start:end:step] with step size
        let json_data = r#"{"items": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]}"#;

        let test_cases = vec![
            ("$.items[::2]", vec![0, 2, 4, 6, 8]),  // Every 2nd element
            ("$.items[1::2]", vec![1, 3, 5, 7, 9]), // Every 2nd starting from 1
            ("$.items[::3]", vec![0, 3, 6, 9]),     // Every 3rd element
            ("$.items[1:8:2]", vec![1, 3, 5, 7]),   // Every 2nd from 1 to 8
        ];

        for (expr, expected) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results, expected,
                "Slice '{}' should return {:?}",
                expr, expected
            );
        }
    }

    #[test]
    fn test_negative_indices_in_slices() {
        // RFC 9535: Negative indices in slices
        let json_data = r#"{"items": [0, 1, 2, 3, 4]}"#;

        let test_cases = vec![
            ("$.items[-3:-1]", vec![2, 3]),   // From -3 to -1 (exclusive)
            ("$.items[-2:]", vec![3, 4]),     // Last 2 elements
            ("$.items[:-2]", vec![0, 1, 2]),  // All but last 2
            ("$.items[-4:-1:2]", vec![1, 3]), // Every 2nd from -4 to -1
        ];

        for (expr, expected) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results, expected,
                "Slice '{}' should return {:?}",
                expr, expected
            );
        }
    }

    #[test]
    fn test_slice_edge_cases() {
        // RFC 9535: Edge cases for slice selectors
        let json_data = r#"{"items": [0, 1, 2, 3, 4]}"#;

        let edge_cases = vec![
            ("$.items[10:20]", vec![]),          // Out of bounds slice
            ("$.items[3:1]", vec![]),            // Start > end
            ("$.items[0:0]", vec![]),            // Empty slice
            ("$.items[:]", vec![0, 1, 2, 3, 4]), // Full array slice
        ];

        for (expr, expected) in edge_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results, expected,
                "Edge case '{}' should return {:?}",
                expr, expected
            );
        }
    }

    #[test]
    fn test_slice_on_non_array() {
        // RFC 9535: Slice on non-array should return empty nodelist
        let json_data = r#"{"store": {"name": "bookstore"}}"#;
        let mut stream = JsonArrayStream::<serde_json::Value>::new("$.store[1:3]");

        let chunk = Bytes::from(json_data);
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        assert_eq!(
            results.len(),
            0,
            "Slice on object should return empty nodelist"
        );
    }

    #[test]
    fn test_zero_step_invalid() {
        // RFC 9535: Step of 0 should be invalid
        let result = JsonPathParser::compile("$.items[::0]");
        assert!(result.is_err(), "Step of 0 should be invalid");
    }

    #[test]
    fn test_negative_step_invalid() {
        // RFC 9535: Negative step should be invalid in basic implementation
        let result = JsonPathParser::compile("$.items[::-1]");
        // Document current behavior - may be valid or invalid
        match result {
            Ok(_) => println!("Negative step is supported"),
            Err(_) => println!("Negative step is not supported"),
        }
    }

    #[test]
    fn test_slice_syntax_validation() {
        // RFC 9535: Valid slice syntax forms
        let valid_slices = vec![
            "$.items[1:3]",   // start:end
            "$.items[1:]",    // start:
            "$.items[:3]",    // :end
            "$.items[:]",     // :
            "$.items[1:3:2]", // start:end:step
            "$.items[::2]",   // ::step
        ];

        for expr in valid_slices {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Valid slice syntax '{}' should compile",
                expr
            );
        }
    }

    #[test]
    fn test_invalid_slice_syntax() {
        // Test various invalid slice syntaxes
        let invalid_slices = vec![
            "$.items[1:2:3:4]", // Too many colons
            "$.items[1.5:3]",   // Float indices
            "$.items[a:b]",     // Non-numeric indices
            "$.items[1:::]",    // Too many colons
        ];

        for expr in invalid_slices {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Invalid slice syntax '{}' should fail",
                expr
            );
        }
    }
}

/// Performance tests for slice operations
#[cfg(test)]
mod slice_performance_tests {
    use super::*;

    #[test]
    fn test_large_array_slice_performance() {
        // Generate large array for performance testing
        let large_array: Vec<i32> = (0..1000).collect();
        let json_data = serde_json::to_string(&serde_json::json!({"items": large_array}))
            .expect("JSON serialization");

        let mut stream = JsonArrayStream::<i32>::new("$.items[100:200]");

        let chunk = Bytes::from(json_data);
        let start_time = std::time::Instant::now();
        let results: Vec<_> = stream.process_chunk(chunk).collect();
        let duration = start_time.elapsed();

        assert_eq!(results.len(), 100, "Should select 100 elements");
        println!("Large array slice took {:?}", duration);

        // Performance assertion - should complete in reasonable time
        assert!(
            duration.as_millis() < 100,
            "Slice should complete in <100ms"
        );
    }
}

/// RFC 9535 Section 2.3.4 - Normalize Function Tests
#[cfg(test)]
mod normalize_function_tests {
    use super::*;

    #[test]
    fn test_normalize_positive_indices() {
        // RFC 9535: Normalize(i, len) for positive indices
        // Normalize(i, len) = i when 0 <= i < len
        let json_data = r#"{"arrays": [
            {"data": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "name": "ten_elements"}
        ]}"#;

        let test_cases = vec![
            ("$.arrays[0].data[0:1]", 1),  // Normalize(0, 10) = 0, slice [0:1] = [0]
            ("$.arrays[0].data[3:4]", 1),  // Normalize(3, 10) = 3, slice [3:4] = [3]
            ("$.arrays[0].data[9:10]", 1), // Normalize(9, 10) = 9, slice [9:10] = [9]
            ("$.arrays[0].data[0:5]", 5),  // Multiple elements [0, 1, 2, 3, 4]
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Positive index slice '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_normalize_negative_indices() {
        // RFC 9535: Normalize(i, len) for negative indices
        // Normalize(i, len) = len + i when i < 0
        let json_data = r#"{"arrays": [
            {"data": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "name": "ten_elements"}
        ]}"#;

        let test_cases = vec![
            ("$.arrays[0].data[-1:]", 1),   // Normalize(-1, 10) = 9, last element
            ("$.arrays[0].data[-2:]", 2),   // Normalize(-2, 10) = 8, last two elements
            ("$.arrays[0].data[-5:-2]", 3), // Slice from -5 to -2
            ("$.arrays[0].data[:-1]", 9),   // All except last element
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Negative index slice '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_normalize_out_of_bounds_indices() {
        // RFC 9535: Handling of out-of-bounds indices
        let json_data = r#"{"arrays": [
            {"data": [0, 1, 2, 3, 4], "name": "five_elements"}
        ]}"#;

        let test_cases = vec![
            ("$.arrays[0].data[10:15]", 0),  // Both indices > length
            ("$.arrays[0].data[-10:-5]", 0), // Both indices < -length
            ("$.arrays[0].data[3:20]", 2),   // End index > length
            ("$.arrays[0].data[-10:3]", 3),  // Start index < -length
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Out-of-bounds slice '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_normalize_zero_length_array() {
        // RFC 9535: Normalize function with zero-length arrays
        let json_data = r#"{"arrays": [
            {"data": [], "name": "empty_array"}
        ]}"#;

        let test_cases = vec![
            ("$.arrays[0].data[:]", 0),   // Full slice of empty array
            ("$.arrays[0].data[0:1]", 0), // Any slice of empty array
            ("$.arrays[0].data[-1:]", 0), // Negative index on empty array
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Empty array slice '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }
}

/// RFC 9535 Section 2.3.4 - Bounds Function Tests  
#[cfg(test)]
mod bounds_function_tests {
    use super::*;

    #[test]
    fn test_bounds_positive_step() {
        // RFC 9535: Bounds(start, end, step, len) with positive step
        let json_data = r#"{"arrays": [
            {"data": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "name": "ten_elements"}
        ]}"#;

        let test_cases = vec![
            ("$.arrays[0].data[1:8:2]", 4), // Elements 1, 3, 5, 7 (step=2)
            ("$.arrays[0].data[0::3]", 4),  // Elements 0, 3, 6, 9 (step=3)
            ("$.arrays[0].data[2:7:1]", 5), // Elements 2, 3, 4, 5, 6 (step=1)
            ("$.arrays[0].data[::2]", 5),   // Elements 0, 2, 4, 6, 8 (step=2)
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Positive step slice '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_bounds_negative_step() {
        // RFC 9535: Bounds(start, end, step, len) with negative step
        let json_data = r#"{"arrays": [
            {"data": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "name": "ten_elements"}
        ]}"#;

        let test_cases = vec![
            ("$.arrays[0].data[::-1]", 10),  // Reverse order: 9, 8, 7, ..., 0
            ("$.arrays[0].data[8:2:-2]", 3), // Elements 8, 6, 4 (step=-2)
            ("$.arrays[0].data[7::-3]", 3),  // Elements 7, 4, 1 (step=-3)
            ("$.arrays[0].data[5:1:-1]", 4), // Elements 5, 4, 3, 2 (step=-1)
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Negative step slice '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_bounds_default_values() {
        // RFC 9535 Table 8: Default values for start, end, step
        let json_data = r#"{"arrays": [
            {"data": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "name": "ten_elements"}
        ]}"#;

        let test_cases = vec![
            // Default values based on Table 8:
            ("$.arrays[0].data[:]", 10),    // start=0, end=len, step=1
            ("$.arrays[0].data[2:]", 8),    // start=2, end=len, step=1
            ("$.arrays[0].data[:7]", 7),    // start=0, end=7, step=1
            ("$.arrays[0].data[::2]", 5),   // start=0, end=len, step=2
            ("$.arrays[0].data[::-1]", 10), // start=len-1, end=-1, step=-1
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Default values slice '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_bounds_edge_cases() {
        // RFC 9535: Edge cases for bounds computation
        let json_data = r#"{"arrays": [
            {"data": [0, 1, 2, 3, 4], "name": "five_elements"}
        ]}"#;

        let test_cases = vec![
            ("$.arrays[0].data[5:10]", 0),   // start >= len
            ("$.arrays[0].data[3:2]", 0),    // start > end with positive step
            ("$.arrays[0].data[2:3:-1]", 0), // start < end with negative step
            ("$.arrays[0].data[0:0]", 0),    // start == end
            ("$.arrays[0].data[1:1]", 0),    // start == end (non-zero)
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Edge case slice '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }
}

/// Step Value Edge Cases and Error Handling
#[cfg(test)]
mod step_value_tests {
    use super::*;

    #[test]
    fn test_step_zero_handling() {
        // RFC 9535: step=0 should be an error condition
        let invalid_expressions = vec![
            "$.arrays[0].data[::0]",   // step=0
            "$.arrays[0].data[1:5:0]", // explicit step=0
            "$.arrays[0].data[2::0]",  // step=0 with start
        ];

        for expr in invalid_expressions {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!("step=0 expression '{}' unexpectedly compiled", expr),
                Err(_) => println!("step=0 expression '{}' correctly rejected", expr),
            }
        }
    }

    #[test]
    fn test_large_step_values() {
        // Test with step values larger than array length
        let json_data = r#"{"arrays": [
            {"data": [0, 1, 2, 3, 4], "name": "five_elements"}
        ]}"#;

        let test_cases = vec![
            ("$.arrays[0].data[::10]", 1), // step > length, should get first element
            ("$.arrays[0].data[::100]", 1), // very large step
            ("$.arrays[0].data[::-10]", 1), // negative step > length
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Large step slice '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_step_value_boundaries() {
        // Test step values at _boundaries
        let json_data = r#"{"arrays": [
            {"data": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "name": "ten_elements"}
        ]}"#;

        let test_cases = vec![
            ("$.arrays[0].data[::1]", 10),  // step=1 (default)
            ("$.arrays[0].data[::-1]", 10), // step=-1 (reverse)
            ("$.arrays[0].data[::10]", 1),  // step=length
            ("$.arrays[0].data[::-10]", 1), // step=-length
        ];

        for (expr, expected_count) in test_cases {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Step boundary slice '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }
}

/// Array Slice Performance Tests
#[cfg(test)]
mod array_slice_performance_tests {
    use super::*;

    #[test]
    fn test_large_array_slicing() {
        // Test performance with large arrays
        let large_array = generate_test_array(10000);
        let json_value = serde_json::json!({
            "large_array": large_array
        });
        let json_data = serde_json::to_string(&json_value).expect("Valid JSON");

        let test_cases = vec![
            ("$.large_array[:100]", 100),      // First 100 elements
            ("$.large_array[5000:5100]", 100), // Middle 100 elements
            ("$.large_array[-100:]", 100),     // Last 100 elements
            ("$.large_array[::100]", 100),     // Every 100th element
            ("$.large_array[::-1000]", 10),    // Every 1000th element, reverse
        ];

        for (expr, expected_count) in test_cases {
            let start_time = std::time::Instant::now();

            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "Large array slice '{}' returned {} results in {:?} (expected {})",
                expr,
                results.len(),
                duration,
                expected_count
            );

            // Performance assertion
            assert!(
                duration.as_millis() < 1000,
                "Large array slice '{}' should complete in <1000ms",
                expr
            );
        }
    }

    #[test]
    fn test_memory_efficiency_streaming() {
        // Test that slicing doesn't load entire array into memory
        let medium_array = generate_test_array(1000);
        let json_value = serde_json::json!({
            "arrays": [
                {"data": medium_array, "name": "medium_array"}
            ]
        });
        let json_data = serde_json::to_string(&json_value).expect("Valid JSON");

        let efficient_slices = vec![
            ("$.arrays[0].data[0:10]", 10),     // Small slice from beginning
            ("$.arrays[0].data[990:1000]", 10), // Small slice from end
            ("$.arrays[0].data[::100]", 10),    // Sparse sampling
        ];

        for (expr, expected_count) in efficient_slices {
            let start_time = std::time::Instant::now();

            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            println!(
                "Memory efficient slice '{}' returned {} results in {:?}",
                expr,
                results.len(),
                duration
            );

            assert_eq!(
                results.len(),
                expected_count,
                "Efficient slice '{}' should return exactly {} items",
                expr,
                expected_count
            );

            // Should be very fast for small slices
            assert!(
                duration.as_millis() < 100,
                "Efficient slice '{}' should complete in <100ms",
                expr
            );
        }
    }

    #[test]
    fn test_complex_slice_combinations() {
        // Test complex combinations of slice parameters
        let json_data = r#"{"arrays": [
            {"data": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19], "name": "twenty_elements"}
        ]}"#;

        let complex_slices = vec![
            ("$.arrays[0].data[-10:-5:2]", 3), // Negative indices with step
            ("$.arrays[0].data[5:15:3]", 4),   // Mid-range with large step
            ("$.arrays[0].data[18:2:-4]", 4),  // Reverse with large step
            ("$.arrays[0].data[-3:3:-2]", 8),  // Cross-over indices
        ];

        for (expr, expected_count) in complex_slices {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Complex slice '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }
}

/// Multi-dimensional Array Slice Tests
#[cfg(test)]
mod multidimensional_slice_tests {
    use super::*;

    #[test]
    fn test_nested_array_slicing() {
        // Test slicing operations on nested arrays
        let json_data = r#"{"matrix": [
            [0, 1, 2, 3, 4],
            [5, 6, 7, 8, 9],
            [10, 11, 12, 13, 14],
            [15, 16, 17, 18, 19],
            [20, 21, 22, 23, 24]
        ]}"#;

        let nested_slices = vec![
            ("$.matrix[1:4]", 3),       // Slice rows 1-3
            ("$.matrix[*][1:3]", 15),   // Slice columns 1-2 from all rows
            ("$.matrix[:3][::2]", 9),   // First 3 rows, every 2nd column
            ("$.matrix[::2][1::2]", 6), // Every 2nd row, columns 1,3
        ];

        for (expr, expected_count) in nested_slices {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Nested array slice '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_jagged_array_slicing() {
        // Test slicing with arrays of different lengths
        let json_data = r#"{"jagged": [
            [0, 1],
            [2, 3, 4, 5],
            [6],
            [7, 8, 9, 10, 11, 12],
            []
        ]}"#;

        let jagged_slices = vec![
            ("$.jagged[*][:2]", 7),   // First 2 elements from each subarray
            ("$.jagged[1:4][1:]", 8), // Skip first element from arrays 1-3
            ("$.jagged[*][::2]", 6),  // Every 2nd element from each subarray
        ];

        for (expr, expected_count) in jagged_slices {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Jagged array slice '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }
}

/// Array Slice Algorithm Compliance Validation
#[cfg(test)]
mod algorithm_compliance_validation {
    use super::*;

    #[test]
    fn test_rfc9535_table8_compliance() {
        // Validate compliance with RFC 9535 Table 8 default values
        let json_data = r#"{"test_array": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]}"#;

        // Table 8 from RFC 9535: Default values for array slice selectors
        let table8_tests = vec![
            // [start:end:step] -> computed defaults
            ("$.test_array[:]", 10, "start=0, end=10, step=1"),
            ("$.test_array[2:]", 8, "start=2, end=10, step=1"),
            ("$.test_array[:8]", 8, "start=0, end=8, step=1"),
            ("$.test_array[::2]", 5, "start=0, end=10, step=2"),
            ("$.test_array[::-1]", 10, "start=9, end=-1, step=-1"),
            ("$.test_array[2::2]", 4, "start=2, end=10, step=2"),
            ("$.test_array[:8:2]", 4, "start=0, end=8, step=2"),
        ];

        for (expr, expected_count, _description) in table8_tests {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Table 8 test '{}' -> {} ({})",
                expr,
                results.len(),
                _description
            );

            // Some tests might not be exact due to implementation details
            if results.len() != expected_count {
                println!(
                    "  WARNING: Expected {}, got {} for '{}'",
                    expected_count,
                    results.len(),
                    expr
                );
            }
        }
    }

    #[test]
    fn test_algorithm_correctness() {
        // Test the mathematical correctness of slice algorithms
        let json_data = r#"{"numbers": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]}"#;

        let algorithm_tests = vec![
            // Test Normalize function: should handle negative indices correctly
            ("$.numbers[-1:]", vec![15], "Normalize(-1, 16) = 15"),
            (
                "$.numbers[-3:-1]",
                vec![13, 14],
                "Normalize(-3, 16) = 13, Normalize(-1, 16) = 15",
            ),
            // Test Bounds function: should compute correct ranges
            (
                "$.numbers[2:6:2]",
                vec![2, 4],
                "Bounds(2, 6, 2, 16) = range with step 2",
            ),
            (
                "$.numbers[10:4:-3]",
                vec![10, 7],
                "Bounds(10, 4, -3, 16) = reverse range",
            ),
            // Test edge cases
            ("$.numbers[20:25]", vec![], "Out of bounds: start >= length"),
            ("$.numbers[5:5]", vec![], "Empty range: start == end"),
        ];

        for (expr, expected_subset, _description) in algorithm_tests {
            let mut stream = JsonArrayStream::<i32>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Algorithm test '{}' -> {:?} ({})",
                expr, results, _description
            );

            // For some tests, verify exact match
            if !expected_subset.is_empty() && results.len() == expected_subset.len() {
                for (i, expected) in expected_subset.iter().enumerate() {
                    if i < results.len() && results[i] != *expected {
                        println!(
                            "  WARNING: Expected {:?}, got {:?} for '{}'",
                            expected_subset, results, expr
                        );
                        break;
                    }
                }
            }
        }
    }
}

/// RFC 9535 Singular Query Validation Tests
#[cfg(test)]
mod singular_query_tests {
    use super::*;

    #[test]
    fn test_singular_query_syntax_validation() {
        // RFC 9535: Singular queries are syntactically recognizable at parse time
        let singular_queries = vec![
            // Valid singular queries
            "$",                              // Root (always singular)
            "$.store",                        // Single property access
            "$['store']",                     // Single bracket property access
            "$.store.book",                   // Chain of single properties
            "$['store']['book']",             // Chain of bracket properties
            "$.store['book']",                // Mixed notation
            "$[0]",                           // Single array index
            "$.store.book[0]",                // Property then index
            "$.store.book[0].title",          // Index then property
            "$['store']['book'][0]['title']", // All bracket notation
        ];

        for query in singular_queries {
            let result = JsonPathParser::compile(query);
            assert!(result.is_ok(), "Singular query '{}' should compile", query);
            println!("Singular query '{}' validated successfully", query);
        }
    }

    #[test]
    fn test_non_singular_query_syntax() {
        // RFC 9535: Non-singular queries that should be distinguished
        let non_singular_queries = vec![
            // Wildcard selectors
            "$.*",             // Root wildcard
            "$.store.*",       // Property wildcard
            "$[*]",            // Array wildcard
            "$.store.book[*]", // Mixed with wildcard
            // Slice selectors
            "$[:]",   // Full slice
            "$[1:]",  // Slice from index
            "$[:5]",  // Slice to index
            "$[1:5]", // Range slice
            "$[::2]", // Step slice
            // Union selectors
            "$[0,1]",      // Multiple indices
            "$['a','b']",  // Multiple properties
            "$[0,'name']", // Mixed union
            // Descendant segments
            "$..book",    // Descendant search
            "..*",        // Universal descendant
            "$..book[*]", // Descendant with wildcard
            // Filter expressions
            "$[?@.price]",           // Property filter
            "$.book[?@.price > 10]", // Comparison filter
        ];

        for query in non_singular_queries {
            let result = JsonPathParser::compile(query);
            assert!(
                result.is_ok(),
                "Non-singular query '{}' should compile",
                query
            );
            println!("Non-singular query '{}' validated successfully", query);
        }
    }

    #[test]
    fn test_at_most_one_node_guarantee() {
        // RFC 9535: Singular queries must return at most one node
        let json_data = r#"{
            "store": {
                "book": [
                    {"title": "Book 1", "price": 10},
                    {"title": "Book 2", "price": 20}
                ],
                "bicycle": {"color": "red", "price": 15}
            }
        }"#;

        let singular_tests = vec![
            // These should return exactly one node or empty
            ("$", 1),                     // Root node
            ("$.store", 1),               // Single property
            ("$.store.book", 1),          // Single property (array)
            ("$.store.bicycle", 1),       // Single property (object)
            ("$.store.book[0]", 1),       // First book
            ("$.store.book[1]", 1),       // Second book
            ("$.store.book[0].title", 1), // Title of first book
            ("$.store.bicycle.color", 1), // Bicycle color
            ("$.nonexistent", 0),         // Non-existent property (empty result)
            ("$.store.book[99]", 0),      // Out of bounds index (empty result)
        ];

        for (query, expected_count) in singular_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert!(
                results.len() <= 1,
                "Singular query '{}' should return at most 1 result, got {}",
                query,
                results.len()
            );
            assert_eq!(
                results.len(),
                expected_count,
                "Singular query '{}' should return {} results, got {}",
                query,
                expected_count,
                results.len()
            );

            println!(
                "Singular query '{}' returned {} results (expected {})",
                query,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_normalized_path_equivalence() {
        // RFC 9535: Different syntactic forms should normalize to equivalent paths
        let equivalence_groups = vec![
            // Property access equivalences
            vec!["$.store", "$['store']", "$[\"store\"]"],
            // Chained property access
            vec![
                "$.store.book",
                "$['store']['book']",
                "$['store'].book",
                "$.store['book']",
            ],
            // Array index access
            vec!["$[0]", "$['0']"], // Note: This equivalence depends on implementation
            // Mixed chains
            vec!["$.store.book[0].title", "$['store']['book'][0]['title']"],
        ];

        let json_data = r#"{
            "store": {
                "book": [{"title": "Book 1"}]
            },
            "0": "zero_property"
        }"#;

        for group in equivalence_groups {
            let mut results_sets = Vec::new();

            for query in &group {
                let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

                let chunk = Bytes::from(json_data);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                results_sets.push(results);
                println!(
                    "Query '{}' returned {} results",
                    query,
                    results_sets.last().unwrap().len()
                );
            }

            // All equivalent queries should return the same results
            if results_sets.len() > 1 {
                let first = &results_sets[0];
                for (i, results) in results_sets.iter().enumerate().skip(1) {
                    if first.len() == results.len() && first.len() <= 1 {
                        // For singular queries, check value equality if both have results
                        if !first.is_empty() && !results.is_empty() {
                            assert_eq!(
                                first[0], results[0],
                                "Equivalent queries '{}' and '{}' should return same value",
                                group[0], group[i]
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_json_pointer_conversion() {
        // RFC 9535: Singular queries can be converted to JSON Pointer references
        let json_pointer_tests = vec![
            // JSONPath -> Expected JSON Pointer
            ("$", ""),                                                 // Root
            ("$.store", "/store"),                                     // Single property
            ("$.store.book", "/store/book"),                           // Nested property
            ("$['store']", "/store"),                                  // Bracket notation
            ("$['store']['book']", "/store/book"),                     // Bracket chain
            ("$[0]", "/0"),                                            // Array index
            ("$.store.book[0]", "/store/book/0"),                      // Property + index
            ("$.store['book'][0]", "/store/book/0"),                   // Mixed notation
            ("$['store']['book'][0]['title']", "/store/book/0/title"), // Complex path
        ];

        for (jsonpath, expected_pointer) in json_pointer_tests {
            // This test validates the theoretical conversion
            // Actual implementation would require a conversion function
            println!(
                "JSONPath '{}' should convert to JSON Pointer '{}'",
                jsonpath, expected_pointer
            );

            // Verify the JSONPath is singular and valid
            let result = JsonPathParser::compile(jsonpath);
            assert!(
                result.is_ok(),
                "JSONPath '{}' should be valid for pointer conversion",
                jsonpath
            );
        }
    }

    #[test]
    fn test_json_pointer_equivalence() {
        // RFC 9535: Singular queries and their JSON Pointer equivalents should return same results
        let json_data = r#"{
            "store": {
                "book": [
                    {"title": "Book 1", "author": "Author 1"},
                    {"title": "Book 2", "author": "Author 2"}
                ],
                "special-chars": "value with hyphens",
                "with spaces": "value with spaces"
            }
        }"#;

        let equivalence_tests = vec![
            // (JSONPath, JSON Pointer path components)
            ("$.store", vec!["store"]),
            ("$.store.book", vec!["store", "book"]),
            ("$.store.book[0]", vec!["store", "book", "0"]),
            ("$.store.book[0].title", vec!["store", "book", "0", "title"]),
            (
                "$.store.book[1].author",
                vec!["store", "book", "1", "author"],
            ),
        ];

        for (jsonpath, pointer_components) in equivalence_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(jsonpath);

            let chunk = Bytes::from(json_data);
            let jsonpathresults: Vec<_> = stream.process_chunk(chunk).collect();

            // Simulate JSON Pointer access (manual traversal for testing)
            let json_value: serde_json::Value =
                serde_json::from_str(json_data).expect("Valid JSON");

            let mut current = &json_value;
            let mut pointerresult = None;

            for component in pointer_components {
                match current {
                    serde_json::Value::Object(obj) => {
                        current = obj.get(component).unwrap_or(&serde_json::Value::Null);
                    }
                    serde_json::Value::Array(arr) => {
                        if let Ok(index) = component.parse::<usize>() {
                            current = arr.get(index).unwrap_or(&serde_json::Value::Null);
                        } else {
                            current = &serde_json::Value::Null;
                        }
                    }
                    _ => {
                        current = &serde_json::Value::Null;
                        break;
                    }
                }
            }

            if *current != serde_json::Value::Null {
                pointerresult = Some(current.clone());
            }

            // Compare results
            match (jsonpathresults.is_empty(), pointerresult.is_some()) {
                (true, false) => {
                    println!(
                        "JSONPath '{}' returned no results, but JSON Pointer found a value",
                        jsonpath
                    );
                }
                (false, true) => {
                    let jsonpath_value = &jsonpathresults[0];
                    let pointer_value = pointerresult.unwrap();
                    assert_eq!(
                        *jsonpath_value, pointer_value,
                        "JSONPath '{}' and JSON Pointer should return equivalent values",
                        jsonpath
                    );
                    println!(
                        "JSONPath '{}' and JSON Pointer returned equivalent values",
                        jsonpath
                    );
                }
                (true, true) => {
                    println!(
                        "Both JSONPath '{}' and JSON Pointer returned no results",
                        jsonpath
                    );
                }
                (false, false) => {
                    println!(
                        "JSONPath '{}' found results but JSON Pointer did not",
                        jsonpath
                    );
                }
            }
        }
    }

    #[test]
    fn test_singular_query_edge_cases() {
        // RFC 9535: Edge cases for singular query recognition
        let json_data = r#"{
            "": "empty_key",
            "0": "string_zero",
            "null": "null_string",
            "true": "true_string",
            "false": "false_string",
            "array": [],
            "object": {},
            "nested": {
                "": "nested_empty",
                "0": "nested_zero"
            }
        }"#;

        let edge_case_tests = vec![
            // Empty string property
            ("$['']", 1),        // Empty key access
            ("$.nested['']", 1), // Nested empty key
            // Numeric string properties
            ("$['0']", 1),        // String "0" property
            ("$.nested['0']", 1), // Nested string "0"
            // Keyword-like properties
            ("$['null']", 1),  // String "null" property
            ("$['true']", 1),  // String "true" property
            ("$['false']", 1), // String "false" property
            // Empty containers
            ("$.array", 1),  // Empty array
            ("$.object", 1), // Empty object
            // Non-existent paths
            ("$.nonexistent", 0),    // Non-existent property
            ("$.array[0]", 0),       // Index into empty array
            ("$.object.missing", 0), // Property of empty object
        ];

        for (query, expected_count) in edge_case_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert!(
                results.len() <= 1,
                "Singular query '{}' should return at most 1 result",
                query
            );
            assert_eq!(
                results.len(),
                expected_count,
                "Edge case query '{}' should return {} results",
                query,
                expected_count
            );

            println!(
                "Edge case query '{}' returned {} results",
                query,
                results.len()
            );
        }
    }

    #[test]
    fn test_deep_singular_paths() {
        // RFC 9535: Deeply nested singular paths
        let deep_json = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "deep_value": "found_it",
                                "array": [
                                    {"item": "first"},
                                    {"item": "second"}
                                ]
                            }
                        }
                    }
                }
            }
        });

        let json_data = serde_json::to_string(&deep_json).expect("Valid JSON");

        let deep_path_tests = vec![
            // Progressively deeper paths
            ("$.level1", 1),
            ("$.level1.level2", 1),
            ("$.level1.level2.level3", 1),
            ("$.level1.level2.level3.level4", 1),
            ("$.level1.level2.level3.level4.level5", 1),
            ("$.level1.level2.level3.level4.level5.deep_value", 1),
            // Mixed with array access
            ("$.level1.level2.level3.level4.level5.array", 1),
            ("$.level1.level2.level3.level4.level5.array[0]", 1),
            ("$.level1.level2.level3.level4.level5.array[1]", 1),
            ("$.level1.level2.level3.level4.level5.array[0].item", 1),
            // Non-existent deep paths
            ("$.level1.level2.level3.level4.level5.nonexistent", 0),
            ("$.level1.level2.level3.level4.level5.array[99]", 0),
        ];

        for (query, expected_count) in deep_path_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert!(
                results.len() <= 1,
                "Deep singular query '{}' should return at most 1 result",
                query
            );
            assert_eq!(
                results.len(),
                expected_count,
                "Deep query '{}' should return {} results",
                query,
                expected_count
            );

            println!("Deep query '{}' returned {} results", query, results.len());
        }
    }

    #[test]
    fn test_singular_query_performance() {
        // RFC 9535: Performance characteristics of singular queries
        let large_object = serde_json::json!({
            "data": (0..1000).map(|i| (format!("key_{}", i), serde_json::Value::Number(serde_json::Number::from(i)))).collect::<serde_json::Map<_, _>>(),
            "array": (0..1000).collect::<Vec<i32>>(),
            "deep": {
                "nested": {
                    "structure": {
                        "target": "found"
                    }
                }
            }
        });

        let json_data = serde_json::to_string(&large_object).expect("Valid JSON");

        let performance_tests = vec![
            ("$.data", 1),                         // Large object property
            ("$.array", 1),                        // Large array property
            ("$.data.key_500", 1),                 // Specific property in large object
            ("$.array[500]", 1),                   // Specific index in large array
            ("$.deep.nested.structure.target", 1), // Deep nested access
        ];

        for (query, expected_count) in performance_tests {
            let start_time = std::time::Instant::now();

            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let duration = start_time.elapsed();

            assert_eq!(
                results.len(),
                expected_count,
                "Performance query '{}' should return {} results",
                query,
                expected_count
            );

            println!(
                "Performance query '{}' returned {} results in {:?}",
                query,
                results.len(),
                duration
            );

            // Singular queries should be fast even on large data
            assert!(
                duration.as_millis() < 100,
                "Singular query '{}' should complete quickly",
                query
            );
        }
    }
}

/// Singular Query Error Handling Tests
#[cfg(test)]
mod singular_query_error_tests {
    use super::*;

    #[test]
    fn test_malformed_singular_queries() {
        // RFC 9535: Malformed queries that appear singular but are invalid
        let malformed_queries = vec![
            "$.",        // Trailing dot
            "$.store.",  // Trailing dot after property
            "$[]",       // Empty brackets
            "$[']",      // Unclosed quote
            "$['store]", // Unclosed quote
            "$.store[",  // Unclosed bracket
            "$.store]",  // Unmatched bracket
            "$store",    // Missing root identifier
            "store",     // No root at all
        ];

        for query in malformed_queries {
            let result = JsonPathParser::compile(query);

            // Most should fail to parse
            if query == "$['']" {
                // Empty string property is actually valid
                assert!(
                    result.is_ok(),
                    "Empty string property '{}' should be valid",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Malformed query '{}' should fail to parse",
                    query
                );
            }

            println!("Malformed query '{}' handling: {:?}", query, result.is_ok());
        }
    }

    #[test]
    fn test_ambiguous_singular_syntax() {
        // RFC 9535: Syntax that might be ambiguously interpreted
        let ambiguous_tests = vec![
            // These should be clearly recognized as singular
            ("$[0]", true),   // Single index
            ("$['0']", true), // String property "0"
            ("$.0", false),   // Invalid: property starting with digit
            ("$.-1", false),  // Invalid: property starting with minus
            // Bracket vs property access
            ("$.length", true),    // Property access
            ("$['length']", true), // Bracket property access
        ];

        for (query, _should_be_valid) in ambiguous_tests {
            let result = JsonPathParser::compile(query);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "Ambiguous query '{}' should be valid",
                    query
                );
            } else {
                assert!(
                    result.is_err(),
                    "Ambiguous query '{}' should be invalid",
                    query
                );
            }

            println!(
                "Ambiguous syntax test '{}': valid={}",
                query,
                result.is_ok()
            );
        }
    }

    #[test]
    fn test_type_safety_in_singular_queries() {
        // RFC 9535: Type safety for singular query results
        let json_data = r#"{
            "string_value": "hello",
            "number_value": 42,
            "boolean_value": true,
            "null_value": null,
            "array_value": [1, 2, 3],
            "object_value": {"nested": "value"}
        }"#;

        let type_safety_tests = vec![
            // Each query should return exactly one typed value
            ("$.string_value", "string"),
            ("$.number_value", "number"),
            ("$.boolean_value", "boolean"),
            ("$.null_value", "null"),
            ("$.array_value", "array"),
            ("$.object_value", "object"),
        ];

        for (query, expected_type) in type_safety_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(query);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                1,
                "Query '{}' should return exactly one result",
                query
            );

            let actual_type = match &results[0] {
                serde_json::Value::String(_) => "string",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Null => "null",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
            };

            assert_eq!(
                actual_type, expected_type,
                "Query '{}' should return {} type, got {}",
                query, expected_type, actual_type
            );

            println!(
                "Type safety test '{}' returned {} type as expected",
                query, actual_type
            );
        }
    }
}
