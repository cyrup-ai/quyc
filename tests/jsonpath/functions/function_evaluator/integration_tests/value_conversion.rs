//! JSON value conversion integration tests
//!
//! Tests that verify proper conversion between JSON values and FilterValue types

use serde_json::json;
use quyc_client::jsonpath::functions::FunctionEvaluator;
use quyc_client::jsonpath::parser::FilterValue;

#[test]
fn test_integration_json_value_conversion() {
    // Test all JSON value types conversion
    assert_eq!(
        FunctionEvaluator::json_value_to_filter_value(&json!(null)),
        FilterValue::Null
    );
    assert_eq!(
        FunctionEvaluator::json_value_to_filter_value(&json!(true)),
        FilterValue::Boolean(true)
    );
    assert_eq!(
        FunctionEvaluator::json_value_to_filter_value(&json!(42)),
        FilterValue::Integer(42)
    );
    assert_eq!(
        FunctionEvaluator::json_value_to_filter_value(&json!(3.14)),
        FilterValue::Number(3.14)
    );
    assert_eq!(
        FunctionEvaluator::json_value_to_filter_value(&json!("test")),
        FilterValue::String("test".to_string())
    );
    assert_eq!(
        FunctionEvaluator::json_value_to_filter_value(&json!([1, 2, 3])),
        FilterValue::Null
    );
    assert_eq!(
        FunctionEvaluator::json_value_to_filter_value(&json!({"key": "value"})),
        FilterValue::Null
    );
}