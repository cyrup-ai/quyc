//! RFC 9535 Function System Validation Tests (Section 2.4)
//!
//! Tests for RFC 9535 Section 2.4 complete function system:
//! - Complete function type system validation (ValueType, LogicalType, NodesType)
//! - Function argument type checking tests
//! - Function result type validation tests
//! - Type conversion and coercion rules
//!
//! This test suite validates:
//! - Function type system compliance
//! - Argument type validation and conversion
//! - Return type validation
//! - Type coercion rules and edge cases

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathError, JsonPathParser};

/// Test data for function system validation
const FUNCTION_SYSTEM_JSON: &str = r#"{
  "library": {
    "books": [
      {
        "id": 1,
        "title": "The Great Gatsby",
        "authors": ["F. Scott Fitzgerald"],
        "metadata": {
          "pages": 180,
          "year": 1925,
          "genres": ["fiction", "classic", "american"],
          "ratings": [4.2, 4.5, 3.8, 4.1]
        },
        "availability": {
          "inStock": true,
          "quantity": 5,
          "locations": ["shelf-A", "shelf-B"]
        }
      },
      {
        "id": 2,
        "title": "To Kill a Mockingbird",
        "authors": ["Harper Lee"],
        "metadata": {
          "pages": 281,
          "year": 1960,
          "genres": ["fiction", "drama", "classic"],
          "ratings": [4.8, 4.6, 4.9, 4.7]
        },
        "availability": {
          "inStock": false,
          "quantity": 0,
          "locations": []
        }
      },
      {
        "id": 3,
        "title": "1984",
        "authors": ["George Orwell"],
        "metadata": {
          "pages": 328,
          "year": 1949,
          "genres": ["dystopian", "political", "science-fiction"],
          "ratings": [4.4, 4.3, 4.6, 4.2, 4.5]
        },
        "availability": {
          "inStock": true,
          "quantity": 3,
          "locations": ["shelf-C"]
        }
      }
    ],
    "config": {
      "name": "City Library System",
      "version": "2.1.0",
      "features": ["search", "reservation", "digital-lending"]
    }
  }
}"#;

/// RFC 9535 Section 2.4 - Function Type System Tests
#[cfg(test)]
mod function_type_system_tests {
    use super::*;

    #[test]
    fn test_valuetype_function_parameters() {
        // RFC 9535: Functions that expect ValueType parameters
        let valuetype_tests = vec![
            // length() function - expects ValueType (string, array, object)
            (
                "$.library.books[?length(@.title) > 0]",
                true,
                "length() with string ValueType",
            ),
            (
                "$.library.books[?length(@.authors) > 0]",
                true,
                "length() with array ValueType",
            ),
            (
                "$.library.books[?length(@.metadata) > 0]",
                true,
                "length() with object ValueType",
            ),
            // value() function - converts NodesType to ValueType
            (
                "$.library.books[?length(value(@.title)) > 10]",
                true,
                "value() to ValueType conversion",
            ),
            (
                "$.library.books[?value(@.id) > 0]",
                true,
                "value() with numeric result",
            ),
            // String functions expecting ValueType
            (
                "$.library.books[?match(@.title, 'Great')]",
                true,
                "match() with string ValueType",
            ),
            (
                "$.library.books[?search(@.title, 'Kill')]",
                true,
                "search() with string ValueType",
            ),
            // Invalid ValueType usage
            (
                "$.library.books[?length(@.authors[*])]",
                false,
                "length() with multi-node (NodesType)",
            ),
            (
                "$.library.books[?length(@..title)]",
                false,
                "length() with recursive descent",
            ),
            (
                "$.library.books[?match(@.authors, 'pattern')]",
                false,
                "match() with array instead of string",
            ),
        ];

        for (expr, _should_be_valid, _description) in valuetype_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid ValueType function should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid ValueType function should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_nodestype_function_parameters() {
        // RFC 9535: Functions that expect NodesType parameters
        let nodestype_tests = vec![
            // count() function - expects NodesType (nodelist)
            (
                "$.library.books[?count(@.authors) > 0]",
                true,
                "count() with single node",
            ),
            (
                "$.library.books[?count(@.authors[*]) > 0]",
                true,
                "count() with multi-node list",
            ),
            (
                "$.library.books[?count(@..genres) > 5]",
                true,
                "count() with recursive descent",
            ),
            (
                "$.library.books[?count(@.metadata.ratings[*]) > 3]",
                true,
                "count() with array elements",
            ),
            // value() function - expects NodesType, converts to ValueType
            (
                "$.library.books[?value(@.id)]",
                true,
                "value() with single node",
            ),
            (
                "$.library.books[?value(@.availability.inStock)]",
                true,
                "value() with boolean node",
            ),
            // Invalid NodesType usage
            (
                "$.library.books[?count(@.title)]",
                false,
                "count() with single ValueType property",
            ),
            (
                "$.library.books[?count('literal string')]",
                false,
                "count() with literal string",
            ),
            (
                "$.library.books[?count(123)]",
                false,
                "count() with literal number",
            ),
            (
                "$.library.books[?value(@.authors[*])]",
                false,
                "value() with multi-node (ambiguous)",
            ),
        ];

        for (expr, _should_be_valid, _description) in nodestype_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid NodesType function should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid NodesType function should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_logicaltype_functionresults() {
        // RFC 9535: Functions that return LogicalType (used in test expressions)
        let logicaltype_tests = vec![
            // Function results used as test expressions (LogicalType context)
            (
                "$.library.books[?length(@.title)]",
                true,
                "length() result as LogicalType",
            ),
            (
                "$.library.books[?count(@.authors)]",
                true,
                "count() result as LogicalType",
            ),
            (
                "$.library.books[?match(@.title, 'Great')]",
                true,
                "match() result as LogicalType",
            ),
            (
                "$.library.books[?search(@.title, 'Kill')]",
                true,
                "search() result as LogicalType",
            ),
            // Function results in logical expressions
            (
                "$.library.books[?length(@.title) && count(@.authors)]",
                true,
                "Functions in AND expression",
            ),
            (
                "$.library.books[?length(@.title) || count(@.metadata.genres)]",
                true,
                "Functions in OR expression",
            ),
            (
                "$.library.books[?!length(@.availability.locations)]",
                true,
                "Function in NOT expression",
            ),
            // Function results in comparisons (converted to ValueType)
            (
                "$.library.books[?length(@.title) > 10]",
                true,
                "Function result compared to number",
            ),
            (
                "$.library.books[?count(@.authors) == 1]",
                true,
                "Function count comparison",
            ),
            (
                "$.library.books[?length(@.title) != length(@.authors[0])]",
                true,
                "Function to function comparison",
            ),
        ];

        for (expr, _should_be_valid, _description) in logicaltype_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid LogicalType function should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid LogicalType function should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}

/// RFC 9535 Section 2.4 - Function Argument Type Checking Tests
#[cfg(test)]
mod function_argument_type_tests {
    use super::*;

    #[test]
    fn test_function_argument_count_validation() {
        // RFC 9535: Functions must receive correct number of arguments
        let argument_count_tests = vec![
            // Correct argument counts
            (
                "$.library.books[?length(@.title)]",
                true,
                "length() with 1 argument",
            ),
            (
                "$.library.books[?count(@.authors)]",
                true,
                "count() with 1 argument",
            ),
            (
                "$.library.books[?value(@.id)]",
                true,
                "value() with 1 argument",
            ),
            (
                "$.library.books[?match(@.title, 'pattern')]",
                true,
                "match() with 2 arguments",
            ),
            (
                "$.library.books[?search(@.title, 'pattern')]",
                true,
                "search() with 2 arguments",
            ),
            // Incorrect argument counts
            (
                "$.library.books[?length()]",
                false,
                "length() with 0 arguments",
            ),
            (
                "$.library.books[?length(@.title, @.authors)]",
                false,
                "length() with 2 arguments",
            ),
            (
                "$.library.books[?count()]",
                false,
                "count() with 0 arguments",
            ),
            (
                "$.library.books[?count(@.authors, @.metadata)]",
                false,
                "count() with 2 arguments",
            ),
            (
                "$.library.books[?value()]",
                false,
                "value() with 0 arguments",
            ),
            (
                "$.library.books[?value(@.id, @.title)]",
                false,
                "value() with 2 arguments",
            ),
            (
                "$.library.books[?match(@.title)]",
                false,
                "match() with 1 argument",
            ),
            (
                "$.library.books[?match(@.title, 'pattern', 'extra')]",
                false,
                "match() with 3 arguments",
            ),
            (
                "$.library.books[?search(@.title)]",
                false,
                "search() with 1 argument",
            ),
            (
                "$.library.books[?search(@.title, 'pattern', 'extra')]",
                false,
                "search() with 3 arguments",
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
    fn test_function_argument_type_conversion() {
        // RFC 9535: Function arguments must be convertible to expected types
        let type_conversion_tests = vec![
            // Valid type conversions
            (
                "$.library.books[?length(@.title)]",
                true,
                "String to ValueType",
            ),
            (
                "$.library.books[?length(@.authors)]",
                true,
                "Array to ValueType",
            ),
            (
                "$.library.books[?length(@.metadata)]",
                true,
                "Object to ValueType",
            ),
            (
                "$.library.books[?count(@.authors[*])]",
                true,
                "Multi-node to NodesType",
            ),
            (
                "$.library.books[?count(@..genres)]",
                true,
                "Recursive descent to NodesType",
            ),
            (
                "$.library.books[?value(@.id)]",
                true,
                "Single node to ValueType",
            ),
            // String function type conversions
            (
                "$.library.books[?match(@.title, 'Great')]",
                true,
                "String literals in match()",
            ),
            (
                "$.library.books[?search(@.authors[0], 'Lee')]",
                true,
                "Array element to string",
            ),
            // Invalid type conversions
            (
                "$.library.books[?length(123)]",
                false,
                "Number literal to ValueType",
            ),
            (
                "$.library.books[?length(true)]",
                false,
                "Boolean literal to ValueType",
            ),
            (
                "$.library.books[?count(@.title)]",
                false,
                "Single property to NodesType",
            ),
            (
                "$.library.books[?count('literal')]",
                false,
                "String literal to NodesType",
            ),
            (
                "$.library.books[?value(@.authors[*])]",
                false,
                "Multi-node to ValueType (ambiguous)",
            ),
            (
                "$.library.books[?match(@.authors, 'pattern')]",
                false,
                "Array to string in match()",
            ),
            (
                "$.library.books[?search(@.metadata, 'pattern')]",
                false,
                "Object to string in search()",
            ),
        ];

        for (expr, _should_be_valid, _description) in type_conversion_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid type conversion should compile: {} ({})",
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
    fn test_nested_function_argument_types() {
        // RFC 9535: Nested function calls must maintain type correctness
        let nested_function_tests = vec![
            // Valid nested function types
            (
                "$.library.books[?length(@.title) > count(@.authors)]",
                true,
                "ValueType vs NodesType result",
            ),
            (
                "$.library.books[?count(@.authors) == count(@.metadata.genres)]",
                true,
                "NodesType vs NodesType result",
            ),
            (
                "$.library.books[?length(@.title) == length(@.authors[0])]",
                true,
                "ValueType vs ValueType result",
            ),
            // Valid function composition with type conversion
            (
                "$.library.books[?length(value(@.title)) > 10]",
                true,
                "NodesType->ValueType->ValueType",
            ),
            (
                "$.library.books[?count(@.authors) > length(@.title) / 10]",
                true,
                "Mixed types in arithmetic",
            ),
            // Invalid nested function types
            (
                "$.library.books[?length(count(@.authors))]",
                false,
                "Number result to string function",
            ),
            (
                "$.library.books[?count(length(@.title))]",
                false,
                "Number result to nodelist function",
            ),
            (
                "$.library.books[?match(count(@.authors), 'pattern')]",
                false,
                "Number to string in match()",
            ),
            (
                "$.library.books[?search(length(@.title), 'pattern')]",
                false,
                "Number to string in search()",
            ),
            (
                "$.library.books[?value(length(@.title))]",
                false,
                "Number to NodesType in value()",
            ),
        ];

        for (expr, _should_be_valid, _description) in nested_function_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid nested function types should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid nested function types should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}

/// RFC 9535 Section 2.4 - Function Result Type Validation Tests
#[cfg(test)]
mod functionresult_type_tests {
    use super::*;

    #[test]
    fn test_functionresult_type_consistency() {
        // RFC 9535: Function results must be consistently typed
        let result_type_tests = vec![
            // Functions returning numeric values (ValueType)
            (
                "$.library.books[?length(@.title) + 1 > 10]",
                true,
                "length() numeric result",
            ),
            (
                "$.library.books[?count(@.authors) * 2 > 1]",
                true,
                "count() numeric result",
            ),
            (
                "$.library.books[?length(@.title) - count(@.authors) > 5]",
                true,
                "Numeric arithmetic",
            ),
            // Functions returning boolean values (LogicalType)
            (
                "$.library.books[?match(@.title, 'Great') == true]",
                true,
                "match() boolean result",
            ),
            (
                "$.library.books[?search(@.title, 'Kill') != false]",
                true,
                "search() boolean result",
            ),
            (
                "$.library.books[?match(@.title, 'Great') && search(@.title, 'Gatsby')]",
                true,
                "Boolean combination",
            ),
            // Functions returning values for comparison
            (
                "$.library.books[?value(@.id) == 1]",
                true,
                "value() result comparison",
            ),
            (
                "$.library.books[?value(@.availability.inStock) == true]",
                true,
                "value() boolean extraction",
            ),
            (
                "$.library.books[?value(@.metadata.year) > 1950]",
                true,
                "value() numeric extraction",
            ),
            // Mixed function result types
            (
                "$.library.books[?length(@.title) > 0 && match(@.title, 'Great')]",
                true,
                "Numeric and boolean mix",
            ),
            (
                "$.library.books[?count(@.authors) == 1 || search(@.title, 'Kill')]",
                true,
                "Count and search mix",
            ),
        ];

        for (expr, _should_be_valid, _description) in result_type_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid function result types should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid function result types should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_function_execution_with_type_validation() {
        // RFC 9535: Test actual execution with proper type handling
        let execution_tests = vec![
            (
                "$.library.books[?length(@.title) > 10]",
                2,
                "String length filtering",
            ),
            (
                "$.library.books[?count(@.authors) == 1]",
                3,
                "Author count filtering",
            ),
            (
                "$.library.books[?length(@.metadata.genres) > 2]",
                3,
                "Array length filtering",
            ),
            (
                "$.library.books[?count(@.metadata.ratings[*]) > 4]",
                1,
                "Rating count filtering",
            ),
            ("$.library.books[?value(@.id) > 1]", 2, "ID value filtering"),
            (
                "$.library.books[?value(@.availability.inStock) == true]",
                2,
                "Boolean value filtering",
            ),
            (
                "$.library.books[?match(@.title, 'Great')]",
                1,
                "Title pattern matching",
            ),
            (
                "$.library.books[?search(@.authors[0], 'Lee')]",
                1,
                "Author search filtering",
            ),
        ];

        for (expr, expected_count, _description) in execution_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(FUNCTION_SYSTEM_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: Function execution with types: {} ({}) should return {} results",
                expr,
                _description,
                expected_count
            );
        }
    }

    #[test]
    fn test_type_coercion_edge_cases() {
        // RFC 9535: Test edge cases in type coercion
        let coercion_tests = vec![
            // Edge cases with null values
            (
                "$.library.books[?value(@.missing_property)]",
                true,
                "value() with missing property",
            ),
            (
                "$.library.books[?length(@.missing_property)]",
                true,
                "length() with missing property",
            ),
            (
                "$.library.books[?count(@.missing_property)]",
                true,
                "count() with missing property",
            ),
            // Edge cases with empty collections
            (
                "$.library.books[?length(@.availability.locations)]",
                true,
                "length() with empty array",
            ),
            (
                "$.library.books[?count(@.availability.locations[*])]",
                true,
                "count() with empty array elements",
            ),
            // Edge cases with numeric vs string types
            (
                "$.library.books[?value(@.id) == '1']",
                true,
                "Numeric ID compared to string",
            ),
            (
                "$.library.books[?length(@.id)]",
                false,
                "length() with numeric value",
            ),
            (
                "$.library.books[?match(@.id, '1')]",
                false,
                "match() with numeric value",
            ),
            // Edge cases with boolean types
            (
                "$.library.books[?value(@.availability.inStock)]",
                true,
                "Boolean value extraction",
            ),
            (
                "$.library.books[?length(@.availability.inStock)]",
                false,
                "length() with boolean",
            ),
            (
                "$.library.books[?count(@.availability.inStock)]",
                false,
                "count() with boolean",
            ),
        ];

        for (expr, _should_be_valid, _description) in coercion_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid type coercion should compile: {} ({})",
                    expr,
                    _description
                );

                // Test execution for valid expressions
                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(FUNCTION_SYSTEM_JSON);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                println!(
                    "Type coercion test '{}': {} results ({})",
                    expr,
                    results.len(),
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid type coercion should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}

/// Complex Function Type System Edge Cases
#[cfg(test)]
mod complex_function_type_tests {
    use super::*;

    #[test]
    fn test_complex_type_scenarios() {
        // RFC 9535: Complex scenarios combining multiple type requirements
        let complex_scenarios = vec![
            // Multi-level function composition
            (
                "$.library.books[?length(@.title) > count(@.metadata.genres) + count(@.authors)]",
                true,
                "Complex arithmetic with functions",
            ),
            (
                "$.library.books[?match(@.title, 'Great') && length(@.authors[0]) > 5]",
                true,
                "Boolean and length combination",
            ),
            (
                "$.library.books[?count(@.metadata.ratings[*]) > length(@.title) / 5]",
                true,
                "Count vs length ratio",
            ),
            // Nested property access with functions
            (
                "$.library.books[?value(@.metadata.year) > 1900 && length(@.metadata.genres) > 2]",
                true,
                "Nested value and length",
            ),
            (
                "$.library.books[?count(@.availability.locations[*]) == value(@.availability.quantity)]",
                true,
                "Count vs value comparison",
            ),
            // Complex logical expressions with type mixing
            (
                "$.library.books[?length(@.title) > 10 || (count(@.authors) == 1 && match(@.title, 'Kill'))]",
                true,
                "Complex logical with mixed types",
            ),
            (
                "$.library.books[?(value(@.id) > 1 && length(@.authors[0]) > 5) || search(@.title, '1984')]",
                true,
                "Nested logical with functions",
            ),
        ];

        for (expr, _should_be_valid, _description) in complex_scenarios {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Complex type scenario should compile: {} ({})",
                    expr,
                    _description
                );

                // Test execution
                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(FUNCTION_SYSTEM_JSON);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                println!(
                    "Complex type scenario '{}': {} results ({})",
                    expr,
                    results.len(),
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid complex type scenario should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_function_type_error_messages() {
        // RFC 9535: Error messages should be informative for type mismatches
        let error_message_tests = vec![
            (
                "$.library.books[?length(count(@.authors))]",
                "Number argument to string function",
            ),
            (
                "$.library.books[?count(@.title)]",
                "String argument to nodelist function",
            ),
            (
                "$.library.books[?match(@.authors, 'pattern')]",
                "Array argument to string function",
            ),
            (
                "$.library.books[?value(@.authors[*])]",
                "Multi-node argument to single-value function",
            ),
            ("$.library.books[?length()]", "Missing required argument"),
            (
                "$.library.books[?count(@.title, @.authors)]",
                "Too many arguments",
            ),
        ];

        for (expr, expected_error_type) in error_message_tests {
            let result = JsonPathParser::compile(expr);

            assert!(
                result.is_err(),
                "RFC 9535: Type error should be detected: {} ({})",
                expr,
                expected_error_type
            );

            if let Err(JsonPathError::InvalidExpression { reason, .. }) = result {
                assert!(
                    !reason.is_empty(),
                    "RFC 9535: Error message should not be empty for: {}",
                    expr
                );

                println!(
                    "Function type error for '{}' ({}): {}",
                    expr, expected_error_type, reason
                );
            }
        }
    }
}

/// RFC 9535 Section 2.4.2 - Type Conversion Boundary Tests
#[cfg(test)]
mod type_conversion_boundary_tests {
    use super::*;

    #[test]
    fn test_valuetype_to_logicaltype_conversion() {
        // RFC 9535 Section 2.4.2: ValueType results used in LogicalType context
        let conversion_tests = vec![
            // ValueType function results used as LogicalType (test expressions)
            (
                "$.library.books[?length(@.title)]",
                true,
                "length() ValueType→LogicalType in test context",
            ),
            (
                "$.library.books[?count(@.authors)]",
                true,
                "count() ValueType→LogicalType in test context",
            ),
            (
                "$.library.books[?value(@.id)]",
                true,
                "value() ValueType→LogicalType in test context",
            ),
            // ValueType results in logical operations
            (
                "$.library.books[?length(@.title) && length(@.authors[0])]",
                true,
                "Multiple ValueType→LogicalType in AND",
            ),
            (
                "$.library.books[?count(@.authors) || count(@.metadata.genres)]",
                true,
                "Multiple ValueType→LogicalType in OR",
            ),
            (
                "$.library.books[?!length(@.availability.locations)]",
                true,
                "ValueType→LogicalType in NOT operation",
            ),
            // Edge cases for ValueType→LogicalType conversion
            (
                "$.library.books[?length(@.missing_property)]",
                true,
                "Missing property ValueType→LogicalType",
            ),
            (
                "$.library.books[?count(@.empty_array) || length(@.empty_string)]",
                true,
                "Empty values ValueType→LogicalType",
            ),
            (
                "$.library.books[?value(@.metadata.year) && value(@.availability.inStock)]",
                true,
                "Multiple value() conversions",
            ),
        ];

        for (expr, _should_be_valid, _description) in conversion_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: ValueType→LogicalType conversion should compile: {} ({})",
                    expr,
                    _description
                );

                // Test execution to verify conversion behavior
                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(FUNCTION_SYSTEM_JSON);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                println!(
                    "ValueType→LogicalType test '{}': {} results ({})",
                    expr,
                    results.len(),
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid ValueType→LogicalType conversion should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_nodestype_to_valuetype_conversion() {
        // RFC 9535 Section 2.4.2: NodesType to ValueType conversion via value() function
        let conversion_tests = vec![
            // Single node NodesType→ValueType conversions
            (
                "$.library.books[?value(@.id) == 1]",
                true,
                "Single node ValueType extraction",
            ),
            (
                "$.library.books[?value(@.availability.inStock) == true]",
                true,
                "Boolean ValueType extraction",
            ),
            (
                "$.library.books[?value(@.metadata.year) > 1950]",
                true,
                "Numeric ValueType extraction",
            ),
            (
                "$.library.books[?value(@.title) == 'The Great Gatsby']",
                true,
                "String ValueType extraction",
            ),
            // Edge cases for NodesType→ValueType conversion
            (
                "$.library.books[?value(@.missing_property) == null]",
                true,
                "Missing property to ValueType",
            ),
            (
                "$.library.books[?value(@.metadata) != null]",
                true,
                "Object to ValueType",
            ),
            (
                "$.library.books[?value(@.authors[0]) != null]",
                true,
                "Array element to ValueType",
            ),
            // Invalid NodesType→ValueType conversions (ambiguous multi-node)
            (
                "$.library.books[?value(@.authors[*])]",
                false,
                "Multi-node to ValueType (ambiguous)",
            ),
            (
                "$.library.books[?value(@..genres)]",
                false,
                "Recursive descent to ValueType (ambiguous)",
            ),
            (
                "$.library.books[?value(@.metadata.ratings[*])]",
                false,
                "Multi-element array to ValueType",
            ),
            // Boundary conditions
            (
                "$.library.books[?value(@) != null]",
                true,
                "Root node to ValueType",
            ),
            (
                "$.library.books[?length(value(@.title)) > 0]",
                true,
                "Nested NodesType→ValueType→ValueType",
            ),
        ];

        for (expr, _should_be_valid, _description) in conversion_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: NodesType→ValueType conversion should compile: {} ({})",
                    expr,
                    _description
                );

                // Test execution to verify conversion behavior
                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(FUNCTION_SYSTEM_JSON);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                println!(
                    "NodesType→ValueType test '{}': {} results ({})",
                    expr,
                    results.len(),
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid NodesType→ValueType conversion should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_type_conversion_with_null_values() {
        // RFC 9535 Section 2.4.2: Type conversion edge cases with null/missing values
        let null_handling_tests = vec![
            // Null value handling in type conversions
            (
                "$.library.books[?value(@.metadata) && length(@.metadata) > 0]",
                true,
                "Null check before length conversion",
            ),
            (
                "$.library.books[?count(@.missing_array) == 0]",
                true,
                "Missing array count conversion",
            ),
            (
                "$.library.books[?value(@.missing_property) || value(@.id)]",
                true,
                "Null coalescing in conversion",
            ),
            // Edge cases with empty vs null vs missing
            (
                "$.library.books[?length(@.availability.locations) >= 0]",
                true,
                "Empty array length conversion",
            ),
            (
                "$.library.books[?count(@.availability.locations[*]) >= 0]",
                true,
                "Empty array element count",
            ),
            (
                "$.library.books[?value(@.availability.quantity) >= 0]",
                true,
                "Zero value extraction",
            ),
            // Complex null handling scenarios
            (
                "$.library.books[?(@.metadata && count(@.metadata) > 0) || !@.metadata]",
                true,
                "Complex null check with conversion",
            ),
            (
                "$.library.books[?value(@.metadata.category) || value(@.id)]",
                true,
                "Nested property null handling",
            ),
        ];

        for (expr, _should_be_valid, _description) in null_handling_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Null handling type conversion should compile: {} ({})",
                    expr,
                    _description
                );

                // Test execution to verify null handling behavior
                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(FUNCTION_SYSTEM_JSON);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                println!(
                    "Null handling conversion test '{}': {} results ({})",
                    expr,
                    results.len(),
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid null handling conversion should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_complex_type_conversion_chains() {
        // RFC 9535 Section 2.4.2: Complex chained type conversions
        let chain_tests = vec![
            // Multi-step conversion chains
            (
                "$.library.books[?length(value(@.title)) > count(@.authors)]",
                true,
                "NodesType→ValueType→ValueType vs NodesType→ValueType",
            ),
            (
                "$.library.books[?value(@.id) > 0 && length(@.title) > 0]",
                true,
                "Mixed conversion types in logical",
            ),
            (
                "$.library.books[?count(@.authors) == length(@.authors)]",
                false,
                "Invalid: NodesType vs ValueType direct comparison",
            ),
            // Function composition with type conversions
            (
                "$.library.books[?length(@.title) > length(value(@.authors[0]))]",
                true,
                "Nested function type conversions",
            ),
            (
                "$.library.books[?count(@.metadata.genres) > value(@.id) - 1]",
                true,
                "Arithmetic with mixed types",
            ),
            (
                "$.library.books[?match(value(@.title), 'Great')]",
                true,
                "String function with value conversion",
            ),
            // Edge cases in conversion chains
            (
                "$.library.books[?value(length(@.title)) > 10]",
                false,
                "Invalid: ValueType→NodesType conversion",
            ),
            (
                "$.library.books[?count(value(@.title))]",
                false,
                "Invalid: ValueType→NodesType conversion",
            ),
            (
                "$.library.books[?length(count(@.authors))]",
                false,
                "Invalid: ValueType(number)→ValueType(string)",
            ),
        ];

        for (expr, _should_be_valid, _description) in chain_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Complex type conversion chain should compile: {} ({})",
                    expr,
                    _description
                );

                // Test execution for valid chains
                let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                let chunk = Bytes::from(FUNCTION_SYSTEM_JSON);
                let results: Vec<_> = stream.process_chunk(chunk).collect();

                println!(
                    "Complex conversion chain '{}': {} results ({})",
                    expr,
                    results.len(),
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid type conversion chain should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}
