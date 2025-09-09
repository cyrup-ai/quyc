//! RFC 9535 I-JSON Integer Range Boundary Tests (Section 2.1)
//!
//! Tests for RFC 9535 Section 2.1 validity requirement #1:
//! "Integer numbers in the JSONPath query that are relevant to the JSONPath processing
//! (e.g., index values and steps) MUST be within the range of exact integer values
//! defined in Internet JSON (I-JSON) (see Section 2.2 of [RFC7493]), namely within
//! the interval [-(2^53)+1, (2^53)-1]."
//!
//! This test suite validates:
//! - Array index boundary validation
//! - Slice parameter boundary validation  
//! - Comparison operand boundary validation
//! - Function argument boundary validation
//! - Edge cases at I-JSON _boundaries

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathError, JsonPathParser};

/// I-JSON safe integer constants from RFC 7493
const MAX_SAFE_INTEGER: i64 = 9007199254740991; // (2^53) - 1
const MIN_SAFE_INTEGER: i64 = -9007199254740991; // -((2^53) - 1)

/// RFC 9535 Section 2.1 - I-JSON Integer Range Boundary Tests
#[cfg(test)]
mod ijson_boundary_tests {
    use super::*;

    #[test]
    fn test_array_index_ijson_boundaries() {
        // RFC 9535: Array indices MUST be within I-JSON range
        let boundary_tests = vec![
            // Valid I-JSON range _boundaries
            (
                format!("$[{}]", MAX_SAFE_INTEGER),
                true,
                "Max safe integer index",
            ),
            (
                format!("$[{}]", MIN_SAFE_INTEGER),
                true,
                "Min safe integer index",
            ),
            (
                format!("$[{}]", MAX_SAFE_INTEGER - 1),
                true,
                "Just below max safe",
            ),
            (
                format!("$[{}]", MIN_SAFE_INTEGER + 1),
                true,
                "Just above min safe",
            ),
            // Beyond I-JSON _boundaries - MUST be rejected per RFC
            (
                format!("$[{}]", MAX_SAFE_INTEGER + 1),
                false,
                "Beyond max safe integer",
            ),
            (
                format!("$[{}]", MIN_SAFE_INTEGER - 1),
                false,
                "Beyond min safe integer",
            ),
            (format!("$[{}]", i64::MAX), false, "Max i64 value"),
            (format!("$[{}]", i64::MIN), false, "Min i64 value"),
            // Scientific notation beyond I-JSON (should be rejected)
            (
                "$[1e16]".to_string(),
                false,
                "Scientific notation beyond safe range",
            ),
            (
                "$[9.007199254740992e15]".to_string(),
                false,
                "Decimal beyond safe range",
            ),
        ];

        for (expr, _should_be_valid, _description) in boundary_tests {
            let result = JsonPathParser::compile(&expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: I-JSON boundary test should pass: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: I-JSON boundary violation MUST be rejected: {} ({})",
                    expr,
                    _description
                );

                // Verify error is related to I-JSON range violation
                if let Err(JsonPathError::InvalidExpression { reason, .. }) = result {
                    assert!(
                        reason.contains("range")
                            || reason.contains("I-JSON")
                            || reason.contains("boundary"),
                        "Error should mention I-JSON range violation: {}",
                        reason
                    );
                }
            }
        }
    }

    #[test]
    fn test_array_slice_ijson_boundaries() {
        // RFC 9535: Array slice parameters MUST be within I-JSON range
        let slice_boundary_tests = vec![
            // Valid I-JSON slice _boundaries
            (
                format!("$[{}:{}]", MIN_SAFE_INTEGER, MAX_SAFE_INTEGER),
                true,
                "Full I-JSON range slice",
            ),
            (format!("$[{}:]", MAX_SAFE_INTEGER), true, "Max safe start"),
            (format!("$[:{}]", MAX_SAFE_INTEGER), true, "Max safe end"),
            (format!("$[::{}]", MAX_SAFE_INTEGER), true, "Max safe step"),
            (
                format!("$[{}:{}:{}]", MIN_SAFE_INTEGER, MAX_SAFE_INTEGER, 1),
                true,
                "Full valid slice",
            ),
            // Beyond I-JSON _boundaries in slice parameters
            (
                format!("$[{}:]", MAX_SAFE_INTEGER + 1),
                false,
                "Start beyond max safe",
            ),
            (
                format!("$[:{}]", MAX_SAFE_INTEGER + 1),
                false,
                "End beyond max safe",
            ),
            (
                format!("$[::{}]", MAX_SAFE_INTEGER + 1),
                false,
                "Step beyond max safe",
            ),
            (
                format!("$[{}:]", MIN_SAFE_INTEGER - 1),
                false,
                "Start beyond min safe",
            ),
            (
                format!("$[:{}]", MIN_SAFE_INTEGER - 1),
                false,
                "End beyond min safe",
            ),
            (
                format!("$[::{}]", MIN_SAFE_INTEGER - 1),
                false,
                "Step beyond min safe",
            ),
            // Multiple boundary violations
            (
                format!("$[{}:{}]", MAX_SAFE_INTEGER + 1, MAX_SAFE_INTEGER + 2),
                false,
                "Multiple violations",
            ),
        ];

        for (expr, _should_be_valid, _description) in slice_boundary_tests {
            let result = JsonPathParser::compile(&expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: I-JSON slice boundary test should pass: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: I-JSON slice boundary violation MUST be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_comparison_operand_ijson_boundaries() {
        // RFC 9535: Comparison operands in filters MUST be within I-JSON range
        let comparison_tests = vec![
            // Valid I-JSON comparisons
            (
                format!("$[?@.value == {}]", MAX_SAFE_INTEGER),
                true,
                "Max safe comparison",
            ),
            (
                format!("$[?@.value == {}]", MIN_SAFE_INTEGER),
                true,
                "Min safe comparison",
            ),
            (
                format!("$[?@.value > {}]", MAX_SAFE_INTEGER - 1),
                true,
                "Greater than near max",
            ),
            (
                format!("$[?@.value < {}]", MIN_SAFE_INTEGER + 1),
                true,
                "Less than near min",
            ),
            // Beyond I-JSON _boundaries in comparisons
            (
                format!("$[?@.value == {}]", MAX_SAFE_INTEGER + 1),
                false,
                "Comparison beyond max safe",
            ),
            (
                format!("$[?@.value == {}]", MIN_SAFE_INTEGER - 1),
                false,
                "Comparison beyond min safe",
            ),
            (
                format!("$[?@.value > {}]", MAX_SAFE_INTEGER + 1),
                false,
                "Greater than beyond max",
            ),
            (
                format!("$[?@.value < {}]", MIN_SAFE_INTEGER - 1),
                false,
                "Less than beyond min",
            ),
            // Complex logical expressions with boundary violations
            (
                format!(
                    "$[?@.value > {} && @.other < {}]",
                    MAX_SAFE_INTEGER + 1,
                    100
                ),
                false,
                "Logical AND with violation",
            ),
            (
                format!(
                    "$[?@.value < {} || @.other > {}]",
                    MIN_SAFE_INTEGER - 1,
                    100
                ),
                false,
                "Logical OR with violation",
            ),
        ];

        for (expr, _should_be_valid, _description) in comparison_tests {
            let result = JsonPathParser::compile(&expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: I-JSON comparison boundary test should pass: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: I-JSON comparison boundary violation MUST be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_function_argument_ijson_boundaries() {
        // RFC 9535: Function arguments MUST respect I-JSON _boundaries
        let function_tests = vec![
            // Valid I-JSON function arguments (these test syntax, not execution)
            (
                format!("$[?length(@.array) == {}]", MAX_SAFE_INTEGER),
                true,
                "Function result comparison at max",
            ),
            (
                format!("$[?count(@.items) > {}]", MAX_SAFE_INTEGER - 1),
                true,
                "Function count near max",
            ),
            // Beyond I-JSON _boundaries in function contexts
            (
                format!("$[?length(@.array) == {}]", MAX_SAFE_INTEGER + 1),
                false,
                "Function comparison beyond max",
            ),
            (
                format!("$[?count(@.items) < {}]", MIN_SAFE_INTEGER - 1),
                false,
                "Function comparison beyond min",
            ),
        ];

        for (expr, _should_be_valid, _description) in function_tests {
            let result = JsonPathParser::compile(&expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: I-JSON function boundary test should pass: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: I-JSON function boundary violation MUST be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_edge_cases_at_ijson_boundaries() {
        // RFC 9535: Test edge cases exactly at I-JSON _boundaries
        let edge_cases = vec![
            // Exactly at _boundaries (should be valid)
            (
                format!("$[{}]", MAX_SAFE_INTEGER),
                true,
                "Exactly max safe integer",
            ),
            (
                format!("$[{}]", MIN_SAFE_INTEGER),
                true,
                "Exactly min safe integer",
            ),
            // One beyond _boundaries (should be invalid)
            (
                format!("$[{}]", MAX_SAFE_INTEGER as u64 + 1),
                false,
                "One beyond max safe",
            ),
            // Floating point representations of boundary values
            (
                "$[9007199254740991.0]".to_string(),
                false,
                "Floating point max safe",
            ),
            (
                "$[-9007199254740991.0]".to_string(),
                false,
                "Floating point min safe",
            ),
            // String representations (should be valid if in I-JSON range)
            (
                "$['9007199254740991']".to_string(),
                true,
                "String representation of max safe",
            ),
            (
                "$['-9007199254740991']".to_string(),
                true,
                "String representation of min safe",
            ),
        ];

        for (expr, _should_be_valid, _description) in edge_cases {
            let result = JsonPathParser::compile(&expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: I-JSON edge case should pass: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: I-JSON edge case violation MUST be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_execution_with_ijson_boundary_values() {
        // RFC 9535: Test actual execution with I-JSON boundary values
        let json_data = format!(
            r#"{{
            "largeArray": {},
            "items": [
                {{"id": {}, "value": "max"}},
                {{"id": {}, "value": "min"}},
                {{"id": 0, "value": "zero"}}
            ]
        }}"#,
            serde_json::json!((0..10).collect::<Vec<i32>>()),
            MAX_SAFE_INTEGER,
            MIN_SAFE_INTEGER
        );

        let execution_tests = vec![
            // Valid executions with I-JSON boundary values
            (
                format!("$.items[?@.id == {}]", MAX_SAFE_INTEGER),
                1,
                "Filter by max safe ID",
            ),
            (
                format!("$.items[?@.id == {}]", MIN_SAFE_INTEGER),
                1,
                "Filter by min safe ID",
            ),
            (
                format!("$.items[?@.id > {}]", MIN_SAFE_INTEGER),
                1,
                "Greater than min safe",
            ),
            (
                format!("$.items[?@.id < {}]", MAX_SAFE_INTEGER),
                1,
                "Less than max safe",
            ),
        ];

        for (expr, expected_count, _description) in execution_tests {
            // First verify the expression compiles
            let compileresult = JsonPathParser::compile(&expr);
            assert!(
                compileresult.is_ok(),
                "RFC 9535: I-JSON boundary execution should compile: {} ({})",
                expr,
                _description
            );

            // Then test execution
            let mut stream = JsonArrayStream::<serde_json::Value>::new(&expr);
            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_count,
                "RFC 9535: I-JSON boundary execution should return {} results: {} ({})",
                expected_count,
                expr,
                _description
            );
        }
    }
}

/// Precision Loss Detection Tests
#[cfg(test)]
mod precision_loss_tests {
    use super::*;

    #[test]
    fn test_precision_loss_detection() {
        // RFC 9535: Values beyond I-JSON range may suffer precision loss
        let precision_tests = vec![
            // These should be rejected due to potential precision loss
            (
                "$[9007199254740992]".to_string(),
                false,
                "First integer with precision loss",
            ),
            (
                "$[18014398509481984]".to_string(),
                false,
                "Large integer with precision loss",
            ),
            (
                "$[-9007199254740992]".to_string(),
                false,
                "Negative integer with precision loss",
            ),
            // Scientific notation that exceeds safe range
            (
                "$[9.007199254740992e15]".to_string(),
                false,
                "Scientific notation exceeding safe range",
            ),
            (
                "$[1.8014398509481984e16]".to_string(),
                false,
                "Large scientific notation",
            ),
        ];

        for (expr, _should_be_valid, _description) in precision_tests {
            let result = JsonPathParser::compile(&expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Precision test should pass: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Precision loss scenario MUST be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }

    #[test]
    fn test_boundary_arithmetic() {
        // RFC 9535: Test arithmetic operations near I-JSON _boundaries
        let arithmetic_tests = vec![
            // These are conceptual - testing if expressions parse correctly
            (
                format!("$[?@.value + 1 == {}]", MAX_SAFE_INTEGER),
                true,
                "Addition near max safe",
            ),
            (
                format!("$[?@.value - 1 == {}]", MIN_SAFE_INTEGER),
                true,
                "Subtraction near min safe",
            ),
            // Operations that would exceed _boundaries (if evaluated)
            (
                format!("$[?@.value + 2 == {}]", MAX_SAFE_INTEGER + 2),
                false,
                "Addition exceeding safe range",
            ),
            (
                format!("$[?@.value - 2 == {}]", MIN_SAFE_INTEGER - 2),
                false,
                "Subtraction exceeding safe range",
            ),
        ];

        for (expr, _should_be_valid, _description) in arithmetic_tests {
            let result = JsonPathParser::compile(&expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Arithmetic boundary test should pass: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Arithmetic boundary violation MUST be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}
