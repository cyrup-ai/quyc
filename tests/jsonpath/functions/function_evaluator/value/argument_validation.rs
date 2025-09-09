//! Argument validation tests for value() function
//!
//! Tests that verify proper argument count validation

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
    fn test_value_function_wrong_arg_count() {
        let context = json!({});
        let args = vec![];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exactly one argument")
        );

        let args = vec![
            FilterExpression::Literal {
                value: FilterValue::String("test".to_string()),
            },
            FilterExpression::Literal {
                value: FilterValue::String("extra".to_string()),
            },
        ];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exactly one argument")
        );
    }
}