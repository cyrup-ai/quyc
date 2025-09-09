//! Property access tests for value() function
//!
//! Tests that verify property path resolution and nested object access

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
    fn test_value_function_property_access() {
        let context = json!({"name": "John", "age": 30});
        let args = vec![FilterExpression::Property {
            path: vec!["name".to_string()],
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(result.unwrap(), FilterValue::String("John".to_string()));

        let args = vec![FilterExpression::Property {
            path: vec!["age".to_string()],
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(result.unwrap(), FilterValue::Integer(30));
    }

    #[test]
    fn test_value_function_property_null() {
        let context = json!({"value": null});
        let args = vec![FilterExpression::Property {
            path: vec!["value".to_string()],
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(result.unwrap(), FilterValue::Null);
    }

    #[test]
    fn test_value_function_property_missing() {
        let context = json!({"other": "value"});
        let args = vec![FilterExpression::Property {
            path: vec!["missing".to_string()],
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(result.unwrap(), FilterValue::Null);
    }

    #[test]
    fn test_value_function_property_nested() {
        let context = json!({"user": {"profile": {"name": "Alice"}}});
        let args = vec![FilterExpression::Property {
            path: vec![
                "user".to_string(),
                "profile".to_string(),
                "name".to_string(),
            ],
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(result.unwrap(), FilterValue::String("Alice".to_string()));
    }

    #[test]
    fn test_value_function_property_nested_missing() {
        let context = json!({"user": "not an object"});
        let args = vec![FilterExpression::Property {
            path: vec!["user".to_string(), "profile".to_string()],
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(result.unwrap(), FilterValue::Null);
    }

    #[test]
    fn test_value_function_deep_nesting() {
        let context = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": "deep"
                    }
                }
            }
        });
        let args = vec![FilterExpression::Property {
            path: vec![
                "level1".to_string(),
                "level2".to_string(),
                "level3".to_string(),
                "value".to_string(),
            ],
        }];
        let result = evaluate_value_function(&context, &args, &mock_evaluator);
        assert_eq!(result.unwrap(), FilterValue::String("deep".to_string()));
    }
}