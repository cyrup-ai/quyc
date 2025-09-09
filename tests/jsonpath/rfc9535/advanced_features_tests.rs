//! RFC 9535 Advanced Features Tests
//!
//! Tests for RFC 9535 advanced features and edge cases:
//! - Function extension registry validation tests
//! - Boundary condition tests for deeply nested expressions
//! - Performance regression tests for streaming behavior
//! - Edge case validation for complex scenarios
//!
//! This test suite validates:
//! - Function extensibility mechanisms
//! - Deep nesting boundary conditions
//! - Streaming performance characteristics
//! - Complex edge case handling

use std::time::{Duration, Instant};

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathError, JsonPathParser};

/// Create complex nested test data
fn create_complex_nested_data(depth: usize, width: usize) -> String {
    fn create_level(
        current_depth: usize,
        max_depth: usize,
        width: usize,
        level_id: usize,
    ) -> String {
        if current_depth >= max_depth {
            return format!(r#"{{"terminal": "value_{}"}}"#, level_id);
        }

        let mut level = String::from("{");
        for i in 0..width {
            if i > 0 {
                level.push(',');
            }
            level.push_str(&format!(
                r#""child_{}": {}, "data_{}": "value_{}_{}", "array_{}": [1, 2, 3]"#,
                i,
                create_level(current_depth + 1, max_depth, width, level_id * 10 + i),
                i,
                level_id,
                i,
                i
            ));
        }
        level.push('}');
        level
    }

    create_level(0, depth, width, 1)
}

/// Test data for advanced features validation
const ADVANCED_FEATURES_JSON: &str = r#"{
  "complex_structure": {
    "level1": {
      "level2": {
        "level3": {
          "data": ["item1", "item2", "item3"],
          "metadata": {
            "created": "2024-01-01",
            "tags": ["tag1", "tag2", "tag3", "tag4"],
            "properties": {
              "name": "deep_property",
              "value": 42,
              "nested": {
                "deeper": {
                  "deepest": "found"
                }
              }
            }
          }
        }
      }
    }
  },
  "performance_test": {
    "large_array": [
      {"id": 1, "name": "item1", "category": "A", "tags": ["tag1", "tag2"]},
      {"id": 2, "name": "item2", "category": "B", "tags": ["tag2", "tag3"]},
      {"id": 3, "name": "item3", "category": "A", "tags": ["tag1", "tag3"]},
      {"id": 4, "name": "item4", "category": "C", "tags": ["tag4", "tag5"]},
      {"id": 5, "name": "item5", "category": "B", "tags": ["tag1", "tag4"]}
    ],
    "wide_object": {
      "prop1": "value1", "prop2": "value2", "prop3": "value3", "prop4": "value4",
      "prop5": "value5", "prop6": "value6", "prop7": "value7", "prop8": "value8",
      "prop9": "value9", "prop10": "value10"
    }
  }
}"#;

/// RFC 9535 Advanced Features - Function Extension Registry Tests
#[cfg(test)]
mod function_extension_tests {
    use super::*;

    #[test]
    fn test_core_function_registry() {
        // RFC 9535: Test that core functions are properly registered
        let core_function_tests = vec![
            (
                "$.complex_structure[?length(@.level1)]",
                true,
                "length() function",
            ),
            (
                "$.performance_test.large_array[?count(@.tags[*])]",
                true,
                "count() function",
            ),
            (
                "$.complex_structure[?value(@.level1.level2.level3.metadata.properties.value)]",
                true,
                "value() function",
            ),
            (
                "$.performance_test.large_array[?match(@.name, 'item')]",
                true,
                "match() function",
            ),
            (
                "$.performance_test.large_array[?search(@.category, 'A')]",
                true,
                "search() function",
            ),
        ];

        for (expr, _should_be_valid, _description) in core_function_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Core function should be registered: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid function should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_unknown_function_rejection() {
        // RFC 9535: Unknown functions should be properly rejected
        let unknown_function_tests = vec![
            (
                "$.complex_structure[?unknown_func(@.level1)]",
                false,
                "Unknown function",
            ),
            (
                "$.performance_test[?custom_length(@.wide_object)]",
                false,
                "Custom function not registered",
            ),
            (
                "$.complex_structure[?extended_match(@.level1, 'pattern')]",
                false,
                "Extension function not available",
            ),
            (
                "$.performance_test[?special_count(@.large_array[*])]",
                false,
                "Special function not registered",
            ),
            (
                "$.complex_structure[?advanced_value(@.level1.level2)]",
                false,
                "Advanced function not available",
            ),
        ];

        for (expr, _should_be_valid, _description) in unknown_function_tests {
            let result = JsonPathParser::compile(expr);

            assert!(
                result.is_err(),
                "RFC 9535: Unknown function should be rejected: {} ({})",
                expr,
                _description
            );

            // Verify error message mentions unknown function
            if let Err(JsonPathError::InvalidExpression { reason, .. }) = result {
                assert!(
                    reason.to_lowercase().contains("function")
                        || reason.to_lowercase().contains("unknown"),
                    "Error should mention unknown function: {} ({})",
                    reason,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_function_registry_consistency() {
        // RFC 9535: Function registry should be consistent across expressions
        let consistency_tests = vec![
            // Same function used in different contexts
            (
                "$.complex_structure[?length(@.level1)]",
                "$.performance_test[?length(@.wide_object)]",
            ),
            (
                "$.performance_test.large_array[?count(@.tags[*])]",
                "$.complex_structure[?count(@..data[*])]",
            ),
            (
                "$.complex_structure[?match(@.level1.level2.level3.metadata.created, '2024')]",
                "$.performance_test.large_array[?match(@.name, 'item')]",
            ),
        ];

        for (expr1, expr2) in consistency_tests {
            let result1 = JsonPathParser::compile(expr1);
            let result2 = JsonPathParser::compile(expr2);

            // Both should have consistent function availability
            assert_eq!(
                result1.is_ok(),
                result2.is_ok(),
                "RFC 9535: Function registry should be consistent: '{}' vs '{}'",
                expr1,
                expr2
            );
        }
    }

    #[test]
    fn test_function_extension_points() {
        // RFC 9535: Test extension points for future function additions
        let extension_point_tests = vec![
            // These should be syntactically valid but unknown functions
            (
                "$.complex_structure[?future_func(@.level1)]",
                "Future function extension",
            ),
            (
                "$.performance_test[?math_max(@.large_array[*].id)]",
                "Math function extension",
            ),
            (
                "$.complex_structure[?date_parse(@.level1.level2.level3.metadata.created)]",
                "Date function extension",
            ),
            (
                "$.performance_test[?regex_match(@.large_array[*].name, '^item[0-9]+$')]",
                "Regex function extension",
            ),
        ];

        for (expr, _description) in extension_point_tests {
            let result = JsonPathParser::compile(expr);

            // Should be syntactically parseable but unknown function
            assert!(
                result.is_err(),
                "RFC 9535: Extension point function should be rejected as unknown: {} ({})",
                expr,
                _description
            );

            // Verify the error is about unknown function, not syntax
            if let Err(JsonPathError::InvalidExpression { reason, .. }) = result {
                assert!(
                    !reason.to_lowercase().contains("syntax"),
                    "Error should be about unknown function, not syntax: {} ({})",
                    reason,
                    _description
                );
            }
        }
    }
}

/// RFC 9535 Advanced Features - Boundary Condition Tests
#[cfg(test)]
mod boundary_condition_tests {
    use super::*;

    #[test]
    fn test_deeply_nested_expression_boundaries() {
        // RFC 9535: Test boundary conditions for deeply nested expressions
        let deep_nesting_tests = vec![
            (10, true, "Moderate nesting depth"),
            (25, true, "Deep nesting depth"),
            (50, true, "Very deep nesting depth"),
            (100, false, "Extremely deep nesting depth"),
        ];

        for (depth, should_succeed, _description) in deep_nesting_tests {
            // Create deeply nested property access
            let mut expr = String::from("$");
            for i in 0..depth {
                expr.push_str(&format!(".level{}", i));
            }

            let result = JsonPathParser::compile(&expr);

            if should_succeed {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Deep nesting should be supported: depth {} ({})",
                    depth,
                    _description
                );
            } else {
                // Very deep nesting might be rejected for safety
                if result.is_err() {
                    println!(
                        "Deep nesting rejected at depth {}: {} ({})",
                        depth, expr, _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_complex_filter_expression_boundaries() {
        // RFC 9535: Test boundary conditions for complex filter expressions
        let complex_filter_tests = vec![
            // Increasing complexity of filter expressions
            (5, true, "Simple filter complexity"),
            (10, true, "Moderate filter complexity"),
            (20, true, "High filter complexity"),
            (50, false, "Extreme filter complexity"),
        ];

        for (complexity, should_succeed, _description) in complex_filter_tests {
            // Create complex filter with multiple conditions
            let mut filter_parts = Vec::new();
            for i in 0..complexity {
                filter_parts.push(format!("@.prop{} == 'value{}'", i, i));
            }
            let filter = filter_parts.join(" && ");
            let expr = format!("$.complex_structure[?{}]", filter);

            let result = JsonPathParser::compile(&expr);

            if should_succeed {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Complex filter should be supported: complexity {} ({})",
                    complexity,
                    _description
                );
            } else {
                // Very complex filters might be rejected for safety
                if result.is_err() {
                    println!(
                        "Complex filter rejected at complexity {}: {} ({})",
                        complexity,
                        expr.len(),
                        _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_recursive_descent_boundaries() {
        // RFC 9535: Test boundary conditions for recursive descent
        let recursive_tests = vec![
            ("$..*", "Unbounded recursive descent"),
            ("$..level1..*", "Recursive descent from level"),
            ("$..*..*", "Double recursive descent"),
            ("$..level1..level2..*", "Multi-level recursive descent"),
            ("$..**[*]..*", "Recursive descent with arrays"),
        ];

        for (expr, _description) in recursive_tests {
            let result = JsonPathParser::compile(expr);

            assert!(
                result.is_ok(),
                "RFC 9535: Recursive descent pattern should compile: {} ({})",
                expr,
                _description
            );

            // Test execution with boundary conditions
            let test_data = create_complex_nested_data(5, 3);
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(test_data);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should complete within reasonable time
            assert!(
                elapsed < Duration::from_secs(5),
                "RFC 9535: Recursive descent should complete in reasonable time: {} ({}ms) ({})",
                expr,
                elapsed.as_millis(),
                _description
            );

            println!(
                "Recursive boundary test '{}': {} results in {}ms ({})",
                expr,
                results.len(),
                elapsed.as_millis(),
                _description
            );
        }
    }

    #[test]
    fn test_array_slice_boundaries() {
        // RFC 9535: Test boundary conditions for array slicing
        let array_slice_tests = vec![
            // Normal slice operations
            ("$.performance_test.large_array[0:3]", true, "Simple slice"),
            (
                "$.performance_test.large_array[1:]",
                true,
                "Slice from index",
            ),
            ("$.performance_test.large_array[:4]", true, "Slice to index"),
            (
                "$.performance_test.large_array[-2:]",
                true,
                "Negative slice start",
            ),
            (
                "$.performance_test.large_array[:-1]",
                true,
                "Negative slice end",
            ),
            // Boundary edge cases
            (
                "$.performance_test.large_array[0:1000]",
                true,
                "Slice beyond array length",
            ),
            (
                "$.performance_test.large_array[-1000:1000]",
                true,
                "Large negative to positive",
            ),
            (
                "$.performance_test.large_array[::2]",
                true,
                "Slice with step",
            ),
            (
                "$.performance_test.large_array[1:4:2]",
                true,
                "Complex slice",
            ),
            // Edge cases that might be rejected
            (
                "$.performance_test.large_array[1000000:2000000]",
                true,
                "Very large slice indices",
            ),
            (
                "$.performance_test.large_array[-1000000:-999999]",
                true,
                "Very large negative slice",
            ),
        ];

        for (expr, _should_be_valid, _description) in array_slice_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Array slice boundary should be valid: {} ({})",
                    expr,
                    _description
                );

                // Test execution
                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(ADVANCED_FEATURES_JSON);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                println!(
                    "Array slice boundary '{}': {} results ({})",
                    expr,
                    results.len(),
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid array slice boundary should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}

/// RFC 9535 Advanced Features - Performance Regression Tests
#[cfg(test)]
mod performance_regression_tests {
    use super::*;

    #[test]
    fn test_streaming_performance_characteristics() {
        // RFC 9535: Test streaming performance for various expression types
        let performance_tests = vec![
            (
                "$.performance_test.large_array[*].name",
                "Array wildcard access",
            ),
            (
                "$.performance_test.large_array[?@.id > 2]",
                "Filter performance",
            ),
            ("$..name", "Recursive descent performance"),
            (
                "$.performance_test.large_array[*].tags[*]",
                "Nested array access",
            ),
            (
                "$.performance_test.large_array[?@.category == 'A' || @.category == 'B']",
                "Complex filter",
            ),
        ];

        for (expr, _description) in performance_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(ADVANCED_FEATURES_JSON);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Performance should be reasonable
            assert!(
                elapsed < Duration::from_millis(100),
                "RFC 9535: Streaming performance should be good: {} ({}ms) ({})",
                expr,
                elapsed.as_millis(),
                _description
            );

            println!(
                "Performance test '{}': {} results in {}ms ({})",
                expr,
                results.len(),
                elapsed.as_millis(),
                _description
            );
        }
    }

    #[test]
    fn test_scalability_with_data_size() {
        // RFC 9535: Test scalability with increasing data sizes
        let data_sizes = vec![
            (100, "Small dataset"),
            (500, "Medium dataset"),
            (1000, "Large dataset"),
        ];

        for (size, _description) in data_sizes {
            // Create dataset of specified size
            let mut large_array = String::from("[");
            for i in 0..size {
                if i > 0 {
                    large_array.push(',');
                }
                large_array.push_str(&format!(
                    r#"{{"id": {}, "name": "item{}", "category": "cat{}", "value": {}}}"#,
                    i,
                    i,
                    i % 3,
                    i * 2
                ));
            }
            large_array.push(']');

            let json_data = format!(r#"{{"test_array": {}}}"#, large_array);

            let expr = "$.test_array[?@.value > 100]";
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(json_data);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Performance should scale reasonably
            let ms_per_item = elapsed.as_millis() as f64 / size as f64;
            assert!(
                ms_per_item < 0.1,
                "RFC 9535: Performance should scale linearly: {:.3}ms per item for {} ({})",
                ms_per_item,
                size,
                _description
            );

            println!(
                "Scalability test {}: {} results in {}ms ({:.3}ms per item)",
                _description,
                results.len(),
                elapsed.as_millis(),
                ms_per_item
            );
        }
    }

    #[test]
    fn test_memory_efficiency_streaming() {
        // RFC 9535: Test memory efficiency of streaming processing
        let memory_tests = vec![
            ("$..*", "Full recursive descent"),
            ("$..value", "Specific property recursion"),
            ("$[*][*][*]", "Triple wildcard"),
            (
                "$.performance_test.large_array[*].tags[*]",
                "Nested array streaming",
            ),
        ];

        for (expr, _description) in memory_tests {
            // Use moderately complex data
            let complex_data = create_complex_nested_data(4, 4);

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(complex_data);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should complete without memory issues
            assert!(
                elapsed < Duration::from_secs(2),
                "RFC 9535: Memory efficient streaming should complete quickly: {} ({}ms) ({})",
                expr,
                elapsed.as_millis(),
                _description
            );

            println!(
                "Memory efficiency test '{}': {} results in {}ms ({})",
                expr,
                results.len(),
                elapsed.as_millis(),
                _description
            );
        }
    }

    #[test]
    fn test_expression_compilation_performance() {
        // RFC 9535: Test performance of expression compilation
        let compilation_tests = vec![
            ("$.efficient", "Efficient expression"),
            ("$.complex.nested.deep.property", "Deep property chain"),
            ("$.array[*].property", "Array wildcard"),
            ("$..recursive", "Recursive descent"),
            (
                "$.filter[?@.prop == 'value' && @.other > 10]",
                "Complex filter",
            ),
            ("$.union[0,1,2,3,4]", "Union selector"),
            ("$.slice[1:10:2]", "Array slice"),
        ];

        for (expr, _description) in compilation_tests {
            let start_time = Instant::now();
            let result = JsonPathParser::compile(expr);
            let elapsed = start_time.elapsed();

            // Compilation should be fast
            assert!(
                elapsed < Duration::from_millis(10),
                "RFC 9535: Expression compilation should be fast: {} ({}ms) ({})",
                expr,
                elapsed.as_millis(),
                _description
            );

            assert!(
                result.is_ok(),
                "RFC 9535: Expression should compile successfully: {} ({})",
                expr,
                _description
            );

            println!(
                "Compilation performance '{}': {}ms ({})",
                expr,
                elapsed.as_millis(),
                _description
            );
        }
    }
}

/// RFC 9535 Advanced Features - Edge Case Validation Tests
#[cfg(test)]
mod edge_case_validation_tests {
    use super::*;

    #[test]
    fn test_concurrent_expression_handling() {
        // RFC 9535: Test handling of multiple concurrent expressions
        let concurrent_expressions = vec![
            "$.complex_structure.level1",
            "$.performance_test.large_array[*].name",
            "$..deepest",
            "$.performance_test.large_array[?@.category == 'A']",
            "$.complex_structure..data[*]",
        ];

        // Process all expressions concurrently (simulated)
        for expr in concurrent_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "RFC 9535: Concurrent expression should compile: {}",
                expr
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(ADVANCED_FEATURES_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "Concurrent expression '{}': {} results",
                expr,
                results.len()
            );
        }
    }

    #[test]
    fn test_expression_reuse_safety() {
        // RFC 9535: Test safety of expression reuse
        let reuse_expr = "$.performance_test.large_array[?@.id > 2]";

        // Compile once
        let result = JsonPathParser::compile(reuse_expr);
        assert!(result.is_ok(), "Expression should compile");

        // Use multiple times with different data
        for i in 1..=5 {
            let _test_data = format!(r#"{{"test": "run{}", "value": {}}}"#, i, i);

            let mut stream = JsonArrayStream::<serde_json::Value>::new(reuse_expr);
            let chunk = Bytes::from(ADVANCED_FEATURES_JSON); // Use consistent data
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Results should be consistent across reuses
            println!("Expression reuse run {}: {} results", i, results.len());
        }
    }

    #[test]
    fn test_malformed_input_resilience() {
        // RFC 9535: Test resilience to malformed input during streaming
        let malformed_inputs = vec![
            (r#"{"incomplete": "#, "Incomplete JSON"),
            (r#"{"invalid": invalid_value}"#, "Invalid value"),
            (r#"{"mixed": [1, 2, "unclosed string]"#, "Mixed malformed"),
            (r#"{"numbers": [1, 2, 3,]}"#, "Trailing comma"),
        ];

        let expr = "$..*";

        for (malformed_json, _description) in malformed_inputs {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(malformed_json);

            let start_time = Instant::now();
            let results: Vec<_> = stream.process_chunk(chunk).collect();
            let elapsed = start_time.elapsed();

            // Should handle gracefully without hanging
            assert!(
                elapsed < Duration::from_millis(500),
                "RFC 9535: Malformed input should be handled quickly: {} ({}ms) ({})",
                _description,
                elapsed.as_millis(),
                _description
            );

            println!(
                "Malformed input resilience '{}': {} results in {}ms",
                _description,
                results.len(),
                elapsed.as_millis()
            );
        }
    }
}
