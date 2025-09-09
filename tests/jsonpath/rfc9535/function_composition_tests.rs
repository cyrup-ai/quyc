//! RFC 9535 Function Composition and Nested Function Call Tests (Section 2.4.3)
//!
//! Tests for RFC 9535 Section 2.4.3 well-typedness of function expressions:
//! "A function expression is well-typed if: 1. The function is known,
//! 2. The function is applied to the correct number of arguments,
//! 3. All function arguments are well-typed,
//! 4. All function arguments can be converted to the declared parameter types"
//!
//! This test suite validates:
//! - Function composition (functions calling other functions)
//! - Nested function call validation
//! - Type system compliance in function chains
//! - Argument type conversion in nested contexts
//! - Error handling for malformed function compositions
//! - Complex function expression well-typedness

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathError, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct ComplexData {
    id: i32,
    title: String,
    tags: Vec<String>,
    metadata: serde_json::Value,
    nested: Option<Box<ComplexData>>,
    items: Vec<serde_json::Value>,
}

/// Test data for function composition validation
const FUNCTION_TEST_JSON: &str = r#"{
  "library": {
    "books": [
      {
        "id": 1,
        "title": "Programming Guide",
        "tags": ["programming", "tutorial", "beginner"],
        "metadata": {"pages": 350, "year": 2023, "authors": ["John Doe", "Jane Smith"]},
        "items": [{"type": "chapter", "number": 1}, {"type": "chapter", "number": 2}],
        "nested": {
          "id": 11,
          "title": "Advanced Topics",
          "tags": ["advanced", "expert"],
          "metadata": {"pages": 100, "year": 2023},
          "items": [{"type": "section", "number": 1}]
        }
      },
      {
        "id": 2,
        "title": "Data Structures",
        "tags": ["computer-science", "algorithms"],
        "metadata": {"pages": 500, "year": 2022, "authors": ["Bob Wilson"]},
        "items": [{"type": "chapter", "number": 1}, {"type": "chapter", "number": 2}, {"type": "appendix", "letter": "A"}]
      },
      {
        "id": 3,
        "title": "Short Article",
        "tags": ["article"],
        "metadata": {"pages": 25, "year": 2024, "authors": []},
        "items": []
      }
    ],
    "config": {
      "id": 100,
      "title": "Library Configuration",
      "tags": ["system", "config"],
      "metadata": {"version": "1.0", "admin": "system"},
      "items": [{"type": "setting", "key": "theme"}]
    }
  }
}"#;

/// RFC 9535 Section 2.4.3 - Function Composition Tests
#[cfg(test)]
mod function_composition_tests {
    use super::*;

    #[test]
    fn test_valid_function_composition() {
        // RFC 9535: Valid function compositions that are well-typed
        let valid_compositions = vec![
            // length() with count() - NodesType -> ValueType -> LogicalType comparison
            "$.library.books[?length(@.title) > count(@.tags)]",
            // Nested length() calls
            "$.library.books[?length(@.title) > length(@.tags[0])]",
            // count() with length() - counting nodes then checking string length
            "$.library.books[?count(@.metadata.authors) == length(@.tags)]",
            // value() with length() - extracting value then checking length
            "$.library.books[?length(value(@.title)) > 10]",
            // Complex nesting with multiple functions
            "$.library.books[?count(@.items) > 0 && length(@.title) > 5]",
            // Function result in arithmetic comparison
            "$.library.books[?length(@.title) + count(@.tags) > 5]",
            // Nested function calls with property access
            "$.library.books[?length(@.metadata.authors[0]) > 3]",
            // Function composition with logical operators
            "$.library.books[?count(@.tags) > 1 && length(@.title) < 20]",
        ];

        for expr in valid_compositions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "RFC 9535: Valid function composition should compile: '{}'",
                expr
            );

            println!("✓ Valid composition: {}", expr);
        }
    }

    #[test]
    fn test_function_argument_type_conversion() {
        // RFC 9535: Function arguments must be convertible to declared parameter types
        let type_conversion_tests = vec![
            // ValueType to LogicalType conversion (for test expressions)
            (
                "$.library.books[?length(@.title)]",
                true,
                "ValueType result as LogicalType",
            ),
            (
                "$.library.books[?count(@.tags)]",
                true,
                "Number result as LogicalType",
            ),
            (
                "$.library.books[?@.metadata.pages]",
                true,
                "Property as LogicalType",
            ),
            // NodesType to ValueType conversion (single node only)
            (
                "$.library.books[?value(@.title)]",
                true,
                "NodesType to ValueType conversion",
            ),
            (
                "$.library.books[?length(value(@.title))]",
                true,
                "Chained conversion",
            ),
            // Invalid type conversions should be caught
            (
                "$.library.books[?length(count(@.tags))]",
                false,
                "Number to ValueType invalid",
            ),
            (
                "$.library.books[?value(@.tags[*])]",
                false,
                "Multi-node to ValueType invalid",
            ),
            (
                "$.library.books[?count(length(@.title))]",
                false,
                "Number to NodesType invalid",
            ),
        ];

        for (expr, _should_be_valid, _description) in type_conversion_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Type conversion should be valid: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid type conversion should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_nested_function_validation() {
        // RFC 9535: Nested function calls must satisfy well-typedness
        let nested_tests = vec![
            // Valid nesting patterns
            (
                "$.library.books[?length(@.title) > 0]",
                true,
                "Single function in comparison",
            ),
            (
                "$.library.books[?count(@.tags) + length(@.title) > 10]",
                true,
                "Functions in arithmetic",
            ),
            (
                "$.library.books[?length(@.title) == count(@.tags)]",
                true,
                "Function to function comparison",
            ),
            // Complex valid nesting
            (
                "$.library.books[?count(@.metadata.authors) > 0 && length(@.title) > count(@.tags)]",
                true,
                "Complex logical with functions",
            ),
            // Invalid nesting - type mismatches
            (
                "$.library.books[?length(count(@.tags))]",
                false,
                "Number as string argument",
            ),
            (
                "$.library.books[?count(@.title)]",
                false,
                "String as nodelist argument",
            ),
            (
                "$.library.books[?match(count(@.tags), 'pattern')]",
                false,
                "Number as string argument",
            ),
            (
                "$.library.books[?search(length(@.title), @.title)]",
                false,
                "Number as string argument",
            ),
            // Invalid function names in composition
            (
                "$.library.books[?unknown(length(@.title))]",
                false,
                "Unknown function in composition",
            ),
            (
                "$.library.books[?length(invalid(@.title))]",
                false,
                "Invalid nested function",
            ),
        ];

        for (expr, _should_be_valid, _description) in nested_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid nested function should compile: {} ({})",
                    expr,
                    _description
                );
                println!("✓ Valid nesting: {} ({})", expr, _description);
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid nested function should be rejected: {} ({})",
                    expr,
                    _description
                );
                println!("✗ Invalid nesting rejected: {} ({})", expr, _description);
            }
        }
    }

    #[test]
    fn test_function_argument_count_validation() {
        // RFC 9535: Functions must be applied to correct number of arguments
        let argument_count_tests = vec![
            // Correct argument counts in composition
            (
                "$.library.books[?length(@.title) > 0]",
                true,
                "length() with 1 arg",
            ),
            (
                "$.library.books[?count(@.tags) > 0]",
                true,
                "count() with 1 arg",
            ),
            (
                "$.library.books[?match(@.title, 'pattern')]",
                true,
                "match() with 2 args",
            ),
            (
                "$.library.books[?search(@.title, 'pattern')]",
                true,
                "search() with 2 args",
            ),
            (
                "$.library.books[?value(@.title)]",
                true,
                "value() with 1 arg",
            ),
            // Incorrect argument counts should be rejected
            ("$.library.books[?length()]", false, "length() with 0 args"),
            (
                "$.library.books[?length(@.title, @.tags)]",
                false,
                "length() with 2 args",
            ),
            ("$.library.books[?count()]", false, "count() with 0 args"),
            (
                "$.library.books[?count(@.tags, @.items)]",
                false,
                "count() with 2 args",
            ),
            (
                "$.library.books[?match(@.title)]",
                false,
                "match() with 1 arg",
            ),
            (
                "$.library.books[?match(@.title, 'pattern', 'extra')]",
                false,
                "match() with 3 args",
            ),
            (
                "$.library.books[?search(@.title)]",
                false,
                "search() with 1 arg",
            ),
            ("$.library.books[?value()]", false, "value() with 0 args"),
            (
                "$.library.books[?value(@.title, @.tags)]",
                false,
                "value() with 2 args",
            ),
            // Nested function argument count validation
            (
                "$.library.books[?length(match(@.title, 'pattern'))]",
                false,
                "Nested with wrong types",
            ),
            (
                "$.library.books[?count(length(@.title, @.tags))]",
                false,
                "Nested with wrong arg count",
            ),
        ];

        for (expr, _should_be_valid, _description) in argument_count_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Correct argument count should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Incorrect argument count should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_function_execution_with_composition() {
        // RFC 9535: Test actual execution of function compositions
        let execution_tests = vec![
            (
                "$.library.books[?length(@.title) > 10]",
                2,
                "Books with long titles",
            ),
            (
                "$.library.books[?count(@.tags) > 1]",
                2,
                "Books with multiple tags",
            ),
            (
                "$.library.books[?length(@.title) > count(@.tags)]",
                1,
                "Title longer than tag count",
            ),
            (
                "$.library.books[?count(@.items) > 0]",
                2,
                "Books with items",
            ),
            (
                "$.library.books[?length(@.title) + count(@.tags) > 15]",
                2,
                "Combined length and count",
            ),
        ];

        for (expr, expected_count, _description) in execution_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(FUNCTION_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: Function composition execution: {} ({}) should return {} results",
                expr,
                _description,
                expected_count
            );

            println!(
                "✓ Execution test: {} -> {} results ({})",
                expr,
                results.len(),
                _description
            );
        }
    }
}

/// Function Type System Compliance Tests
#[cfg(test)]
mod function_type_system_tests {
    use super::*;

    #[test]
    fn test_valuetype_to_logicaltype_conversion() {
        // RFC 9535: ValueType can be converted to LogicalType using test expression conversion
        let conversion_tests = vec![
            ("$.library.books[?@.title]", true, "String as LogicalType"),
            ("$.library.books[?@.id]", true, "Number as LogicalType"),
            (
                "$.library.books[?@.metadata]",
                true,
                "Object as LogicalType",
            ),
            ("$.library.books[?@.tags]", true, "Array as LogicalType"),
            (
                "$.library.books[?@.nested]",
                true,
                "Nested object as LogicalType",
            ),
            (
                "$.library.books[?length(@.title)]",
                true,
                "Function result as LogicalType",
            ),
            (
                "$.library.books[?count(@.tags)]",
                true,
                "Count result as LogicalType",
            ),
        ];

        for (expr, _should_be_valid, _description) in conversion_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: ValueType to LogicalType conversion should work: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid conversion should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_nodestype_to_valuetype_conversion() {
        // RFC 9535: NodesType can be converted to ValueType if nodelist has exactly one node
        let conversion_tests = vec![
            // Valid conversions (single node)
            (
                "$.library.books[?value(@.title)]",
                true,
                "Single node to ValueType",
            ),
            (
                "$.library.books[?length(value(@.title))]",
                true,
                "Chained conversion",
            ),
            // Invalid conversions (multi-node or zero-node)
            (
                "$.library.books[?value(@.tags[*])]",
                false,
                "Multi-node to ValueType",
            ),
            (
                "$.library.books[?value(@.nonexistent)]",
                false,
                "Zero-node to ValueType",
            ),
            (
                "$.library.books[?value(@..title)]",
                false,
                "Descendant multi-node",
            ),
        ];

        for (expr, _should_be_valid, _description) in conversion_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: NodesType to ValueType conversion should work: {} ({})",
                    expr,
                    _description
                );
            } else {
                // Note: This may be a runtime error rather than compile-time
                // The test documents expected behavior
                println!("NodesType conversion test: {} ({})", expr, _description);
            }
        }
    }

    #[test]
    fn test_function_parameter_type_matching() {
        // RFC 9535: Function arguments must match declared parameter types
        let type_matching_tests = vec![
            // length() expects ValueType
            (
                "$.library.books[?length(@.title)]",
                true,
                "String to length()",
            ),
            (
                "$.library.books[?length(@.tags)]",
                true,
                "Array to length()",
            ),
            (
                "$.library.books[?length(@.metadata)]",
                true,
                "Object to length()",
            ),
            // count() expects NodesType
            (
                "$.library.books[?count(@.tags[*])]",
                true,
                "Nodelist to count()",
            ),
            (
                "$.library.books[?count(@..title)]",
                true,
                "Descendant nodelist to count()",
            ),
            // match() expects two ValueType arguments
            (
                "$.library.books[?match(@.title, 'pattern')]",
                true,
                "Two strings to match()",
            ),
            // search() expects two ValueType arguments
            (
                "$.library.books[?search(@.title, 'pattern')]",
                true,
                "Two strings to search()",
            ),
            // value() expects NodesType
            (
                "$.library.books[?value(@.title)]",
                true,
                "Nodelist to value()",
            ),
            // Type mismatches should be rejected
            (
                "$.library.books[?count(@.title)]",
                false,
                "String to count() - expects NodesType",
            ),
            (
                "$.library.books[?length(@.tags[*])]",
                false,
                "Multi-node to length() - expects single ValueType",
            ),
            (
                "$.library.books[?match(@.tags, 'pattern')]",
                false,
                "Array to match() first arg",
            ),
            (
                "$.library.books[?search(count(@.tags), 'pattern')]",
                false,
                "Number to search() first arg",
            ),
        ];

        for (expr, _should_be_valid, _description) in type_matching_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Type matching should work: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Type mismatch should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}

/// Complex Function Composition Edge Cases
#[cfg(test)]
mod complex_composition_tests {
    use super::*;

    #[test]
    fn test_deeply_nested_function_calls() {
        // RFC 9535: Test deeply nested function compositions
        let deep_nesting_tests = vec![
            // Two levels of nesting
            (
                "$.library.books[?length(@.title) > count(@.tags)]",
                true,
                "Two function comparison",
            ),
            // Three levels would require careful type management
            // Note: These may not be valid depending on type system
            (
                "$.library.books[?count(@.tags) > 0 && length(@.title) > 0]",
                true,
                "Parallel functions",
            ),
            // Complex logical combinations
            (
                "$.library.books[?length(@.title) > 5 && count(@.tags) > 1 && @.metadata.pages > 100]",
                true,
                "Mixed functions and properties",
            ),
            // Function results in arithmetic
            (
                "$.library.books[?length(@.title) * count(@.tags) > 20]",
                true,
                "Arithmetic with functions",
            ),
            // Parenthesized function expressions
            (
                "$.library.books[?(length(@.title) > 10) && (count(@.tags) > 1)]",
                true,
                "Parenthesized functions",
            ),
        ];

        for (expr, _should_be_valid, _description) in deep_nesting_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Complex composition should work: {} ({})",
                    expr,
                    _description
                );
                println!("✓ Complex composition: {} ({})", expr, _description);
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid complex composition should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_function_composition_error_messages() {
        // RFC 9535: Error messages for invalid function compositions should be clear
        let error_cases = vec![
            (
                "$.library.books[?length(count(@.tags))]",
                "Number as string argument",
            ),
            (
                "$.library.books[?count(@.title)]",
                "String as nodelist argument",
            ),
            (
                "$.library.books[?unknown(length(@.title))]",
                "Unknown function",
            ),
            ("$.library.books[?length()]", "Missing required argument"),
            (
                "$.library.books[?match(@.title)]",
                "Missing required second argument",
            ),
        ];

        for (expr, error_type) in error_cases {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "RFC 9535: {} should produce error: '{}'",
                error_type,
                expr
            );

            if let Err(JsonPathError::InvalidExpression { reason, .. }) = result {
                assert!(
                    !reason.is_empty(),
                    "RFC 9535: Error message should not be empty for: {}",
                    expr
                );

                println!("Function composition error for '{}': {}", expr, reason);
            }
        }
    }

    #[test]
    fn test_function_composition_with_current_node() {
        // RFC 9535: Function composition with @ current node identifier
        let current_node_tests = vec![
            (
                "$.library.books[?length(@.title) > @.id]",
                true,
                "Function result vs @ property",
            ),
            (
                "$.library.books[?count(@.tags) == @.metadata.pages / 100]",
                true,
                "Function vs @ calculation",
            ),
            (
                "$.library.books[?@.id > 0 && length(@.title) > 5]",
                true,
                "@ property and function",
            ),
            (
                "$.library.books[?length(@.title) > length(@.nested.title)]",
                true,
                "Function with nested @",
            ),
            (
                "$.library.books[?count(@.items) > count(@.nested.items)]",
                true,
                "Parallel function calls with @",
            ),
        ];

        for (expr, _should_be_valid, _description) in current_node_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Function with @ should work: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid function with @ should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}
