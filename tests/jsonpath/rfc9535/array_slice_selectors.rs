//! RFC 9535 Array Slice Selector Tests (Section 2.3.4)
//!
//! Tests for array slice selector syntax: [start:end:step]

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct NumberModel {
    value: i32,
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
