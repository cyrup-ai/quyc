//! Tests for expression core implementation
//! 
//! Extracted from src/jsonpath/expression/core.rs
//! Tests core JSONPath expression functionality

use quyc_client::jsonpath::expression::core::JsonPathExpression;
use quyc_client::jsonpath::parser::JsonSelector;

fn create_test_expression() -> JsonPathExpression {
    JsonPathExpression::new(
        vec![JsonSelector::Root, JsonSelector::Wildcard],
        "$..*".to_string(),
        false,
    )
}

#[test]
fn test_expression_creation() {
    let expr = create_test_expression();
    assert_eq!(expr.original(), "$..*");
    assert_eq!(expr.as_string(), "$..*");
    assert!(!expr.is_array_stream());
    assert_eq!(expr.selectors().len(), 2);
}

#[test]
fn test_has_recursive_descent() {
    let expr_with_recursive = JsonPathExpression::new(
        vec![JsonSelector::Root, JsonSelector::RecursiveDescent],
        "$..".to_string(),
        false,
    );
    assert!(expr_with_recursive.has_recursive_descent());

    let expr_without_recursive = JsonPathExpression::new(
        vec![JsonSelector::Root, JsonSelector::Wildcard],
        "$.*".to_string(),
        false,
    );
    assert!(!expr_without_recursive.has_recursive_descent());
}

#[test]
fn test_recursive_descent_start() {
    let expr = JsonPathExpression::new(
        vec![JsonSelector::Root, JsonSelector::RecursiveDescent],
        "$..".to_string(),
        false,
    );
    assert_eq!(expr.recursive_descent_start(), Some(1));
}

#[test]
fn test_root_selector() {
    let expr = create_test_expression();
    match expr.root_selector() {
        Some(JsonSelector::Wildcard) => {
            // Expected
        }
        _ => panic!("Expected Wildcard selector"),
    }
}