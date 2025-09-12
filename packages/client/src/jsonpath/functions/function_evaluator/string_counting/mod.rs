//! String counting functions for `JSONPath` function evaluator
//!
//! Provides string length and count operations for `JSONPath` expressions.

use serde_json::Value;

use crate::jsonpath::error::JsonPathResult;

/// Evaluate length function on a JSON value
///
/// # Errors
/// This function currently never fails but returns `JsonPathResult` for consistency
/// with the function evaluator interface.
pub fn evaluate_length_function(value: &Value) -> JsonPathResult<Value> {
    match value {
        Value::String(s) => Ok(Value::Number(serde_json::Number::from(s.len()))),
        Value::Array(arr) => Ok(Value::Number(serde_json::Number::from(arr.len()))),
        Value::Object(obj) => Ok(Value::Number(serde_json::Number::from(obj.len()))),
        _ => Ok(Value::Number(serde_json::Number::from(0))),
    }
}

/// Evaluate count function on a JSON value
///
/// # Errors
/// This function currently never fails but returns `JsonPathResult` for consistency
/// with the function evaluator interface.
pub fn evaluate_count_function(value: &Value) -> JsonPathResult<Value> {
    match value {
        Value::Array(arr) => Ok(Value::Number(serde_json::Number::from(arr.len()))),
        Value::Object(obj) => Ok(Value::Number(serde_json::Number::from(obj.len()))),
        _ => Ok(Value::Number(serde_json::Number::from(1))),
    }
}
