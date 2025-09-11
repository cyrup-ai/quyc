//! Value conversion functions for `JSONPath` function evaluator
//!
//! Provides type conversion operations for `JSONPath` expressions.

use serde_json::Value;

use crate::jsonpath::error::JsonPathResult;

/// Convert a JSON value to string
pub fn to_string_value(value: &Value) -> JsonPathResult<Value> {
    match value {
        Value::String(s) => Ok(Value::String(s.clone())),
        Value::Number(n) => Ok(Value::String(n.to_string())),
        Value::Bool(b) => Ok(Value::String(b.to_string())),
        Value::Null => Ok(Value::String("null".to_string())),
        _ => Ok(Value::String(value.to_string())),
    }
}

/// Convert a JSON value to number
pub fn to_number_value(value: &Value) -> JsonPathResult<Value> {
    match value {
        Value::Number(n) => Ok(Value::Number(n.clone())),
        Value::String(s) => {
            if let Ok(num) = s.parse::<f64>() {
                Ok(Value::Number(
                    serde_json::Number::from_f64(num).unwrap_or(serde_json::Number::from(0)),
                ))
            } else {
                Ok(Value::Number(serde_json::Number::from(0)))
            }
        }
        Value::Bool(true) => Ok(Value::Number(serde_json::Number::from(1))),
        Value::Bool(false) => Ok(Value::Number(serde_json::Number::from(0))),
        _ => Ok(Value::Number(serde_json::Number::from(0))),
    }
}

/// Convert a JSON value to boolean
pub fn to_boolean_value(value: &Value) -> JsonPathResult<Value> {
    match value {
        Value::Bool(b) => Ok(Value::Bool(*b)),
        Value::Number(n) => Ok(Value::Bool(n.as_f64().unwrap_or(0.0) != 0.0)),
        Value::String(s) => Ok(Value::Bool(!s.is_empty())),
        Value::Null => Ok(Value::Bool(false)),
        Value::Array(arr) => Ok(Value::Bool(!arr.is_empty())),
        Value::Object(obj) => Ok(Value::Bool(!obj.is_empty())),
    }
}

/// Evaluate value conversion functions
pub fn evaluate_value_function(function_name: &str, args: &[Value]) -> JsonPathResult<Value> {
    match function_name {
        "to_string" => {
            if args.len() != 1 {
                return Err(crate::jsonpath::error::invalid_expression_error(
                    "to_string",
                    "requires exactly 1 argument",
                    None,
                ));
            }
            to_string_value(&args[0])
        }
        "to_number" => {
            if args.len() != 1 {
                return Err(crate::jsonpath::error::invalid_expression_error(
                    "to_number",
                    "requires exactly 1 argument",
                    None,
                ));
            }
            to_number_value(&args[0])
        }
        "to_boolean" => {
            if args.len() != 1 {
                return Err(crate::jsonpath::error::invalid_expression_error(
                    "to_boolean",
                    "requires exactly 1 argument",
                    None,
                ));
            }
            to_boolean_value(&args[0])
        }
        _ => Err(crate::jsonpath::error::invalid_expression_error(
            function_name,
            "unknown value function",
            None,
        )),
    }
}
