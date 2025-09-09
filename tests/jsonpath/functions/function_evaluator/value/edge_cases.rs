//! Edge case tests for value() function
//!
//! Tests that verify proper handling of Unicode, special characters, and edge cases

use serde_json::json;
use quyc::json_path::parser::{FilterExpression, FilterValue};
use quyc::json_path::functions::function_evaluator::value::evaluate_value_function;

// Mock evaluator for testing
fn mock_evaluator(_context: &serde_json::Value, _expr: &FilterExpression) -> Result<FilterValue, Box<dyn std::error::Error + Send + Sync>> {
    Ok(FilterValue::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_function_unicode_handling() {
        let context = json!({"message": "Hello 世界 🌍"});
        let args = vec![FilterExpression::Property {
            path: vec!["message".to_string()],
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(
            result.unwrap(),
            FilterValue::String("Hello 世界 🌍".to_string())
        );
    }
}