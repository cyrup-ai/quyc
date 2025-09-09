use serde_json::json;
use quyc_client::jsonpath::functions::function_evaluator::length::*;
use quyc_client::jsonpath::filter::{FilterExpression, FilterValue};
use quyc_client::jsonpath::error::JsonPathResult;

fn mock_evaluator(
    _context: &serde_json::Value,
    expr: &FilterExpression,
) -> JsonPathResult<FilterValue> {
    match expr {
        FilterExpression::Literal { value } => Ok(value.clone()),
        _ => Ok(FilterValue::Null),
    }
}

#[test]
fn test_length_function_wrong_arg_count() {
    let context = json!({});
    let args = vec![];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert!(result.is_err());
    assert!(
        result
            .expect_err("Should fail with wrong arg count")
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
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert!(result.is_err());
    assert!(
        result
            .expect_err("Should fail with too many args")
            .to_string()
            .contains("exactly one argument")
    );
}

#[test]
fn test_length_function_property_array() {
    let context = json!({"items": [1, 2, 3, 4, 5]});
    let args = vec![FilterExpression::Property {
        path: vec!["items".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Array length should work"), FilterValue::Integer(5));
}

#[test]
fn test_length_function_property_object() {
    let context = json!({"user": {"name": "John", "age": 30, "city": "NYC"}});
    let args = vec![FilterExpression::Property {
        path: vec!["user".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Object length should work"), FilterValue::Integer(3));
}

#[test]
fn test_length_function_property_string() {
    let context = json!({"message": "Hello World"});
    let args = vec![FilterExpression::Property {
        path: vec!["message".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("String length should work"), FilterValue::Integer(11));
}

#[test]
fn test_length_function_property_unicode_string() {
    let context = json!({"text": "Hello 世界 🌍"});
    let args = vec![FilterExpression::Property {
        path: vec!["text".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Unicode length should work"), FilterValue::Integer(10)); // Unicode-aware counting
}

#[test]
fn test_length_function_property_null() {
    let context = json!({"value": null});
    let args = vec![FilterExpression::Property {
        path: vec!["value".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Null length should work"), FilterValue::Null);
}

#[test]
fn test_length_function_property_primitive() {
    let context = json!({"number": 42});
    let args = vec![FilterExpression::Property {
        path: vec!["number".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Number length should return null"), FilterValue::Null); // Primitives return null per RFC

    let context = json!({"flag": true});
    let args = vec![FilterExpression::Property {
        path: vec!["flag".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Boolean length should return null"), FilterValue::Null);
}

#[test]
fn test_length_function_property_missing() {
    let context = json!({"other": "value"});
    let args = vec![FilterExpression::Property {
        path: vec!["missing".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Missing property should return null"), FilterValue::Null);
}

#[test]
fn test_length_function_property_nested() {
    let context = json!({"data": {"items": [1, 2, 3]}});
    let args = vec![FilterExpression::Property {
        path: vec!["data".to_string(), "items".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Nested property length should work"), FilterValue::Integer(3));
}

#[test]
fn test_length_function_property_nested_missing() {
    let context = json!({"data": "not an object"});
    let args = vec![FilterExpression::Property {
        path: vec!["data".to_string(), "items".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Missing nested property should return null"), FilterValue::Null);
}

#[test]
fn test_length_function_literal_string() {
    let context = json!({});
    let args = vec![FilterExpression::Literal {
        value: FilterValue::String("test string".to_string()),
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Literal string length should work"), FilterValue::Integer(11));
}

#[test]
fn test_length_function_literal_primitives() {
    let context = json!({});

    let args = vec![FilterExpression::Literal {
        value: FilterValue::Integer(42),
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Integer literal should return null"), FilterValue::Null);

    let args = vec![FilterExpression::Literal {
        value: FilterValue::Number(3.14),
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Number literal should return null"), FilterValue::Null);

    let args = vec![FilterExpression::Literal {
        value: FilterValue::Boolean(true),
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Boolean literal should return null"), FilterValue::Null);
}

#[test]
fn test_length_function_literal_null_and_missing() {
    let context = json!({});

    let args = vec![FilterExpression::Literal {
        value: FilterValue::Null,
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Null literal should return null"), FilterValue::Null);

    let args = vec![FilterExpression::Literal {
        value: FilterValue::Missing,
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Missing literal should return null"), FilterValue::Null);
}

#[test]
fn test_length_function_empty_collections() {
    let context = json!({"empty_array": [], "empty_object": {}, "empty_string": ""});

    let args = vec![FilterExpression::Property {
        path: vec!["empty_array".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Empty array length should be 0"), FilterValue::Integer(0));

    let args = vec![FilterExpression::Property {
        path: vec!["empty_object".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Empty object length should be 0"), FilterValue::Integer(0));

    let args = vec![FilterExpression::Property {
        path: vec!["empty_string".to_string()],
    }];
    let result = evaluate_length_function(&context, &args, &mock_evaluator);
    assert_eq!(result.expect("Empty string length should be 0"), FilterValue::Integer(0));
}