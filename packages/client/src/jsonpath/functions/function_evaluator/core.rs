//! Core function evaluator and dispatcher
//!
//! This module contains the main `FunctionEvaluator` struct and the central
//! function dispatch logic for RFC 9535 `JSONPath` function extensions.

use crate::jsonpath::error::{JsonPathResult, invalid_expression_error};
use crate::jsonpath::parser::{FilterExpression, FilterValue};

/// RFC 9535 Function Extensions Implementation
pub struct FunctionEvaluator;

impl FunctionEvaluator {
    /// Evaluate function calls to get their actual values (RFC 9535 Section 2.4)
    #[inline]
    pub fn evaluate_function_value(
        context: &serde_json::Value,
        name: &str,
        args: &[FilterExpression],
        expression_evaluator: &dyn Fn(
            &serde_json::Value,
            &FilterExpression,
        ) -> JsonPathResult<FilterValue>,
    ) -> JsonPathResult<FilterValue> {
        match name {
            "length" => {
                super::length::evaluate_length_function(context, args, expression_evaluator)
            }
            "count" => super::count::evaluate_count_function(context, args, expression_evaluator),
            "match" => {
                super::regex_functions::evaluate_match_function(context, args, expression_evaluator)
            }
            "search" => super::regex_functions::evaluate_search_function(
                context,
                args,
                expression_evaluator,
            ),
            "value" => super::value::evaluate_value_function(context, args, expression_evaluator),
            _ => Err(invalid_expression_error(
                "",
                format!("unknown function: {name}"),
                None,
            )),
        }
    }

    /// Convert `serde_json::Value` to `FilterValue`
    #[inline]
    #[must_use] 
    pub fn json_value_to_filter_value(value: &serde_json::Value) -> FilterValue {
        match value {
            serde_json::Value::Bool(b) => FilterValue::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FilterValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    FilterValue::Number(f)
                } else {
                    FilterValue::Null
                }
            }
            serde_json::Value::String(s) => FilterValue::String(s.clone()),
            _ => FilterValue::Null, // Null, arrays and objects cannot be converted to FilterValue
        }
    }
}
