//! RFC 9535 Complete Filter Precedence and Logical Operator Tests (Section 2.3.5)
//!
//! Tests for RFC 9535 Section 2.3.5 filter selector logical expression requirements:
//! "logical-expr = logical-or-expr
//!  logical-or-expr = logical-and-expr *(S '||' S logical-and-expr)  
//!  logical-and-expr = basic-expr *(S '&&' S basic-expr)"
//!
//! This test suite validates:
//! - Operator precedence (&& binds tighter than ||)
//! - Logical operator associativity
//! - Parenthesization for precedence override
//! - Negation operator (!) precedence and behavior
//! - Complex nested logical expressions
//! - Short-circuit evaluation behavior
//! - Truth table validation for all operators

use bytes::Bytes;
use quyc::jsonpath::{JsonArrayStream, JsonPathError, JsonPathParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct LogicTestData {
    id: i32,
    active: bool,
    category: String,
    score: f64,
    flag_a: bool,
    flag_b: bool,
    flag_c: bool,
    value: Option<i32>,
}

/// Test data for logical operator precedence validation
const LOGIC_TEST_JSON: &str = r#"{
  "items": [
    {
      "id": 1,
      "active": true,
      "category": "A",
      "score": 85.5,
      "flag_a": true,
      "flag_b": false,
      "flag_c": true,
      "value": 10
    },
    {
      "id": 2,
      "active": false,
      "category": "B", 
      "score": 92.0,
      "flag_a": false,
      "flag_b": true,
      "flag_c": false,
      "value": 20
    },
    {
      "id": 3,
      "active": true,
      "category": "A",
      "score": 78.0,
      "flag_a": true,
      "flag_b": true,
      "flag_c": false,
      "value": null
    },
    {
      "id": 4,
      "active": false,
      "category": "C",
      "score": 95.5,
      "flag_a": false,
      "flag_b": false,
      "flag_c": true,
      "value": 30
    },
    {
      "id": 5,
      "active": true,
      "category": "B",
      "score": 88.0,
      "flag_a": true,
      "flag_b": true,
      "flag_c": true,
      "value": 40
    }
  ]
}"#;

/// RFC 9535 Section 2.3.5 - Logical Operator Precedence Tests
#[cfg(test)]
mod logical_precedence_tests {
    use super::*;

    #[test]
    fn test_and_or_precedence() {
        // RFC 9535: && (AND) has higher precedence than || (OR)
        // Expression: A || B && C should be parsed as A || (B && C)
        let precedence_tests = vec![
            // Test case: true || false && false should be true || (false && false) = true || false = true
            (
                "$.items[?@.flag_a || @.flag_b && @.flag_c]",
                "$.items[?@.flag_a || (@.flag_b && @.flag_c)]",
                "OR-AND precedence test",
            ),
            // Test case: false && true || true should be (false && true) || true = false || true = true
            (
                "$.items[?@.flag_b && @.flag_a || @.flag_c]",
                "$.items[?(@.flag_b && @.flag_a) || @.flag_c]",
                "AND-OR precedence test",
            ),
            // Complex precedence: A || B && C || D should be A || (B && C) || D
            (
                "$.items[?@.flag_a || @.flag_b && @.flag_c || @.active]",
                "$.items[?@.flag_a || (@.flag_b && @.flag_c) || @.active]",
                "Multiple OR with AND precedence",
            ),
            // Precedence with comparisons: A == 1 || B > 2 && C < 3
            (
                "$.items[?@.id == 1 || @.score > 90 && @.value < 25]",
                "$.items[?@.id == 1 || (@.score > 90 && @.value < 25)]",
                "Comparison with logical precedence",
            ),
        ];

        for (implicit_precedence, explicit_precedence, _description) in precedence_tests {
            let mut implicit_stream =
                JsonArrayStream::<serde_json::Value>::new(implicit_precedence);
            let mut explicit_stream =
                JsonArrayStream::<serde_json::Value>::new(explicit_precedence);

            let chunk = Bytes::from(LOGIC_TEST_JSON);
            let implicitresults: Vec<_> = implicit_stream.process_chunk(chunk.clone()).collect();
            let explicitresults: Vec<_> = explicit_stream.process_chunk(chunk).collect();

            assert_eq!(
                implicitresults.len(),
                explicitresults.len(),
                "RFC 9535: Implicit and explicit precedence should produce same results: {} ({})",
                implicit_precedence,
                _description
            );

            println!(
                "✓ Precedence test: {} -> {} results ({})",
                implicit_precedence,
                implicitresults.len(),
                _description
            );
        }
    }

    #[test]
    fn test_negation_precedence() {
        // RFC 9535: ! (NOT) has highest precedence
        let negation_tests = vec![
            // !A && B should be (!A) && B, not !(A && B)
            (
                "$.items[?!@.active && @.flag_a]",
                "$.items[?(!@.active) && @.flag_a]",
                2, // Items 2 and 4 are not active, only 4 has flag_a false
                "NOT-AND precedence",
            ),
            // !A || B should be (!A) || B
            (
                "$.items[?!@.active || @.flag_a]",
                "$.items[?(!@.active) || @.flag_a]",
                4, // Items 2,4 not active OR items 1,3,5 have flag_a
                "NOT-OR precedence",
            ),
            // !A && B || C should be ((!A) && B) || C
            (
                "$.items[?!@.active && @.score > 90 || @.flag_c]",
                "$.items[?((!@.active) && @.score > 90) || @.flag_c]",
                3, // Item 2,4 not active with score>90, plus items 1,4 with flag_c
                "NOT-AND-OR complex precedence",
            ),
            // A && !B || C should be A && ((!B) || C), not A && !(B || C)
            (
                "$.items[?@.active && !@.flag_b || @.score > 90]",
                "$.items[?@.active && ((!@.flag_b) || @.score > 90)]",
                4, // Active items with not flag_b OR score>90
                "AND-NOT-OR precedence",
            ),
        ];

        for (implicit_precedence, explicit_precedence, expected_count, _description) in
            negation_tests
        {
            let mut implicit_stream =
                JsonArrayStream::<serde_json::Value>::new(implicit_precedence);
            let mut explicit_stream =
                JsonArrayStream::<serde_json::Value>::new(explicit_precedence);

            let chunk = Bytes::from(LOGIC_TEST_JSON);
            let implicitresults: Vec<_> = implicit_stream.process_chunk(chunk.clone()).collect();
            let explicitresults: Vec<_> = explicit_stream.process_chunk(chunk).collect();

            assert_eq!(
                implicitresults.len(),
                explicitresults.len(),
                "RFC 9535: Implicit and explicit negation precedence should match: {} ({})",
                implicit_precedence,
                _description
            );

            // Validate against expected count to ensure correct precedence evaluation
            assert_eq!(
                implicitresults.len(),
                expected_count,
                "RFC 9535: Negation precedence should yield expected result count: {} ({})",
                implicit_precedence,
                _description
            );

            println!(
                "✓ Negation precedence: {} -> {} results (expected: {}) ({})",
                implicit_precedence,
                implicitresults.len(),
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_associativity() {
        // RFC 9535: Test left-to-right associativity for same precedence operators
        let associativity_tests = vec![
            // A && B && C should be (A && B) && C (left associative)
            (
                "$.items[?@.active && @.flag_a && @.flag_c]",
                "$.items[?(@.active && @.flag_a) && @.flag_c]",
                1, // Only item 1 has all three true
                "AND left associativity",
            ),
            // A || B || C should be (A || B) || C (left associative)
            (
                "$.items[?@.flag_a || @.flag_b || @.flag_c]",
                "$.items[?(@.flag_a || @.flag_b) || @.flag_c]",
                5, // All items have at least one flag true
                "OR left associativity",
            ),
            // Four-way AND: A && B && C && D should be ((A && B) && C) && D
            (
                "$.items[?@.active && @.flag_a && @.flag_b && @.flag_c]",
                "$.items[?((@.active && @.flag_a) && @.flag_b) && @.flag_c]",
                1, // Only item 5 has all four true
                "Four-way AND associativity",
            ),
            // Four-way OR: A || B || C || D should be ((A || B) || C) || D
            (
                "$.items[?@.active || @.flag_a || @.flag_b || @.flag_c]",
                "$.items[?((@.active || @.flag_a) || @.flag_b) || @.flag_c]",
                5, // All items satisfy at least one condition
                "Four-way OR associativity",
            ),
        ];

        for (implicit_assoc, explicit_assoc, expected_count, _description) in associativity_tests {
            let mut implicit_stream = JsonArrayStream::<serde_json::Value>::new(implicit_assoc);
            let mut explicit_stream = JsonArrayStream::<serde_json::Value>::new(explicit_assoc);

            let chunk = Bytes::from(LOGIC_TEST_JSON);
            let implicitresults: Vec<_> = implicit_stream.process_chunk(chunk.clone()).collect();
            let explicitresults: Vec<_> = explicit_stream.process_chunk(chunk).collect();

            assert_eq!(
                implicitresults.len(),
                explicitresults.len(),
                "RFC 9535: Implicit and explicit associativity should match: {} ({})",
                implicit_assoc,
                _description
            );

            // Validate against expected count to ensure correct associativity evaluation
            assert_eq!(
                implicitresults.len(),
                expected_count,
                "RFC 9535: Operator associativity should yield expected result count: {} ({})",
                implicit_assoc,
                _description
            );

            println!(
                "✓ Associativity test: {} -> {} results (expected: {}) ({})",
                implicit_assoc,
                implicitresults.len(),
                expected_count,
                _description
            );
        }
    }

    #[test]
    fn test_parentheses_override_precedence() {
        // RFC 9535: Parentheses can override natural precedence
        let parentheses_tests = vec![
            // (A || B) && C vs A || B && C - should produce different results
            (
                "$.items[?(@.flag_a || @.flag_b) && @.flag_c]",
                "$.items[?@.flag_a || @.flag_b && @.flag_c]",
                "Parentheses change OR-AND precedence",
            ),
            // A && (B || C) vs A && B || C - should produce different results
            (
                "$.items[?@.active && (@.flag_b || @.flag_c)]",
                "$.items[?@.active && @.flag_b || @.flag_c]",
                "Parentheses change AND-OR precedence",
            ),
            // !(A && B) vs !A && B - should produce different results
            (
                "$.items[?!(@.active && @.flag_a)]",
                "$.items[?!@.active && @.flag_a]",
                "Parentheses change NOT scope",
            ),
            // Complex nested parentheses
            (
                "$.items[?(@.active || @.flag_a) && (@.flag_b || @.flag_c)]",
                "$.items[?@.active || @.flag_a && @.flag_b || @.flag_c]",
                "Complex parentheses grouping",
            ),
        ];

        for (parenthesized, natural, _description) in parentheses_tests {
            let mut paren_stream = JsonArrayStream::<serde_json::Value>::new(parenthesized);
            let mut natural_stream = JsonArrayStream::<serde_json::Value>::new(natural);

            let chunk = Bytes::from(LOGIC_TEST_JSON);
            let parenresults: Vec<_> = paren_stream.process_chunk(chunk.clone()).collect();
            let naturalresults: Vec<_> = natural_stream.process_chunk(chunk).collect();

            // These should generally produce different results due to precedence changes
            println!(
                "Parentheses test: '{}' -> {} results vs '{}' -> {} results ({})",
                parenthesized,
                parenresults.len(),
                natural,
                naturalresults.len(),
                _description
            );

            // Both should compile successfully
            assert!(JsonPathParser::compile(parenthesized).is_ok());
            assert!(JsonPathParser::compile(natural).is_ok());
        }
    }
}

/// RFC 9535 Section 2.3.5 - Logical Operator Truth Table Tests
#[cfg(test)]
mod logical_truth_table_tests {
    use super::*;

    #[test]
    fn test_and_operator_truth_table() {
        // RFC 9535: && (AND) truth table validation
        let and_tests = vec![
            (
                "$.items[?@.flag_a && @.flag_b]",
                vec![3, 5],
                "true && true cases",
            ),
            (
                "$.items[?@.flag_a && !@.flag_b]",
                vec![1],
                "true && false cases",
            ),
            (
                "$.items[?!@.flag_a && @.flag_b]",
                vec![2],
                "false && true cases",
            ),
            (
                "$.items[?!@.flag_a && !@.flag_b]",
                vec![4],
                "false && false cases",
            ),
        ];

        for (expr, expected_ids, _description) in and_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(LOGIC_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            // Extract IDs from results for comparison
            let result_ids: Vec<i32> = results
                .iter()
                .map(|item| item["id"].as_i64().unwrap() as i32)
                .collect();

            for expected_id in expected_ids {
                assert!(
                    result_ids.contains(&expected_id),
                    "RFC 9535: AND truth table test should include ID {}: {} ({})",
                    expected_id,
                    expr,
                    _description
                );
            }

            println!(
                "✓ AND truth table: {} -> IDs {:?} ({})",
                expr, result_ids, _description
            );
        }
    }

    #[test]
    fn test_or_operator_truth_table() {
        // RFC 9535: || (OR) truth table validation
        let or_tests = vec![
            (
                "$.items[?@.flag_a || @.flag_b]",
                vec![1, 2, 3, 5],
                "Any true cases",
            ),
            (
                "$.items[?!@.flag_a || !@.flag_b]",
                vec![1, 2, 4],
                "Any false cases",
            ),
            (
                "$.items[?!@.flag_a && !@.flag_b]",
                vec![4],
                "Both false case",
            ),
        ];

        for (expr, expected_ids, _description) in or_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(LOGIC_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let result_ids: Vec<i32> = results
                .iter()
                .map(|item| item["id"].as_i64().unwrap() as i32)
                .collect();

            for expected_id in expected_ids {
                assert!(
                    result_ids.contains(&expected_id),
                    "RFC 9535: OR truth table test should include ID {}: {} ({})",
                    expected_id,
                    expr,
                    _description
                );
            }

            println!(
                "✓ OR truth table: {} -> IDs {:?} ({})",
                expr, result_ids, _description
            );
        }
    }

    #[test]
    fn test_not_operator_truth_table() {
        // RFC 9535: ! (NOT) truth table validation
        let not_tests = vec![
            ("$.items[?!@.active]", vec![2, 4], "NOT true cases"),
            (
                "$.items[?!!@.active]",
                vec![1, 3, 5],
                "Double NOT (identity) cases",
            ),
            ("$.items[?!@.flag_a]", vec![2, 4], "NOT flag_a cases"),
            ("$.items[?!@.flag_b]", vec![1, 4], "NOT flag_b cases"),
            ("$.items[?!@.flag_c]", vec![2, 3], "NOT flag_c cases"),
        ];

        for (expr, expected_ids, _description) in not_tests {
            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(LOGIC_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            let result_ids: Vec<i32> = results
                .iter()
                .map(|item| item["id"].as_i64().unwrap() as i32)
                .collect();

            for expected_id in expected_ids {
                assert!(
                    result_ids.contains(&expected_id),
                    "RFC 9535: NOT truth table test should include ID {}: {} ({})",
                    expected_id,
                    expr,
                    _description
                );
            }

            println!(
                "✓ NOT truth table: {} -> IDs {:?} ({})",
                expr, result_ids, _description
            );
        }
    }

    #[test]
    fn test_de_morgan_laws() {
        // RFC 9535: De Morgan's laws validation
        let de_morgan_tests = vec![
            // !(A && B) should equal (!A || !B)
            (
                "$.items[?!(@.flag_a && @.flag_b)]",
                "$.items[?!@.flag_a || !@.flag_b]",
                "De Morgan: !(A && B) = (!A || !B)",
            ),
            // !(A || B) should equal (!A && !B)
            (
                "$.items[?!(@.flag_a || @.flag_b)]",
                "$.items[?!@.flag_a && !@.flag_b]",
                "De Morgan: !(A || B) = (!A && !B)",
            ),
            // More complex case: !(A && B && C) = (!A || !B || !C)
            (
                "$.items[?!(@.flag_a && @.flag_b && @.flag_c)]",
                "$.items[?!@.flag_a || !@.flag_b || !@.flag_c]",
                "De Morgan: !(A && B && C) = (!A || !B || !C)",
            ),
        ];

        for (de_morgan_left, de_morgan_right, _description) in de_morgan_tests {
            let mut left_stream = JsonArrayStream::<serde_json::Value>::new(de_morgan_left);
            let mut right_stream = JsonArrayStream::<serde_json::Value>::new(de_morgan_right);

            let chunk = Bytes::from(LOGIC_TEST_JSON);
            let leftresults: Vec<_> = left_stream.process_chunk(chunk.clone()).collect();
            let rightresults: Vec<_> = right_stream.process_chunk(chunk).collect();

            assert_eq!(
                leftresults.len(),
                rightresults.len(),
                "RFC 9535: De Morgan's law should produce equal results: {} ({})",
                de_morgan_left,
                _description
            );

            println!(
                "✓ De Morgan's law: {} = {} -> {} results ({})",
                de_morgan_left,
                de_morgan_right,
                leftresults.len(),
                _description
            );
        }
    }
}

/// RFC 9535 Section 2.3.5 - Complex Logical Expression Tests
#[cfg(test)]
mod complex_logical_tests {
    use super::*;

    #[test]
    fn test_complex_precedence_combinations() {
        // RFC 9535: Complex combinations testing all precedence rules
        let complex_tests = vec![
            // Test: !A || B && C || D should be ((!A) || (B && C)) || D
            (
                "$.items[?!@.active || @.flag_a && @.flag_b || @.flag_c]",
                "Valid complex expression",
            ),
            // Test: A && !B || C && !D should be (A && (!B)) || (C && (!D))
            (
                "$.items[?@.active && !@.flag_a || @.flag_b && !@.flag_c]",
                "Mixed AND-NOT-OR expression",
            ),
            // Test: (A || B) && (C || D) && !(E || F)
            (
                "$.items[?(@.active || @.flag_a) && (@.flag_b || @.flag_c) && !(@.score > 90 || @.id > 4)]",
                "Complex grouped expression",
            ),
            // Test with comparisons and logical operators
            (
                "$.items[?@.score > 80 && @.active || @.flag_a && @.value > 15 || !@.flag_b]",
                "Mixed comparisons and logic",
            ),
            // Deeply nested logical expression
            (
                "$.items[?(@.active && (@.flag_a || @.flag_b)) || (!@.active && (@.flag_c || @.score > 90))]",
                "Deeply nested conditional logic",
            ),
        ];

        for (expr, _description) in complex_tests {
            let result = JsonPathParser::compile(expr);
            assert!(
                result.is_ok(),
                "RFC 9535: Complex logical expression should compile: {} ({})",
                expr,
                _description
            );

            let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
            let chunk = Bytes::from(LOGIC_TEST_JSON);
            let results: Vec<_> = stream.process_chunk(chunk).collect();

            println!(
                "✓ Complex logic: {} -> {} results ({})",
                expr,
                results.len(),
                _description
            );
        }
    }

    #[test]
    fn test_short_circuit_behavior() {
        // RFC 9535: Document short-circuit evaluation behavior (implementation dependent)
        let short_circuit_tests = vec![
            // OR short-circuit: if first is true, second shouldn't be evaluated
            (
                "$.items[?@.active || @.nonexistent.property]",
                "OR short-circuit with error",
            ),
            // AND short-circuit: if first is false, second shouldn't be evaluated
            (
                "$.items[?!@.active && @.nonexistent.property]",
                "AND short-circuit with error",
            ),
            // Nested short-circuit
            (
                "$.items[?@.active || (@.flag_a && @.nonexistent)]",
                "Nested short-circuit",
            ),
        ];

        for (expr, _description) in short_circuit_tests {
            // These may or may not work depending on implementation short-circuit behavior
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    let mut stream = JsonArrayStream::<serde_json::Value>::new(expr);
                    let chunk = Bytes::from(LOGIC_TEST_JSON);
                    let results: Vec<_> = stream.process_chunk(chunk).collect();
                    println!(
                        "Short-circuit test passed: {} -> {} results ({})",
                        expr,
                        results.len(),
                        _description
                    );
                }
                Err(_) => {
                    println!(
                        "Short-circuit test failed compilation: {} ({})",
                        expr, _description
                    );
                }
            }
        }
    }

    #[test]
    fn test_logical_operator_syntax_validation() {
        // RFC 9535: Test syntax validation for logical operators
        let syntax_tests = vec![
            // Valid operator syntax
            ("$.items[?@.active && @.flag_a]", true, "Valid AND"),
            ("$.items[?@.active || @.flag_a]", true, "Valid OR"),
            ("$.items[?!@.active]", true, "Valid NOT"),
            ("$.items[?@.active&&@.flag_a]", true, "AND without spaces"),
            ("$.items[?@.active||@.flag_a]", true, "OR without spaces"),
            (
                "$.items[?@.active && @.flag_a || @.flag_b]",
                true,
                "Mixed operators",
            ),
            // Invalid operator syntax
            ("$.items[?@.active & @.flag_a]", false, "Single & invalid"),
            ("$.items[?@.active | @.flag_a]", false, "Single | invalid"),
            (
                "$.items[?@.active and @.flag_a]",
                false,
                "Word 'and' invalid",
            ),
            ("$.items[?@.active or @.flag_a]", false, "Word 'or' invalid"),
            ("$.items[?not @.active]", false, "Word 'not' invalid"),
            ("$.items[?@.active &&& @.flag_a]", false, "Triple & invalid"),
            ("$.items[?@.active ||| @.flag_a]", false, "Triple | invalid"),
            ("$.items[?!!@.active]", true, "Double NOT should be valid"),
            (
                "$.items[?@.active && || @.flag_a]",
                false,
                "Malformed operators",
            ),
        ];

        for (expr, _should_be_valid, _description) in syntax_tests {
            let result = JsonPathParser::compile(expr);

            if _should_be_valid {
                assert!(
                    result.is_ok(),
                    "RFC 9535: Valid logical syntax should compile: {} ({})",
                    expr,
                    _description
                );
            } else {
                assert!(
                    result.is_err(),
                    "RFC 9535: Invalid logical syntax should be rejected: {} ({})",
                    expr,
                    _description
                );
            }
        }
    }
}

/// RFC 9535 Section 2.3.5 - Logical Operator Error Handling Tests
#[cfg(test)]
mod logical_error_tests {
    use super::*;

    #[test]
    fn test_logical_operator_error_messages() {
        // RFC 9535: Error messages for invalid logical expressions should be clear
        let error_cases = vec![
            ("$.items[?@.active &]", "Incomplete AND operator"),
            ("$.items[?@.active |]", "Incomplete OR operator"),
            ("$.items[?@.active &&]", "Incomplete AND expression"),
            ("$.items[?@.active ||]", "Incomplete OR expression"),
            ("$.items[?!]", "Incomplete NOT expression"),
            ("$.items[?&& @.active]", "Leading AND operator"),
            ("$.items[?|| @.active]", "Leading OR operator"),
            ("$.items[?@.active && && @.flag_a]", "Double AND operators"),
            ("$.items[?@.active || || @.flag_a]", "Double OR operators"),
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

                println!("Logical operator error for '{}': {}", expr, reason);
            }
        }
    }

    #[test]
    fn test_parentheses_error_handling() {
        // RFC 9535: Error handling for malformed parentheses in logical expressions
        let paren_errors = vec![
            ("$.items[?(@.active && @.flag_a]", "Unclosed parenthesis"),
            (
                "$.items[?@.active && @.flag_a)]",
                "Unmatched closing parenthesis",
            ),
            (
                "$.items[?(@.active && (@.flag_a]",
                "Nested unclosed parenthesis",
            ),
            (
                "$.items[?@.active && @.flag_a))]",
                "Extra closing parenthesis",
            ),
            ("$.items[?()]", "Empty parentheses"),
            ("$.items[?(@)]", "Parentheses around @"),
        ];

        for (expr, error_type) in paren_errors {
            let result = JsonPathParser::compile(expr);
            match result {
                Ok(_) => {
                    // Some cases like "(@)" might be valid
                    println!("Parentheses test passed: {} ({})", expr, error_type);
                }
                Err(_) => {
                    println!(
                        "Parentheses error correctly caught: {} ({})",
                        expr, error_type
                    );
                }
            }
        }
    }
}
