//! Performance integration tests
//!
//! Tests that verify function performance with large data sets and stress conditions

use serde_json::json;
use quyc_client::jsonpath::functions::FunctionEvaluator;
use quyc_client::jsonpath::parser::{FilterExpression, FilterValue};

// Mock evaluator for testing
fn mock_evaluator(
    _context: &serde_json::Value,
    _expr: &quyc_client::jsonpath::parser::FilterExpression,
) -> quyc_client::jsonpath::error::JsonPathResult<quyc_client::jsonpath::parser::FilterValue> {
    Ok(quyc_client::jsonpath::parser::FilterValue::String("mock".to_string()))
}

#[test]
fn test_integration_performance_large_data() {
    // Create a large array for performance testing
    let large_array: Vec<i32> = (0..10000).collect();
    let context = json!({"large_data": large_array});

    // Test length function performance
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "length",
        &[FilterExpression::Property {
            path: vec!["large_data".to_string()],
        }],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Integer(10000));

    // Test count function performance
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "count",
        &[FilterExpression::Property {
            path: vec!["large_data".to_string()],
        }],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Integer(10000));
}