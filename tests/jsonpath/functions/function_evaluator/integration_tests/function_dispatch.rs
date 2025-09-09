//! Function dispatch integration tests
//!
//! Tests that verify proper function routing and dispatch for all supported functions

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
fn test_integration_all_functions_dispatch() {
    let context = json!({"text": "hello", "items": [1, 2, 3]});

    // Test length function dispatch
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "length",
        &[FilterExpression::Property {
            path: vec!["text".to_string()],
        }],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Integer(5));

    // Test count function dispatch
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "count",
        &[FilterExpression::Property {
            path: vec!["items".to_string()],
        }],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Integer(3));

    // Test value function dispatch
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "value",
        &[FilterExpression::Property {
            path: vec!["text".to_string()],
        }],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::String("hello".to_string()));
}

#[test]
fn test_integration_regex_functions_dispatch() {
    let context = json!({});

    // Test match function dispatch
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "match",
        &[
            FilterExpression::Literal {
                value: FilterValue::String("hello".to_string()),
            },
            FilterExpression::Literal {
                value: FilterValue::String("^h.*o$".to_string()),
            },
        ],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Boolean(true));

    // Test search function dispatch
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "search",
        &[
            FilterExpression::Literal {
                value: FilterValue::String("hello world".to_string()),
            },
            FilterExpression::Literal {
                value: FilterValue::String("world".to_string()),
            },
        ],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Boolean(true));
}