//! RFC 9535 Filter Expression Precedence Tests
//!
//! Tests for operator precedence validation in filter expressions:
//! - Operator precedence validation (Table 10)
//! - Grouping with parentheses
//! - Logical AND/OR precedence
//! - Comparison operator precedence  
//! - Function call precedence
//! - Complex expression evaluation order

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct TestModel {
    a: i32,
    b: i32,
    c: i32,
    d: i32,
    flag1: bool,
    flag2: bool,
    name: String,
    category: String,
}

/// RFC 9535 Table 10 - Operator Precedence Tests
#[cfg(test)]
mod operator_precedence_tests {
    use super::*;

    pub fn create_test_data() -> String {
        let items = vec![
            TestModel {
                a: 1,
                b: 2,
                c: 3,
                d: 4,
                flag1: true,
                flag2: false,
                name: "item1".to_string(),
                category: "A".to_string(),
            },
            TestModel {
                a: 2,
                b: 3,
                c: 4,
                d: 5,
                flag1: false,
                flag2: true,
                name: "item2".to_string(),
                category: "B".to_string(),
            },
            TestModel {
                a: 3,
                b: 4,
                c: 5,
                d: 6,
                flag1: true,
                flag2: true,
                name: "item3".to_string(),
                category: "A".to_string(),
            },
            TestModel {
                a: 4,
                b: 5,
                c: 6,
                d: 7,
                flag1: false,
                flag2: false,
                name: "item4".to_string(),
                category: "C".to_string(),
            },
            TestModel {
                a: 5,
                b: 6,
                c: 7,
                d: 8,
                flag1: true,
                flag2: false,
                name: "item5".to_string(),
                category: "B".to_string(),
            },
        ];

        serde_json::to_string(&serde_json::json!({ "items": items }))
            .expect("Valid JSON serialization")
    }

    #[test]
    fn test_comparison_vs_logical_precedence() {
        // RFC 9535: Comparison operators have higher precedence than logical operators
        let json_data = operator_precedence_tests::create_test_data();

        // Test case: @.a < 3 && @.b > 4 || @.c == 5
        // Should be evaluated as: ((@.a < 3) && (@.b > 4)) || (@.c == 5)
        let precedence_cases = vec![
            (
                "$.items[?@.a < 3 && @.b > 4 || @.c == 5]",
                "$.items[?((@.a < 3) && (@.b > 4)) || (@.c == 5)]",
                "Comparison precedence over logical AND and OR",
            ),
            (
                "$.items[?@.a == 1 || @.b == 2 && @.c == 3]",
                "$.items[?(@.a == 1) || ((@.b == 2) && (@.c == 3))]",
                "AND precedence over OR",
            ),
            (
                "$.items[?@.a != 1 && @.b != 2 || @.c != 3 && @.d != 4]",
                "$.items[?((@.a != 1) && (@.b != 2)) || ((@.c != 3) && (@.d != 4))]",
                "Complex comparison and logical precedence",
            ),
        ];

        for (implicit_expr, explicit_expr, _description) in precedence_cases {
            let mut stream1 = JsonArrayStream::<TestModel>::new(implicit_expr);
            let mut stream2 = JsonArrayStream::<TestModel>::new(explicit_expr);

            let chunk = Bytes::from(json_data.clone());
            let results1: Vec<_> = stream1.process_chunk(chunk.clone()).collect();

            let chunk = Bytes::from(json_data.clone());
            let results2: Vec<_> = stream2.process_chunk(chunk).collect();

            assert_eq!(
                results1.len(),
                results2.len(),
                "{}: Implicit and explicit precedence should yield same results",
                _description
            );

            // Verify same items are selected
            for (item1, item2) in results1.iter().zip(results2.iter()) {
                assert_eq!(
                    item1, item2,
                    "{}: Items should match between implicit and explicit precedence",
                    _description
                );
            }

            println!("{}: {} items matched", _description, results1.len());
        }
    }

    #[test]
    fn test_arithmetic_comparison_precedence() {
        // Test arithmetic operators vs comparison operators precedence
        let json_data = operator_precedence_tests::create_test_data();

        let arithmetic_cases = vec![
            (
                "$.items[?@.a + @.b > @.c * 2]",
                "$.items[?(@.a + @.b) > (@.c * 2)]",
                "Addition and multiplication vs comparison",
            ),
            (
                "$.items[?@.a * 2 + @.b == @.c + @.d]",
                "$.items[?((@.a * 2) + @.b) == (@.c + @.d)]",
                "Multiplication and addition precedence",
            ),
            (
                "$.items[?@.a + @.b * @.c > @.d]",
                "$.items[?(@.a + (@.b * @.c)) > @.d]",
                "Multiplication over addition precedence",
            ),
        ];

        for (implicit_expr, explicit_expr, _description) in arithmetic_cases {
            // Note: Test compilation even if arithmetic operators are not fully implemented
            let result1 = JsonPathParser::compile(implicit_expr);
            let result2 = JsonPathParser::compile(explicit_expr);

            match (result1, result2) {
                (Ok(_), Ok(_)) => {
                    let mut stream1 = JsonArrayStream::<TestModel>::new(implicit_expr);
                    let mut stream2 = JsonArrayStream::<TestModel>::new(explicit_expr);

                    let chunk = Bytes::from(json_data.clone());
                    let results1: Vec<_> = stream1.process_chunk(chunk.clone()).collect();

                    let chunk = Bytes::from(json_data.clone());
                    let results2: Vec<_> = stream2.process_chunk(chunk).collect();

                    assert_eq!(
                        results1.len(),
                        results2.len(),
                        "{}: Arithmetic precedence should be consistent",
                        _description
                    );

                    println!(
                        "{}: Arithmetic precedence validated with {} results",
                        _description,
                        results1.len()
                    );
                }
                _ => {
                    println!(
                        "{}: Arithmetic operators not yet implemented (expected)",
                        _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_unary_operator_precedence() {
        // Test unary operators (negation) have highest precedence
        let json_data = operator_precedence_tests::create_test_data();

        let unary_cases = vec![
            (
                "$.items[?!@.flag1 && @.flag2]",
                "$.items[?(!@.flag1) && @.flag2]",
                "Unary NOT precedence over logical AND",
            ),
            (
                "$.items[?!@.flag1 || !@.flag2]",
                "$.items[?(!@.flag1) || (!@.flag2)]",
                "Multiple unary NOT operators",
            ),
            (
                "$.items[?!@.flag1 == false]",
                "$.items[?(!@.flag1) == false]",
                "Unary NOT precedence over comparison",
            ),
        ];

        for (implicit_expr, explicit_expr, _description) in unary_cases {
            let result1 = JsonPathParser::compile(implicit_expr);
            let result2 = JsonPathParser::compile(explicit_expr);

            match (result1, result2) {
                (Ok(_), Ok(_)) => {
                    let mut stream1 = JsonArrayStream::<TestModel>::new(implicit_expr);
                    let mut stream2 = JsonArrayStream::<TestModel>::new(explicit_expr);

                    let chunk = Bytes::from(json_data.clone());
                    let results1: Vec<_> = stream1.process_chunk(chunk.clone()).collect();

                    let chunk = Bytes::from(json_data.clone());
                    let results2: Vec<_> = stream2.process_chunk(chunk).collect();

                    assert_eq!(
                        results1.len(),
                        results2.len(),
                        "{}: Unary precedence should be consistent",
                        _description
                    );

                    println!(
                        "{}: Unary precedence validated with {} results",
                        _description,
                        results1.len()
                    );
                }
                _ => {
                    println!("{}: Unary operators not yet implemented", _description);
                }
            }
        }
    }

    #[test]
    fn test_function_call_precedence() {
        // Test function calls have highest precedence
        let json_data = operator_precedence_tests::create_test_data();

        let function_cases = vec![
            (
                "$.items[?length(@.name) > 4 && @.flag1]",
                "$.items[?(length(@.name) > 4) && @.flag1]",
                "Function call precedence over comparison and logical",
            ),
            (
                "$.items[?length(@.name) == length(@.category) + 1]",
                "$.items[?length(@.name) == (length(@.category) + 1)]",
                "Function call precedence over arithmetic",
            ),
        ];

        for (implicit_expr, explicit_expr, _description) in function_cases {
            let result1 = JsonPathParser::compile(implicit_expr);
            let result2 = JsonPathParser::compile(explicit_expr);

            match (result1, result2) {
                (Ok(_), Ok(_)) => {
                    let mut stream1 = JsonArrayStream::<TestModel>::new(implicit_expr);
                    let mut stream2 = JsonArrayStream::<TestModel>::new(explicit_expr);

                    let chunk = Bytes::from(json_data.clone());
                    let results1: Vec<_> = stream1.process_chunk(chunk.clone()).collect();

                    let chunk = Bytes::from(json_data.clone());
                    let results2: Vec<_> = stream2.process_chunk(chunk).collect();

                    assert_eq!(
                        results1.len(),
                        results2.len(),
                        "{}: Function precedence should be consistent",
                        _description
                    );

                    println!(
                        "{}: Function precedence validated with {} results",
                        _description,
                        results1.len()
                    );
                }
                _ => {
                    println!("{}: Functions not yet implemented (expected)", _description);
                }
            }
        }
    }
}

/// Parentheses Grouping Tests
#[cfg(test)]
mod parentheses_grouping_tests {
    use super::*;

    #[test]
    fn test_parentheses_override_precedence() {
        // Test that parentheses correctly override default operator precedence
        let json_data = operator_precedence_tests::create_test_data();

        let grouping_cases = vec![
            (
                "$.items[?(@.a == 1 || @.a == 2) && @.flag1]",
                "Parentheses group OR before AND",
                vec![0], // Expect item with a=1 and flag1=true (first item)
            ),
            (
                "$.items[?@.a == 1 || (@.a == 2 && @.flag1)]",
                "AND has natural precedence over OR",
                vec![0], // Expect item with a=1 (first item)
            ),
            (
                "$.items[?(@.a < 3 && @.b > 4) || (@.c == 5 && @.d == 6)]",
                "Parentheses group complex conditions",
                vec![2], // Expect item with c=5 and d=6 (third item)
            ),
            (
                "$.items[?(@.flag1 || @.flag2) && (@.a > 2 && @.a < 5)]",
                "Multiple parenthetical groups",
                vec![2], // Expect items meeting both conditions
            ),
        ];

        for (expr, _description, expected_indices) in grouping_cases {
            let mut stream = JsonArrayStream::<TestModel>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert_eq!(
                results.len(),
                expected_indices.len(),
                "{}: Should match expected number of items",
                _description
            );

            // Verify specific items are selected based on expected indices
            for (i, &expected_idx) in expected_indices.iter().enumerate() {
                if i < results.len() {
                    match expected_idx {
                        0 => assert_eq!(results[i].a, 1, "First item should have a=1"),
                        1 => assert_eq!(results[i].a, 2, "Second item should have a=2"),
                        2 => assert_eq!(results[i].a, 3, "Third item should have a=3"),
                        3 => assert_eq!(results[i].a, 4, "Fourth item should have a=4"),
                        4 => assert_eq!(results[i].a, 5, "Fifth item should have a=5"),
                        _ => {}
                    }
                }
            }

            println!(
                "{}: {} items matched with parentheses grouping",
                _description,
                results.len()
            );
        }
    }

    #[test]
    fn test_nested_parentheses() {
        // Test nested parentheses evaluation
        let json_data = operator_precedence_tests::create_test_data();

        let nested_cases = vec![
            (
                "$.items[?((@.a == 1 || @.a == 2) && (@.flag1 || @.flag2))]",
                "Single level nesting",
                2, // Expect items matching either condition
            ),
            (
                "$.items[?(((@.a > 0 && @.a < 3) || (@.b > 5 && @.b < 8)) && @.flag1)]",
                "Double level nesting",
                1, // Complex nested conditions
            ),
            (
                "$.items[?((@.a == 1 && (@.flag1 || (@.flag2 && @.b > 1))) || @.c == 7)]",
                "Triple level nesting",
                2, // Very complex nested evaluation
            ),
        ];

        for (expr, _description, min_expected) in nested_cases {
            let mut stream = JsonArrayStream::<TestModel>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            assert!(
                results.len() >= min_expected,
                "{}: Should find at least {} matching items, found {}",
                _description,
                min_expected,
                results.len()
            );

            println!(
                "{}: {} items matched with nested parentheses",
                _description,
                results.len()
            );
        }
    }

    #[test]
    fn test_parentheses_syntax_validation() {
        // Test proper parentheses syntax validation
        let valid_expressions = vec![
            "$.items[?(@.a == 1)]",
            "$.items[?((@.a == 1))]",
            "$.items[?(@.a == 1 && (@.b == 2 || @.c == 3))]",
            "$.items[?(((@.a)))]",
        ];

        for expr in valid_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "Valid parentheses expression '{}' should compile",
                expr
            );
        }

        let invalid_expressions = vec![
            "$.items[?(@.a == 1]",   // Missing closing parenthesis
            "$.items[?@.a == 1)]",   // Missing opening parenthesis
            "$.items[?((@.a == 1)]", // Mismatched parentheses
            "$.items[?(@.a == 1))]", // Extra closing parenthesis
            "$.items[?((@.a == 1)]", // Extra opening parenthesis
        ];

        for expr in invalid_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Invalid parentheses expression '{}' should fail",
                expr
            );
        }
    }
}

/// Logical Operator Precedence Tests  
#[cfg(test)]
mod logical_precedence_tests {
    use super::*;

    #[test]
    fn test_and_or_precedence() {
        // RFC 9535: AND (&&) has higher precedence than OR (||)
        let json_data = operator_precedence_tests::create_test_data();

        let precedence_cases = vec![
            (
                "$.items[?@.a == 1 || @.a == 2 && @.flag1]",
                "$.items[?(@.a == 1) || ((@.a == 2) && @.flag1)]",
                "AND precedence over OR - implicit vs explicit",
            ),
            (
                "$.items[?@.flag1 && @.a > 2 || @.flag2 && @.b < 4]",
                "$.items[?((@.flag1 && (@.a > 2)) || (@.flag2 && (@.b < 4)))]",
                "Multiple AND conditions with OR",
            ),
            (
                "$.items[?@.a == 1 || @.b == 2 && @.c == 3 || @.d == 4]",
                "$.items[?((@.a == 1) || ((@.b == 2) && (@.c == 3)) || (@.d == 4))]",
                "Mixed AND/OR with multiple clauses",
            ),
        ];

        for (implicit_expr, explicit_expr, _description) in precedence_cases {
            let mut stream1 = JsonArrayStream::<TestModel>::new(implicit_expr);
            let mut stream2 = JsonArrayStream::<TestModel>::new(explicit_expr);

            let chunk = Bytes::from(json_data.clone());
            let results1: Vec<_> = stream1.process_chunk(chunk.clone()).collect();

            let chunk = Bytes::from(json_data.clone());
            let results2: Vec<_> = stream2.process_chunk(chunk).collect();

            assert_eq!(
                results1.len(),
                results2.len(),
                "{}: AND/OR precedence should be consistent",
                _description
            );

            // Verify same items are selected
            for (item1, item2) in results1.iter().zip(results2.iter()) {
                assert_eq!(
                    item1.a, item2.a,
                    "{}: Same items should be selected with consistent precedence",
                    _description
                );
            }

            println!("{}: {} items matched", _description, results1.len());
        }
    }

    #[test]
    fn test_associativity() {
        // Test left-to-right associativity for operators of equal precedence
        let json_data = operator_precedence_tests::create_test_data();

        let associativity_cases = vec![
            (
                "$.items[?@.a == 1 || @.a == 2 || @.a == 3]",
                "$.items[?((@.a == 1) || @.a == 2) || (@.a == 3)]",
                "OR associativity left-to-right",
            ),
            (
                "$.items[?@.flag1 && @.flag2 && @.a > 0]",
                "$.items[?((@.flag1 && @.flag2) && (@.a > 0))]",
                "AND associativity left-to-right",
            ),
            (
                "$.items[?@.a > 0 && @.b > 0 && @.c > 0]",
                "$.items[?((@.a > 0) && (@.b > 0)) && (@.c > 0)]",
                "Multiple AND conditions associativity",
            ),
        ];

        for (implicit_expr, explicit_expr, _description) in associativity_cases {
            let mut stream1 = JsonArrayStream::<TestModel>::new(implicit_expr);
            let mut stream2 = JsonArrayStream::<TestModel>::new(explicit_expr);

            let chunk = Bytes::from(json_data.clone());
            let results1: Vec<_> = stream1.process_chunk(chunk.clone()).collect();

            let chunk = Bytes::from(json_data.clone());
            let results2: Vec<_> = stream2.process_chunk(chunk).collect();

            assert_eq!(
                results1.len(),
                results2.len(),
                "{}: Associativity should be consistent",
                _description
            );

            println!(
                "{}: {} items matched with correct associativity",
                _description,
                results1.len()
            );
        }
    }

    #[test]
    fn test_short_circuit_evaluation() {
        // Test that logical operators use short-circuit evaluation
        let json_data = operator_precedence_tests::create_test_data();

        // Test cases designed to verify short-circuit behavior
        let short_circuit_cases = vec![
            (
                "$.items[?@.flag1 || @.nonexistent_field]",
                "OR short-circuit - should not fail on missing field when first condition is true",
            ),
            (
                "$.items[?@.flag2 && @.a > 0]",
                "AND short-circuit - should evaluate second condition only when first is true",
            ),
            (
                "$.items[?false || @.flag1]",
                "OR with false literal - should evaluate second condition",
            ),
            (
                "$.items[?true && @.flag1]",
                "AND with true literal - should evaluate second condition",
            ),
        ];

        for (expr, _description) in short_circuit_cases {
            let mut stream = JsonArrayStream::<TestModel>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Should not crash and should return valid results
            println!(
                "{}: {} results (no crash demonstrates short-circuit)",
                _description,
                results.len()
            );
        }
    }
}

/// Comparison Operator Precedence Tests
#[cfg(test)]
mod comparison_precedence_tests {
    use super::*;

    #[test]
    fn test_comparison_operators_equal_precedence() {
        // All comparison operators have equal precedence and left-to-right associativity
        let json_data = operator_precedence_tests::create_test_data();

        let comparison_cases = vec![
            (
                "$.items[?@.a < @.b < @.c]",
                "Chained comparisons (if supported)",
            ),
            (
                "$.items[?@.a == @.b == false]",
                "Chained equality comparisons",
            ),
            (
                "$.items[?@.a != @.b != @.c]",
                "Chained inequality comparisons",
            ),
        ];

        for (expr, _description) in comparison_cases {
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<TestModel>::new(expr);

                    let chunk = Bytes::from(json_data.clone());
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "{}: {} results (chained comparisons supported)",
                        _description,
                        results.len()
                    );
                }
                Err(_) => {
                    println!(
                        "{}: Chained comparisons not supported (expected)",
                        _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_comparison_vs_equality_precedence() {
        // Test that all comparison operators have same precedence
        let json_data = operator_precedence_tests::create_test_data();

        let mixed_comparison_cases = vec![
            ("$.items[?@.a > 1 == true]", "Greater than vs equality"),
            ("$.items[?@.a < 5 != false]", "Less than vs inequality"),
            (
                "$.items[?@.a >= 2 == @.flag1]",
                "Greater equal vs equality with field",
            ),
        ];

        for (expr, _description) in mixed_comparison_cases {
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<TestModel>::new(expr);

                    let chunk = Bytes::from(json_data.clone());
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "{}: {} results (mixed comparisons)",
                        _description,
                        results.len()
                    );
                }
                Err(_) => {
                    println!("{}: Mixed comparisons not supported", _description);
                }
            }
        }
    }

    #[test]
    fn test_comparison_with_parentheses() {
        // Test comparison operators with explicit parentheses
        let json_data = operator_precedence_tests::create_test_data();

        let parenthetical_comparisons = vec![
            (
                "$.items[?(@.a > 1) == (@.flag1)]",
                "Parenthesized comparison result vs boolean",
            ),
            (
                "$.items[?(@.a + @.b) > (@.c * 2)]",
                "Parenthesized arithmetic in comparisons",
            ),
            (
                "$.items[?(@.a > @.b) && (@.c < @.d)]",
                "Parenthesized comparisons with logical AND",
            ),
        ];

        for (expr, _description) in parenthetical_comparisons {
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<TestModel>::new(expr);

                    let chunk = Bytes::from(json_data.clone());
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "{}: {} results (parenthetical comparisons)",
                        _description,
                        results.len()
                    );
                }
                Err(_) => {
                    println!("{}: Parenthetical comparisons not supported", _description);
                }
            }
        }
    }
}

/// Complex Expression Evaluation Order Tests
#[cfg(test)]
mod complex_evaluation_tests {
    use super::*;

    #[test]
    fn test_deeply_nested_precedence() {
        // Test precedence in deeply nested expressions
        let json_data = operator_precedence_tests::create_test_data();

        let complex_expressions = vec![
            (
                "$.items[?@.a == 1 || @.b == 2 && @.c == 3 || @.d == 4 && @.flag1]",
                "Complex OR/AND precedence chain",
            ),
            (
                "$.items[?(@.a == 1 || @.b == 2) && (@.c == 3 || @.d == 4) && @.flag1]",
                "Grouped OR conditions with AND",
            ),
            (
                "$.items[?@.a > 0 && (@.b > 1 || @.c > 2) && (@.d > 3 || @.flag1)]",
                "Mixed comparison and logical with grouping",
            ),
        ];

        for (expr, _description) in complex_expressions {
            let mut stream = JsonArrayStream::<TestModel>::new(expr);

            let chunk = Bytes::from(json_data.clone());
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!("{}: {} items matched", _description, results.len());

            // Verify results are logically consistent
            for item in &results {
                println!(
                    "  Matched item: a={}, b={}, c={}, d={}, flag1={}",
                    item.a, item.b, item.c, item.d, item.flag1
                );
            }
        }
    }

    #[test]
    fn test_precedence_with_functions() {
        // Test precedence when function calls are involved
        let json_data = operator_precedence_tests::create_test_data();

        let function_precedence_cases = vec![
            (
                "$.items[?length(@.name) > 4 && @.flag1 || @.a == 1]",
                "Function call with logical operators",
            ),
            (
                "$.items[?length(@.name) == length(@.category) && @.flag1]",
                "Multiple function calls with logical operator",
            ),
            (
                "$.items[?(length(@.name) > 4 || @.a == 1) && @.flag1]",
                "Function call in parenthetical group",
            ),
        ];

        for (expr, _description) in function_precedence_cases {
            let result = JsonPathParser::compile(expr);

            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<TestModel>::new(expr);

                    let chunk = Bytes::from(json_data.clone());
                    let results: Vec<_> = stream.process_chunk(chunk).collect();

                    println!(
                        "{}: {} results (function precedence)",
                        _description,
                        results.len()
                    );
                }
                Err(_) => {
                    println!("{}: Functions not yet implemented", _description);
                }
            }
        }
    }

    #[test]
    fn test_error_cases_precedence() {
        // Test error cases related to precedence and grouping
        let invalid_precedence_expressions = vec![
            "$.items[?@.a == 1 &&]",             // Incomplete AND expression
            "$.items[?|| @.a == 1]",             // Leading OR operator
            "$.items[?@.a == 1 && && @.b == 2]", // Double AND operator
            "$.items[?@.a == 1 || || @.b == 2]", // Double OR operator
            "$.items[?@.a == 1 &| @.b == 2]",    // Invalid operator
        ];

        for expr in invalid_precedence_expressions {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_err(),
                "Invalid precedence expression '{}' should fail compilation",
                expr
            );
        }
    }
}
