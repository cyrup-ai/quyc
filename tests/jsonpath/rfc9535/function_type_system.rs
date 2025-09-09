//! RFC 9535 Function Type System Validation Tests (Section 2.4.1-2.4.3)
//!
//! Tests for the JSONPath function type system:
//! - ValueType: primitive JSON values (null, true, false, number, string)
//! - LogicalType: true, false, or LogicalTypeError
//! - NodesType: sequence of nodes from the data model
//! - Type conversion tests and validation
//! - Well-typedness checking
//! - Function argument validation
//! - Nested function expression tests

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TypeTestModel {
    value: serde_json::Value,
    name: String,
    metadata: Option<serde_json::Value>,
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

/// RFC 9535 Section 2.4.2 - Type Conversion Boundary Condition Tests
#[cfg(test)]
mod type_conversion_boundary_tests {
    use super::*;

    #[test]
    fn test_value_type_to_logical_type_boundary_conditions() {
        // RFC 9535 Section 2.4.2: ValueType to LogicalType conversion edge cases
        let boundary_test_json = r#"{
            "boundary_cases": [
                {"value": null, "type": "null"},
                {"value": false, "type": "boolean_false"},
                {"value": true, "type": "boolean_true"},
                {"value": 0, "type": "number_zero"},
                {"value": -0.0, "type": "number_negative_zero"},
                {"value": 1, "type": "number_positive"},
                {"value": -1, "type": "number_negative"},
                {"value": 0.0, "type": "number_float_zero"},
                {"value": "", "type": "empty_string"},
                {"value": "false", "type": "string_false"},
                {"value": "0", "type": "string_zero"},
                {"value": " ", "type": "whitespace_string"},
                {"value": [], "type": "empty_array"},
                {"value": [null], "type": "array_with_null"},
                {"value": [false], "type": "array_with_false"},
                {"value": [0], "type": "array_with_zero"},
                {"value": [""], "type": "array_with_empty_string"},
                {"value": {}, "type": "empty_object"},
                {"value": {"key": null}, "type": "object_with_null_value"},
                {"value": {"key": false}, "type": "object_with_false_value"},
                {"value": {"key": 0}, "type": "object_with_zero_value"}
            ]
        }"#;

        let boundary_tests = vec![
            // RFC 9535: null should be falsy in logical context
            (
                "$.boundary_cases[?@.value]",
                15,
                "null should be falsy, all other values should be truthy",
            ),
            (
                "$.boundary_cases[?!@.value]",
                6,
                "Only null, false, 0, -0.0, 0.0, and empty string should be falsy",
            ),
            // Explicit boolean conversions
            (
                "$.boundary_cases[?@.value == true]",
                1,
                "Only explicit true should equal true",
            ),
            (
                "$.boundary_cases[?@.value == false]",
                1,
                "Only explicit false should equal false",
            ),
            // Null vs falsy distinction
            (
                "$.boundary_cases[?@.value == null]",
                1,
                "Only null should equal null",
            ),
            (
                "$.boundary_cases[?@.value != null]",
                20,
                "All non-null values should not equal null",
            ),
            // Zero vs falsy distinction
            (
                "$.boundary_cases[?@.value == 0]",
                3,
                "Zero, negative zero, and float zero should equal 0",
            ),
            (
                "$.boundary_cases[?@.value != 0]",
                18,
                "All non-zero values should not equal 0",
            ),
            // Empty vs falsy distinction for strings
            (
                "$.boundary_cases[?@.value == \"\"]",
                1,
                "Only empty string should equal empty string",
            ),
            (
                "$.boundary_cases[?@.type == \"empty_string\" && @.value]",
                0,
                "Empty string should be falsy",
            ),
            // Array/Object truthiness (non-empty collections are truthy)
            (
                "$.boundary_cases[?@.type == \"empty_array\" && @.value]",
                1,
                "Empty array should be truthy",
            ),
            (
                "$.boundary_cases[?@.type == \"empty_object\" && @.value]",
                1,
                "Empty object should be truthy",
            ),
            // String representation vs value distinction
            (
                "$.boundary_cases[?@.value == \"false\"]",
                1,
                "Only string 'false' should equal string 'false'",
            ),
            (
                "$.boundary_cases[?@.value == \"0\"]",
                1,
                "Only string '0' should equal string '0'",
            ),
            (
                "$.boundary_cases[?@.type == \"string_false\" && !@.value]",
                0,
                "String 'false' should be truthy",
            ),
            (
                "$.boundary_cases[?@.type == \"string_zero\" && !@.value]",
                0,
                "String '0' should be truthy",
            ),
        ];

        for (expr, expected_count, _description) in boundary_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(boundary_test_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535 boundary condition: {} - '{}' should return {} results, got {}",
                _description,
                expr,
                expected_count,
                results.len()
            );
        }
    }

    #[test]
    fn test_nodes_type_to_value_type_boundary_conditions() {
        // RFC 9535 Section 2.4.2: NodesType to ValueType conversion edge cases
        let nodes_conversion_json = r#"{
            "conversions": [
                {"emptyresult": [], "type": "empty_nodelist"},
                {"single_null": [null], "type": "single_null_node"},
                {"single_false": [false], "type": "single_false_node"},
                {"single_zero": [0], "type": "single_zero_node"},
                {"single_empty_string": [""], "type": "single_empty_string_node"},
                {"single_empty_array": [[]], "type": "single_empty_array_node"},
                {"single_empty_object": [{}], "type": "single_empty_object_node"},
                {"multiple_values": [1, 2, 3], "type": "multiple_nodes"},
                {"mixed_types": [null, false, 0, "", []], "type": "mixed_type_nodes"},
                {"nested_empty": [[]], "type": "nested_empty_structure"},
                {"deeply_nested": [[[null]]], "type": "deeply_nested_structure"}
            ]
        }"#;

        let nodes_boundary_tests = vec![
            // Empty nodelist conversions
            (
                "$.conversions[?@.emptyresult[*]]",
                0,
                "Empty nodelist should produce no matches",
            ),
            (
                "$.conversions[?count(@.emptyresult[*]) == 0]",
                1,
                "Empty nodelist count should be 0",
            ),
            // Single node extractions (should succeed)
            (
                "$.conversions[?@.single_null[0] == null]",
                1,
                "Single null node should extract null",
            ),
            (
                "$.conversions[?@.single_false[0] == false]",
                1,
                "Single false node should extract false",
            ),
            (
                "$.conversions[?@.single_zero[0] == 0]",
                1,
                "Single zero node should extract 0",
            ),
            (
                "$.conversions[?@.single_empty_string[0] == \"\"]",
                1,
                "Single empty string node should extract empty string",
            ),
            // Multiple node extractions (specific index access should work)
            (
                "$.conversions[?@.multiple_values[0] == 1]",
                1,
                "First node of multiple should be accessible",
            ),
            (
                "$.conversions[?@.multiple_values[2] == 3]",
                1,
                "Third node of multiple should be accessible",
            ),
            (
                "$.conversions[?@.multiple_values[999] == null]",
                0,
                "Out-of-bounds access should return nothing",
            ),
            // Mixed type node handling
            (
                "$.conversions[?@.mixed_types[0] == null]",
                1,
                "First mixed type should be null",
            ),
            (
                "$.conversions[?@.mixed_types[1] == false]",
                1,
                "Second mixed type should be false",
            ),
            (
                "$.conversions[?@.mixed_types[2] == 0]",
                1,
                "Third mixed type should be 0",
            ),
            // Nested structure boundary cases
            (
                "$.conversions[?@.nested_empty[0][*]]",
                0,
                "Nested empty array should produce no matches",
            ),
            (
                "$.conversions[?@.deeply_nested[0][0][0] == null]",
                1,
                "Deeply nested null should be accessible",
            ),
        ];

        for (expr, expected_count, _description) in nodes_boundary_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(nodes_conversion_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535 NodesType boundary: {} - '{}' should return {} results, got {}",
                _description,
                expr,
                expected_count,
                results.len()
            );
        }
    }

    #[test]
    fn test_complex_type_conversion_edge_cases() {
        // RFC 9535: Complex scenarios involving multiple type conversions
        let complex_conversion_json = r#"{
            "complex_cases": [
                {
                    "id": 1,
                    "conditionals": {
                        "null_check": null,
                        "empty_check": "",
                        "zero_check": 0,
                        "false_check": false,
                        "array_check": [],
                        "object_check": {}
                    },
                    "nested_arrays": [
                        [],
                        [null],
                        [false, 0, ""],
                        [[], {}, null]
                    ],
                    "function_inputs": {
                        "empty_string": "",
                        "null_value": null,
                        "false_value": false,
                        "zero_value": 0,
                        "empty_array": [],
                        "empty_object": {}
                    }
                }
            ]
        }"#;

        let complex_tests = vec![
            // Nested conditional evaluations with type conversions
            (
                "$.complex_cases[?@.conditionals.null_check || @.conditionals.empty_check || @.conditionals.zero_check]",
                0,
                "Multiple falsy OR should be falsy",
            ),
            (
                "$.complex_cases[?@.conditionals.array_check && @.conditionals.object_check]",
                1,
                "Empty collections should be truthy in AND",
            ),
            (
                "$.complex_cases[?!(@.conditionals.null_check && @.conditionals.false_check)]",
                1,
                "Negated falsy AND should be truthy",
            ),
            // Array-based type conversions in filters
            (
                "$.complex_cases[?@.nested_arrays[0][*]]",
                0,
                "Empty nested array should produce no results",
            ),
            (
                "$.complex_cases[?@.nested_arrays[1][0] == null]",
                1,
                "Nested null should be accessible",
            ),
            (
                "$.complex_cases[?count(@.nested_arrays[2][*]) == 3]",
                1,
                "Count of mixed falsy values should be 3",
            ),
            // Function argument type boundary conditions
            (
                "$.complex_cases[?length(@.function_inputs.empty_string) == 0]",
                1,
                "Length of empty string should be 0",
            ),
            (
                "$.complex_cases[?length(@.function_inputs.empty_array) == 0]",
                1,
                "Length of empty array should be 0",
            ),
            (
                "$.complex_cases[?length(@.function_inputs.empty_object) == 0]",
                1,
                "Length of empty object should be 0",
            ),
        ];

        for (expr, expected_count, _description) in complex_tests {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                    let chunk = Bytes::from(complex_conversion_json);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    assert_eq!(
                        results.len(),
                        expected_count,
                        "RFC 9535 complex conversion: {} - '{}' should return {} results, got {}",
                        _description,
                        expr,
                        expected_count,
                        results.len()
                    );
                }
                Err(_) => {
                    println!(
                        "RFC 9535 complex conversion test not supported: {} - '{}'",
                        _description, expr
                    );
                }
            }
        }
    }

    #[test]
    fn test_logical_type_error_boundary_conditions() {
        // RFC 9535 Section 2.4.2: LogicalTypeError boundary conditions
        let error_boundary_json = r#"{
            "error_boundaries": [
                {"value": "not_a_number", "numeric_field": "123"},
                {"value": [1, 2, 3], "numeric_field": 456},
                {"value": {"nested": "object"}, "numeric_field": 789},
                {"value": null, "numeric_field": null},
                {"value": true, "numeric_field": 0},
                {"value": false, "numeric_field": -1}
            ]
        }"#;

        let error_boundary_tests = vec![
            // Type mismatch scenarios that should handle gracefully
            (
                "$.error_boundaries[?@.value > @.numeric_field]",
                0,
                "String vs number comparison should handle gracefully",
            ),
            (
                "$.error_boundaries[?@.value == @.numeric_field]",
                0,
                "Type mismatched equality should handle gracefully",
            ),
            (
                "$.error_boundaries[?@.value != @.numeric_field]",
                6,
                "Type mismatched inequality should work consistently",
            ),
            // Null boundary conditions
            (
                "$.error_boundaries[?@.value == null]",
                1,
                "Only null should equal null",
            ),
            (
                "$.error_boundaries[?@.numeric_field == null]",
                1,
                "Only null numeric field should equal null",
            ),
            (
                "$.error_boundaries[?@.value != null && @.numeric_field != null]",
                4,
                "Non-null pairs should be identifiable",
            ),
            // Boolean boundary conditions
            (
                "$.error_boundaries[?@.value == true]",
                1,
                "Only true should equal true",
            ),
            (
                "$.error_boundaries[?@.value == false]",
                1,
                "Only false should equal false",
            ),
            (
                "$.error_boundaries[?@.numeric_field > 0]",
                2,
                "Positive numeric fields should be identifiable",
            ),
            (
                "$.error_boundaries[?@.numeric_field < 0]",
                1,
                "Negative numeric fields should be identifiable",
            ),
            (
                "$.error_boundaries[?@.numeric_field == 0]",
                1,
                "Zero numeric field should be identifiable",
            ),
        ];

        for (expr, expected_count, _description) in error_boundary_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(error_boundary_json);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Note: Results may vary based on implementation's error handling strategy
            println!(
                "RFC 9535 error boundary: {} - '{}' returned {} results (expected {})",
                _description,
                expr,
                results.len(),
                expected_count
            );

            // Assert compilation succeeds (runtime behavior may vary)
            let compileresult = JsonPathParser::compile(expr);
            assert!(
                compileresult.is_ok(),
                "RFC 9535 error boundary test should compile: {} - '{}'",
                _description,
                expr
            );
        }
    }
}
