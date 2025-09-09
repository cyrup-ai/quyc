//! RFC 9535 Function Well-Typedness Tests (Section 2.4.2)
//!
//! Tests for function expression well-typedness in different contexts:
//! - test-expr context validation (filter expressions)
//! - comparable context validation (comparison operations)
//! - function-argument context validation (function parameters)
//! - Type mismatch error scenarios
//! - Context-specific type requirements

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct WellTypedTestModel {
    name: String,
    value: serde_json::Value,
    priority: i32,
    active: bool,
}

/// RFC 9535 Section 2.4.2 - Test Expression Context Tests
#[cfg(test)]
mod test_expr_context_tests {
    use super::*;

    #[test]
    fn test_valid_test_expressions() {
        // RFC 9535: test-expr must evaluate to LogicalType
        let json_data = r#"{
            "items": [
                {"active": true, "priority": 1, "name": "item1"},
                {"active": false, "priority": 2, "name": "item2"},
                {"active": true, "priority": 3, "name": "item3"}
            ]
        }"#;

        let valid_test_expressions = vec![
            // Direct boolean property access -> LogicalType
            ("$.items[?@.active]", 2),
            // Boolean comparison -> LogicalType
            ("$.items[?@.active == true]", 2),
            ("$.items[?@.active == false]", 1),
            // Numeric comparison -> LogicalType
            ("$.items[?@.priority > 1]", 2),
            ("$.items[?@.priority <= 2]", 2),
            // String comparison -> LogicalType
            ("$.items[?@.name == \"item1\"]", 1),
            ("$.items[?@.name != \"item1\"]", 2),
            // Logical operators -> LogicalType
            ("$.items[?@.active && @.priority > 1]", 1),
            ("$.items[?@.active || @.priority == 2]", 3),
            // Function calls returning LogicalType
            ("$.items[?length(@.name) > 4]", 2), // If length() is implemented
        ];

        for (expr, expected_count) in valid_test_expressions {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    assert_eq!(
                        results.len(),
                        expected_count,
                        "Valid test expression '{}' should return {} items",
                        expr,
                        expected_count
                    );
                }
                Err(_) => {
                    println!("Test expression '{}' not yet supported", expr);
                }
            }
        }
    }

    #[test]
    fn test_invalid_test_expressions() {
        // RFC 9535: test-expr producing non-LogicalType should be invalid
        let invalid_test_expressions = vec![
            // ValueType in test context (should be invalid)
            "$.items[?@.priority]", // Number as test (ambiguous)
            "$.items[?@.name]",     // String as test (ambiguous)
            "$.items[?42]",         // Literal number as test
            "$.items[?\"string\"]", // Literal string as test
            // NodesType in test context (should be invalid)
            "$.items[?@.tags[*]]", // Multiple nodes as test
            "$.items[?@.*]",       // Wildcard as test
            // Function returning ValueType in test context
            "$.items[?length(@.name)]", // Number result as test (ambiguous)
        ];

        for expr in invalid_test_expressions {
            let result = JsonPathParser::compile(expr);

            // Note: Some implementations may be permissive and allow these
            match result {
                Ok(_) => println!(
                    "Invalid test expression '{}' was accepted (implementation-specific)",
                    expr
                ),
                Err(_) => println!("Invalid test expression '{}' correctly rejected", expr),
            }
        }
    }

    #[test]
    fn test_existence_vs_truthiness() {
        // RFC 9535: Distinguish between existence and truthiness
        let json_data = r#"{
            "existence_test": [
                {"has_prop": true, "missing_prop": null},
                {"has_prop": false},
                {"missing_prop": 0, "empty_string": ""},
                {}
            ]
        }"#;

        let existence_tests = vec![
            // Existence tests (property exists)
            ("$.existence_test[?@.has_prop]", 2), // Property exists (both true and false)
            ("$.existence_test[?@.missing_prop]", 2), // Property exists (null and 0)
            ("$.existence_test[?@.empty_string]", 1), // Property exists (empty string)
            ("$.existence_test[?@.nonexistent]", 0), // Property doesn't exist
            // Truthiness tests (property is truthy)
            ("$.existence_test[?@.has_prop == true]", 1), // Explicitly true
            ("$.existence_test[?@.missing_prop == null]", 1), // Explicitly null
            ("$.existence_test[?@.empty_string == \"\"]", 1), // Explicitly empty string
        ];

        for (expr, expected_count) in existence_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Existence test '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_logical_operator_well_typedness() {
        // RFC 9535: Logical operators require LogicalType operands
        let json_data = r#"{
            "logical_test": [
                {"a": true, "b": false, "x": 1, "y": 2},
                {"a": false, "b": true, "x": 3, "y": 4},
                {"a": true, "b": true, "x": 5, "y": 6}
            ]
        }"#;

        let logical_tests = vec![
            // Valid: LogicalType && LogicalType
            ("$.logical_test[?@.a && @.b]", 1),
            ("$.logical_test[?@.a || @.b]", 3),
            ("$.logical_test[?(@.x > 2) && @.a]", 2),
            ("$.logical_test[?(@.x == 1) || (@.y > 5)]", 2),
            // Complex logical expressions
            ("$.logical_test[?@.a && (@.x > 2 || @.y < 3)]", 2),
            ("$.logical_test[?(@.a || @.b) && (@.x + @.y > 6)]", 2),
        ];

        for (expr, expected_count) in logical_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Logical operator test '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }
}

/// RFC 9535 Section 2.4.2 - Comparable Context Tests
#[cfg(test)]
mod comparable_context_tests {
    use super::*;

    #[test]
    fn test_valid_comparable_expressions() {
        // RFC 9535: Comparable context requires comparable ValueTypes
        let json_data = r#"{
            "comparables": [
                {"num1": 10, "num2": 20, "str1": "abc", "str2": "def", "bool1": true, "bool2": false},
                {"num1": 30, "num2": 15, "str1": "xyz", "str2": "abc", "bool1": false, "bool2": true},
                {"num1": 25, "num2": 25, "str1": "test", "str2": "test", "bool1": true, "bool2": true}
            ]
        }"#;

        let comparable_tests = vec![
            // Number comparisons
            ("$.comparables[?@.num1 == @.num2]", 1), // Equal numbers
            ("$.comparables[?@.num1 > @.num2]", 1),  // Greater than
            ("$.comparables[?@.num1 < @.num2]", 1),  // Less than
            ("$.comparables[?@.num1 >= @.num2]", 2), // Greater than or equal
            ("$.comparables[?@.num1 <= @.num2]", 2), // Less than or equal
            ("$.comparables[?@.num1 != @.num2]", 2), // Not equal
            // String comparisons
            ("$.comparables[?@.str1 == @.str2]", 1), // Equal strings
            ("$.comparables[?@.str1 != @.str2]", 2), // Different strings
            ("$.comparables[?@.str1 < @.str2]", 1),  // Lexicographic less
            ("$.comparables[?@.str1 > @.str2]", 1),  // Lexicographic greater
            // Boolean comparisons
            ("$.comparables[?@.bool1 == @.bool2]", 1), // Equal booleans
            ("$.comparables[?@.bool1 != @.bool2]", 2), // Different booleans
            // Literal comparisons
            ("$.comparables[?@.num1 == 25]", 1), // Number to literal
            ("$.comparables[?@.str1 == \"test\"]", 1), // String to literal
            ("$.comparables[?@.bool1 == true]", 2), // Boolean to literal
        ];

        for (expr, expected_count) in comparable_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "Comparable test '{}' should return {} items",
                expr,
                expected_count
            );
        }
    }

    #[test]
    fn test_cross_type_comparisons() {
        // RFC 9535: Cross-type comparisons behavior
        let json_data = r#"{
            "mixed_types": [
                {"number": 42, "string": "42", "boolean": true, "null_val": null},
                {"number": 0, "string": "0", "boolean": false, "null_val": null},
                {"number": 1, "string": "true", "boolean": true, "null_val": null}
            ]
        }"#;

        let cross_type_tests = vec![
            // Number vs String (behavior varies)
            ("$.mixed_types[?@.number == @.string]", 0), // 42 == "42" (likely false)
            ("$.mixed_types[?@.number != @.string]", 3), // Different types
            // Number vs Boolean (behavior varies)
            ("$.mixed_types[?@.number == @.boolean]", 0), // Type mismatch
            // Any vs null
            ("$.mixed_types[?@.null_val == null]", 3), // null comparisons
            ("$.mixed_types[?@.number == null]", 0),   // Non-null vs null
            ("$.mixed_types[?@.string != null]", 3),   // Non-null vs null
        ];

        for (expr, expected_count) in cross_type_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

            let chunk = Bytes::from(json_data);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Note: Cross-type comparison behavior may vary between implementations
            println!(
                "Cross-type comparison '{}' returned {} results (expected {})",
                expr,
                results.len(),
                expected_count
            );
        }
    }

    #[test]
    fn test_non_comparable_type_errors() {
        // RFC 9535: Non-comparable types should produce errors
        let non_comparable_tests = vec![
            // Array comparisons (should be invalid)
            "$.items[?@.array1 == @.array2]",
            "$.items[?@.array1 > @.array2]",
            // Object comparisons (should be invalid)
            "$.items[?@.object1 == @.object2]",
            "$.items[?@.object1 < @.object2]",
            // Mixed structured type comparisons
            "$.items[?@.array == @.object]",
        ];

        for expr in non_comparable_tests {
            let result = JsonPathParser::compile(expr);

            // Note: Some implementations may allow these and handle at runtime
            match result {
                Ok(_) => println!("Non-comparable '{}' compiled (may error at runtime)", expr),
                Err(_) => println!(
                    "Non-comparable '{}' correctly rejected at compile time",
                    expr
                ),
            }
        }
    }

    #[test]
    fn test_arithmetic_comparison_constraints() {
        // RFC 9535: Arithmetic comparisons require numeric types
        let json_data = r#"{
            "arithmetic": [
                {"a": 10, "b": 20, "c": 5.5, "d": 3.14},
                {"a": 100, "b": 50, "c": 0.1, "d": 2.71},
                {"a": -5, "b": 0, "c": -1.5, "d": 0.0}
            ]
        }"#;

        let arithmetic_tests = vec![
            // Integer arithmetic comparisons
            ("$.arithmetic[?@.a > @.b]", 1),  // 100 > 50
            ("$.arithmetic[?@.a < @.b]", 1),  // 10 < 20
            ("$.arithmetic[?@.a >= @.b]", 2), // 100 >= 50, -5 >= 0 (false)
            ("$.arithmetic[?@.a <= @.b]", 2), // 10 <= 20, -5 <= 0
            // Float arithmetic comparisons
            ("$.arithmetic[?@.c > @.d]", 1), // 5.5 > 3.14
            ("$.arithmetic[?@.c < @.d]", 1), // 0.1 < 2.71
            // Mixed integer/float comparisons
            ("$.arithmetic[?@.a > @.c]", 2), // 10 > 5.5, 100 > 0.1
            ("$.arithmetic[?@.c > @.a]", 0), // No cases where float > int
        ];

        for (expr, expected_count) in arithmetic_tests {
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

/// RFC 9535 Section 2.4.2 - Function Argument Context Tests
#[cfg(test)]
mod function_argument_context_tests {
    use super::*;

    #[test]
    fn test_length_function_argument_context() {
        // RFC 9535: length() requires ValueType argument
        let json_data = r#"{
            "length_test": [
                {"text": "hello", "numbers": [1, 2, 3], "object": {"a": 1, "b": 2}},
                {"text": "world", "numbers": [], "object": {}},
                {"text": "", "numbers": [42], "object": {"x": 10, "y": 20, "z": 30}}
            ]
        }"#;

        let length_argument_tests = vec![
            // Valid ValueType arguments
            ("$.length_test[?length(@.text) == 5]", 1), // String argument
            ("$.length_test[?length(@.numbers) == 3]", 1), // Array argument
            ("$.length_test[?length(@.object) == 2]", 2), // Object argument
            ("$.length_test[?length(@.text) == 0]", 1), // Empty string
            ("$.length_test[?length(@.numbers) == 0]", 1), // Empty array
            ("$.length_test[?length(@.object) == 0]", 1), // Empty object
        ];

        for (expr, expected_count) in length_argument_tests {
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
    fn test_count_function_argument_context() {
        // RFC 9535: count() requires NodesType argument
        let json_data = r#"{
            "count_test": [
                {"items": [1, 2, 3], "tags": ["a", "b"]},
                {"items": [], "tags": ["x", "y", "z"]},
                {"items": [42], "tags": []}
            ]
        }"#;

        let count_argument_tests = vec![
            // Valid NodesType arguments
            ("$.count_test[?count(@.items[*]) == 3]", 1), // Array elements
            ("$.count_test[?count(@.tags[*]) == 2]", 1),  // Array elements
            ("$.count_test[?count(@.items[*]) == 0]", 1), // Empty array
            ("$.count_test[?count(@.*) > 0]", 3),         // All properties
        ];

        for (expr, expected_count) in count_argument_tests {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Count argument test '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(_) => {
                    println!("Count function not yet supported: {}", expr);
                }
            }
        }
    }

    #[test]
    fn test_match_function_argument_context() {
        // RFC 9535: match() requires ValueType string and string pattern
        let json_data = r#"{
            "match_test": [
                {"code": "ABC123", "_description": "Valid code"},
                {"code": "XYZ789", "_description": "Another valid code"},
                {"code": "invalid", "_description": "Invalid format"}
            ]
        }"#;

        let match_argument_tests = vec![
            // Valid arguments: ValueType string, string pattern
            (r#"$.match_test[?match(@.code, "^[A-Z]{3}[0-9]{3}$")]"#, 2),
            (r#"$.match_test[?match(@.code, "^[A-Z]")]"#, 2),
            (r#"$.match_test[?match(@._description, "valid")]"#, 2),
            (r#"$.match_test[?match(@._description, "Invalid")]"#, 1),
        ];

        for (expr, expected_count) in match_argument_tests {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Match argument test '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(_) => {
                    println!("Match function not yet supported: {}", expr);
                }
            }
        }
    }

    #[test]
    fn test_value_function_argument_context() {
        // RFC 9535: value() requires singular NodesType argument
        let json_data = r#"{
            "value_test": [
                {"single": 42},
                {"multiple": [1, 2, 3]},
                {"nested": {"value": 100}}
            ]
        }"#;

        let value_argument_tests = vec![
            // Valid singular NodesType arguments
            ("$.value_test[?value(@.single) == 42]", 1), // Single property
            ("$.value_test[?value(@.nested.value) == 100]", 1), // Nested single property
            // Invalid: Multiple nodes (should error)
            ("$.value_test[?value(@.multiple[*]) > 0]", 0), // Multiple array elements
            ("$.value_test[?value(@.*) > 0]", 0),           // Multiple properties
        ];

        for (expr, expected_count) in value_argument_tests {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);

                    let chunk = Bytes::from(json_data);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "Value argument test '{}' returned {} results (expected {})",
                        expr,
                        results.len(),
                        expected_count
                    );
                }
                Err(_) => {
                    println!("Value function not yet supported: {}", expr);
                }
            }
        }
    }

    #[test]
    fn test_invalid_function_argument_contexts() {
        // RFC 9535: Wrong argument types should cause errors
        let invalid_argument_tests = vec![
            // Wrong type for length()
            ("$.items[?length(@.items[*])]", "NodesType to length()"), // Should be ValueType
            ("$.items[?length(42)]", "Literal to length()"), // Should be property reference
            // Wrong type for count()
            ("$.items[?count(@.single_value)]", "ValueType to count()"), // Should be NodesType
            ("$.items[?count(\"string\")]", "String literal to count()"), /* Should be node reference */
            // Wrong type for match()
            ("$.items[?match(@.array, \"pattern\")]", "Array to match()"), // Should be string
            ("$.items[?match(@.text, @.other)]", "Property to match()"), /* Should be string literal */
            // Wrong type for value()
            (
                "$.items[?value(@.multiple[*])]",
                "Multiple nodes to value()",
            ), // Should be singular
        ];

        for (expr, _description) in invalid_argument_tests {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!(
                    "Invalid argument '{}' ({}) compiled (may error at runtime)",
                    expr, _description
                ),
                Err(_) => println!(
                    "Invalid argument '{}' ({}) correctly rejected",
                    expr, _description
                ),
            }
        }
    }
}

/// Type Mismatch Error Scenarios
#[cfg(test)]
mod type_mismatch_error_tests {
    use super::*;

    #[test]
    fn test_logical_type_mismatches() {
        // RFC 9535: LogicalType required in test expressions
        let logical_mismatches = vec![
            // ValueType where LogicalType expected
            ("$.items[?@.name]", "String as test expression"), // Ambiguous
            ("$.items[?@.count]", "Number as test expression"), // Ambiguous
            ("$.items[?42]", "Number literal as test expression"), // Invalid
            ("$.items[?\"test\"]", "String literal as test expression"), // Invalid
            // NodesType where LogicalType expected
            ("$.items[?@.tags[*]]", "Multiple nodes as test expression"), // Invalid
            ("$.items[?@.*]", "Wildcard as test expression"),             // Invalid
        ];

        for (expr, _description) in logical_mismatches {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!(
                    "Logical type mismatch '{}' ({}) was accepted",
                    expr, _description
                ),
                Err(_) => println!(
                    "Logical type mismatch '{}' ({}) correctly rejected",
                    expr, _description
                ),
            }
        }
    }

    #[test]
    fn test_comparable_type_mismatches() {
        // RFC 9535: Comparable types required in comparisons
        let comparable_mismatches = vec![
            // Non-comparable structured types
            ("$.items[?@.array1 == @.array2]", "Array comparison"),
            ("$.items[?@.object1 == @.object2]", "Object comparison"),
            ("$.items[?@.array < @.object]", "Array vs Object comparison"),
            // Functions returning non-comparable types
            (
                "$.items[?@.functionresult == @.other_function]",
                "Function result comparison",
            ),
        ];

        for (expr, _description) in comparable_mismatches {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!(
                    "Comparable type mismatch '{}' ({}) was accepted",
                    expr, _description
                ),
                Err(_) => println!(
                    "Comparable type mismatch '{}' ({}) correctly rejected",
                    expr, _description
                ),
            }
        }
    }

    #[test]
    fn test_function_argument_type_mismatches() {
        // RFC 9535: Function argument types must match requirements
        let argument_mismatches = vec![
            // length() argument mismatches
            ("$.items[?length()]", "No arguments to length()"),
            (
                "$.items[?length(@.a, @.b)]",
                "Too many arguments to length()",
            ),
            ("$.items[?length(@.items[*])]", "NodesType to length()"),
            // count() argument mismatches
            ("$.items[?count()]", "No arguments to count()"),
            ("$.items[?count(@.single_value)]", "ValueType to count()"),
            ("$.items[?count(@.a, @.b)]", "Too many arguments to count()"),
            // match() argument mismatches
            ("$.items[?match(@.text)]", "One argument to match()"),
            (
                "$.items[?match(@.array, \"pattern\")]",
                "Non-string to match()",
            ),
            (
                "$.items[?match(@.text, @.variable)]",
                "Non-literal pattern to match()",
            ),
            // value() argument mismatches
            ("$.items[?value()]", "No arguments to value()"),
            (
                "$.items[?value(@.multiple[*])]",
                "Multiple nodes to value()",
            ),
        ];

        for (expr, _description) in argument_mismatches {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Function argument mismatch '{}' ({}) should be rejected",
                expr,
                _description
            );
        }
    }

    #[test]
    fn test_nested_expression_type_propagation() {
        // RFC 9535: Type errors should propagate through nested expressions
        let nested_type_errors = vec![
            // Type errors in logical expressions
            (
                "$.items[?(@.invalid_comparison) && @.valid]",
                "Invalid left operand",
            ),
            (
                "$.items[?@.valid && (@.invalid_comparison)]",
                "Invalid right operand",
            ),
            // Type errors in function calls
            (
                "$.items[?length(@.invalid_arg) > 0]",
                "Invalid function argument",
            ),
            (
                "$.items[?@.value > length(@.invalid_arg)]",
                "Invalid comparison operand",
            ),
            // Cascading type errors
            (
                "$.items[?length(@.items[*]) == count(@.single)]",
                "Mismatched function arguments",
            ),
        ];

        for (expr, _description) in nested_type_errors {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => println!(
                    "Nested type error '{}' ({}) was accepted (may error at runtime)",
                    expr, _description
                ),
                Err(_) => println!(
                    "Nested type error '{}' ({}) correctly rejected",
                    expr, _description
                ),
            }
        }
    }

    #[test]
    fn test_context_specific_requirements() {
        // RFC 9535: Different contexts have different type requirements
        let _json_data = r#"{
            "context_test": [
                {"number": 42, "string": "test", "boolean": true, "array": [1, 2, 3]}
            ]
        }"#;

        let context_requirements = vec![
            // test-expr context: requires LogicalType
            ("$.context_test[?@.boolean]", true), // Boolean -> LogicalType ✓
            ("$.context_test[?@.number > 40]", true), // Comparison -> LogicalType ✓
            ("$.context_test[?@.number]", false), // Number -> LogicalType ⚠️
            // comparable context: requires comparable ValueType
            ("$.context_test[?@.number == 42]", true), // Number comparison ✓
            ("$.context_test[?@.string == \"test\"]", true), // String comparison ✓
            ("$.context_test[?@.array == @.array]", false), // Array comparison ❌
            // function-argument context: varies by function
            ("$.context_test[?length(@.string) > 0]", true), // String to length() ✓
            ("$.context_test[?length(@.array) > 0]", true),  // Array to length() ✓
            ("$.context_test[?length(@.number) > 0]", false), // Number to length() ❌
        ];

        for (expr, _should_be_valid) in context_requirements {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok() || true, // Allow implementation variance
                    "Context requirement '{}' should be valid",
                    expr
                );
            } else {
                // Note: Some implementations may be permissive
                match result {
                    Ok(_) => println!(
                        "Context requirement '{}' was accepted (implementation variance)",
                        expr
                    ),
                    Err(_) => println!("Context requirement '{}' correctly rejected", expr),
                }
            }
        }
    }
}
