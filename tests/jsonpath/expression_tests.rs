use quyc_client::jsonpath::expression::JsonPathExpression;
use quyc_client::jsonpath::ast::JsonSelector;

#[test]
fn test_module_integration() {
    // Test that all modules are properly integrated
    let expr = JsonPathExpression::new(
        vec![JsonSelector::Root, JsonSelector::Wildcard],
        "$.*".to_string(),
        false,
    );

    // Test core functionality
    assert_eq!(expr.original(), "$.*");
    assert_eq!(expr.selectors().len(), 2);

    // Test complexity analysis
    let score = expr.complexity_score();
    assert!(score > 0);

    // Test evaluation
    assert!(expr.matches_at_depth(0));
}

#[test]
fn test_comprehensive_functionality() {
    let expr = JsonPathExpression::new(
        vec![
            JsonSelector::Root,
            JsonSelector::RecursiveDescent,
            JsonSelector::Child {
                key: "test".to_string(),
            },
        ],
        "$..test".to_string(),
        true,
    );

    // Test all module capabilities
    assert!(expr.is_array_stream());
    assert!(expr.has_recursive_descent());
    assert_eq!(expr.recursive_descent_start(), Some(1));

    let metrics = expr.complexity_metrics();
    assert_eq!(metrics.recursive_descent_depth, 1);

    assert!(expr.matches_at_depth(0));
    assert!(expr.matches_at_depth(2));
}