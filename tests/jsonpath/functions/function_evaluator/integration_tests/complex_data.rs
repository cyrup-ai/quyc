//! Complex data structure integration tests
//!
//! Tests that verify function behavior with nested objects, arrays, and complex JSON structures

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
fn test_integration_complex_data_structures() {
    let context = json!({
        "users": [
            {"name": "Alice", "age": 30, "active": true},
            {"name": "Bob", "age": 25, "active": false},
            {"name": "Charlie", "age": 35, "active": true}
        ],
        "metadata": {
            "total": 3,
            "description": "User database"
        }
    });

    // Test length on nested array
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "length",
        &[FilterExpression::Property {
            path: vec!["users".to_string()],
        }],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Integer(3));

    // Test count on nested object
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "count",
        &[FilterExpression::Property {
            path: vec!["metadata".to_string()],
        }],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Integer(2));

    // Test value on nested property
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "value",
        &[FilterExpression::Property {
            path: vec!["metadata".to_string(), "description".to_string()],
        }],
        &mock_evaluator,
    );
    assert_eq!(
        result.unwrap(),
        FilterValue::String("User database".to_string())
    );
}