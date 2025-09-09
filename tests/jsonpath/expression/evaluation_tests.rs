//! Tests for expression evaluation implementation
//! 
//! Extracted from src/jsonpath/expression/evaluation.rs
//! Tests evaluation logic for JSONPath expressions

use quyc_client::jsonpath::expression::evaluation::JsonPathExpression;
use quyc_client::jsonpath::parser::JsonSelector;

fn create_root_expression() -> JsonPathExpression {
    JsonPathExpression::new(vec![JsonSelector::Root], "$".to_string(), false)
}

fn create_child_expression() -> JsonPathExpression {
    JsonPathExpression::new(
        vec![
            JsonSelector::Root,
            JsonSelector::Child {
                key: "test".to_string(),
            },
        ],
        "$.test".to_string(),
        false,
    )
}

fn create_recursive_expression() -> JsonPathExpression {
    JsonPathExpression::new(
        vec![
            JsonSelector::Root,
            JsonSelector::RecursiveDescent,
            JsonSelector::Wildcard,
        ],
        "$..*".to_string(),
        false,
    )
}

#[test]
fn test_matches_at_depth_root() {
    let expr = create_root_expression();

    assert!(expr.matches_at_depth(0));
    assert!(!expr.matches_at_depth(1));
}

#[test]
fn test_matches_at_depth_child() {
    let expr = create_child_expression();

    assert!(expr.matches_at_depth(0)); // Root matches
    assert!(expr.matches_at_depth(1)); // Child matches
    assert!(!expr.matches_at_depth(2)); // Too deep
}

#[test]
fn test_matches_at_depth_recursive() {
    let expr = create_recursive_expression();

    assert!(expr.matches_at_depth(0)); // Root matches
    assert!(expr.matches_at_depth(1)); // Recursive descent matches
    assert!(expr.matches_at_depth(5)); // Deep recursive descent matches
}

#[test]
fn test_evaluate_single_selector_root() {
    let expr = create_root_expression();

    assert!(expr.evaluate_single_selector_at_depth(&JsonSelector::Root, 0));
    assert!(!expr.evaluate_single_selector_at_depth(&JsonSelector::Root, 1));
}

#[test]
fn test_evaluate_single_selector_recursive_descent() {
    let expr = create_recursive_expression();

    assert!(expr.evaluate_single_selector_at_depth(&JsonSelector::RecursiveDescent, 0));
    assert!(expr.evaluate_single_selector_at_depth(&JsonSelector::RecursiveDescent, 5));
    assert!(expr.evaluate_single_selector_at_depth(&JsonSelector::RecursiveDescent, 10));
}

#[test]
fn test_evaluate_single_selector_wildcard() {
    let expr = create_recursive_expression();

    assert!(!expr.evaluate_single_selector_at_depth(&JsonSelector::Wildcard, 0));
    assert!(expr.evaluate_single_selector_at_depth(&JsonSelector::Wildcard, 1));
    assert!(expr.evaluate_single_selector_at_depth(&JsonSelector::Wildcard, 5));
}

#[test]
fn test_depth_limit_protection() {
    let expr = create_recursive_expression();

    // Test that very deep recursion is handled gracefully
    assert!(expr.matches_at_depth(19)); // Should work
    // Depth 20+ is limited in recursive descent evaluation
}