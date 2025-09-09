//! Literal value tests for value() function
//!
//! Tests that verify literal value handling and expression delegation

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
    fn test_value_function_literal() {
        let context = json!({});
        let args = vec![FilterExpression::Literal {
            value: FilterValue::String("literal value".to_string()),
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(
            result.unwrap(),
            FilterValue::String("literal value".to_string())
        );

        let args = vec![FilterExpression::Literal {
            value: FilterValue::Integer(123),
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(result.unwrap(), FilterValue::Integer(123));
    }

    #[test]
    fn test_value_function_other_expressions() {
        let context = json!({});

        // Test with a function call expression (should delegate to evaluator)
        let args = vec![FilterExpression::Function {
            name: "length".to_string(),
            args: vec![FilterExpression::Literal {
                value: FilterValue::String("test".to_string()),
            }],
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(result.unwrap(), FilterValue::Null); // Mock evaluator returns null
    }
}