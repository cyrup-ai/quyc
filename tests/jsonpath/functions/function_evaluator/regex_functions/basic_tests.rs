//! Basic tests for regex functions - First 30 lines
use serde_json::json;
use quyc_client::jsonpath::functions::function_evaluator::regex_functions::core::{evaluate_match_function, evaluate_search_function};
use quyc_client::jsonpath::parser::{FilterExpression, FilterValue};

// Mock evaluator for testing
fn mock_evaluator(
    _context: &serde_json::Value,
    expr: &quyc_client::jsonpath::parser::FilterExpression,
) -> quyc_client::jsonpath::error::JsonPathResult<quyc_client::jsonpath::parser::FilterValue> {
    match expr {
        FilterExpression::Literal { value } => Ok(value.clone()),
        _ => Ok(FilterValue::Null),
    }
}

#[test]
fn test_match_function_wrong_arg_count() {
    let context = json!({});
    let args = vec![];
    let result = evaluate_match_function(&context, &args, &mock_evaluator);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("exactly two arguments")
    );

    let args = vec![FilterExpression::Literal {
        value: FilterValue::String("test".to_string()),
    }];    let result = evaluate_match_function(&context, &args, &mock_evaluator);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("exactly two arguments")
    );
}

#[test]
fn test_search_function_wrong_arg_count() {
    let context = json!({});
    let args = vec![];
    let result = evaluate_search_function(&context, &args, &mock_evaluator);
    assert!(result.is_err());

    let args = vec![FilterExpression::Literal {
        value: FilterValue::String("test".to_string()),
    }];
    let result = evaluate_search_function(&context, &args, &mock_evaluator);
    assert!(result.is_err());
}

#[test]
fn test_match_function_valid_pattern() {
    let context = json!({});
    let args = vec![
        FilterExpression::Literal {
            value: FilterValue::String("hello world".to_string()),
        },
        FilterExpression::Literal {
            value: FilterValue::String("^hello".to_string()),
        },
    ];
    let result = evaluate_match_function(&context, &args, &mock_evaluator);
    assert_eq!(result.unwrap(), FilterValue::Boolean(true));
}#[test]
fn test_match_function_no_match() {
    let context = json!({});
    let args = vec![
        FilterExpression::Literal {
            value: FilterValue::String("hello world".to_string()),
        },
        FilterExpression::Literal {
            value: FilterValue::String("^world".to_string()),
        },
    ];
    let result = evaluate_match_function(&context, &args, &mock_evaluator);
    assert_eq!(result.unwrap(), FilterValue::Boolean(false));
}

#[test]
fn test_search_function_valid_pattern() {
    let context = json!({});
    let args = vec![
        FilterExpression::Literal {
            value: FilterValue::String("hello world".to_string()),
        },
        FilterExpression::Literal {
            value: FilterValue::String("world".to_string()),
        },
    ];
    let result = evaluate_search_function(&context, &args, &mock_evaluator);
    assert_eq!(result.unwrap(), FilterValue::Boolean(true));
}

#[test]
fn test_search_function_no_match() {
    let context = json!({});
    let args = vec![
        FilterExpression::Literal {
            value: FilterValue::String("hello world".to_string()),
        },
        FilterExpression::Literal {
            value: FilterValue::String("xyz".to_string()),
        },
    ];
    let result = evaluate_search_function(&context, &args, &mock_evaluator);
    assert_eq!(result.unwrap(), FilterValue::Boolean(false));
}