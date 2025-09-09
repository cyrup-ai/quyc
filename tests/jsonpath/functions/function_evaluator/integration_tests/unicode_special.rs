//! Unicode and special character integration tests
//!
//! Tests that verify proper handling of Unicode characters, emojis, and special characters

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
fn test_integration_unicode_and_special_characters() {
    let context = json!({
        "emoji": "🌍🌎🌏",
        "chinese": "你好世界",
        "mixed": "Hello 世界 🌍",
        "special": "line1\nline2\ttab"
    });

    // Test length with Unicode characters
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "length",
        &[FilterExpression::Property {
            path: vec!["emoji".to_string()],
        }],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Integer(3)); // 3 emoji characters

    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "length",
        &[FilterExpression::Property {
            path: vec!["chinese".to_string()],
        }],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Integer(4)); // 4 Chinese characters

    // Test regex with Unicode
    let result = FunctionEvaluator::evaluate_function_value(
        &context,
        "search",
        &[
            FilterExpression::Property {
                path: vec!["mixed".to_string()],
            },
            FilterExpression::Literal {
                value: FilterValue::String("世界".to_string()),
            },
        ],
        &mock_evaluator,
    );
    assert_eq!(result.unwrap(), FilterValue::Boolean(true));
}