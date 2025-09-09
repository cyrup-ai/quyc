//! Count function tests for JSONPath function evaluator

use quyc::json_path::functions::function_evaluator::count::evaluate_count_function;
use quyc::json_path::error::JsonPathResult;
use quyc::json_path::parser::{FilterExpression, FilterValue};
use serde_json::json;

fn mock_evaluator(
    _context: &serde_json::Value,
    expr: &FilterExpression,
) -> JsonPathResult<FilterValue> {
    match expr {
        FilterExpression::Literal { value } => Ok(value.clone()),
        _ => Ok(FilterValue::Null),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_function_wrong_arg_count() {
        let context = json!({});
        let args = vec![];
        let result = evaluate_count_function(&context, &args, &mock_evaluator);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exactly one argument")
        );
    }

    #[test]
    fn test_count_function_array() {
        let context = json!([1, 2, 3, 4, 5]);
        let args = vec![FilterExpression::Literal {
            value: FilterValue::Array(vec![
                FilterValue::Number(1.0),
                FilterValue::Number(2.0),
                FilterValue::Number(3.0),
            ]),
        }];
        let result = evaluate_count_function(&context, &args, &mock_evaluator);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilterValue::Number(3.0));
    }

    #[test]
    fn test_count_function_object() {
        let context = json!({"a": 1, "b": 2});
        let args = vec![FilterExpression::Literal {
            value: FilterValue::Object(std::collections::HashMap::from([
                ("key1".to_string(), FilterValue::String("value1".to_string())),
                ("key2".to_string(), FilterValue::String("value2".to_string())),
            ])),
        }];
        let result = evaluate_count_function(&context, &args, &mock_evaluator);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilterValue::Number(2.0));
    }

    #[test]
    fn test_count_function_string() {
        let context = json!("hello");
        let args = vec![FilterExpression::Literal {
            value: FilterValue::String("test".to_string()),
        }];
        let result = evaluate_count_function(&context, &args, &mock_evaluator);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilterValue::Number(4.0));
    }

    #[test]
    fn test_count_function_null() {
        let context = json!(null);
        let args = vec![FilterExpression::Literal {
            value: FilterValue::Null,
        }];
        let result = evaluate_count_function(&context, &args, &mock_evaluator);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilterValue::Number(0.0));
    }

    #[test]
    fn test_count_function_boolean() {
        let context = json!(true);
        let args = vec![FilterExpression::Literal {
            value: FilterValue::Boolean(true),
        }];
        let result = evaluate_count_function(&context, &args, &mock_evaluator);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilterValue::Number(1.0));
    }

    #[test]
    fn test_count_function_empty_array() {
        let context = json!([]);
        let args = vec![FilterExpression::Literal {
            value: FilterValue::Array(vec![]),
        }];
        let result = evaluate_count_function(&context, &args, &mock_evaluator);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FilterValue::Number(0.0));
    }
}