use quyc_client::jsonpath::expression::*;

fn create_simple_expression() -> JsonPathExpression {
    JsonPathExpression::new(
        vec![JsonSelector::Root, JsonSelector::Wildcard],
        "$.*".to_string(),
        false,
    )
}

fn create_complex_expression() -> JsonPathExpression {
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
fn test_simple_complexity_metrics() {
    let expr = create_simple_expression();
    let metrics = expr.complexity_metrics();

    assert_eq!(metrics.total_selector_count, 2);
    assert_eq!(metrics.recursive_descent_depth, 0);
    assert_eq!(metrics.filter_complexity_sum, 0);
    assert_eq!(metrics.union_selector_count, 0);
}

#[test]
fn test_complex_complexity_metrics() {
    let expr = create_complex_expression();
    let metrics = expr.complexity_metrics();

    assert_eq!(metrics.total_selector_count, 3);
    assert_eq!(metrics.recursive_descent_depth, 1);
}

#[test]
fn test_complexity_score_simple() {
    let expr = create_simple_expression();
    let score = expr.complexity_score();

    // Simple expression should have low complexity
    assert!(score < 20);
}

#[test]
fn test_complexity_score_complex() {
    let expr = create_complex_expression();
    let score = expr.complexity_score();

    // Complex expression with recursive descent should have higher complexity
    assert!(score > 10);
}

#[test]
fn test_complexity_score_overflow_protection() {
    // Create expression with many recursive descents to test overflow protection
    let selectors = vec![JsonSelector::RecursiveDescent; 15];
    let expr = JsonPathExpression::new(selectors, "$..".repeat(15), false);

    // Should not panic due to overflow
    let score = expr.complexity_score();
    assert!(score > 0);
}