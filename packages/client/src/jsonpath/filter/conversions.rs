//! Value conversion and expression evaluation for filter expressions
//!
//! Contains logic for converting between JSON values and `FilterValues`,
//! and evaluating filter expressions to produce values.

use std::collections::HashSet;

use crate::jsonpath::error::{JsonPathResult, deserialization_error};
use crate::jsonpath::functions::FunctionEvaluator;
use crate::jsonpath::parser::{FilterExpression, FilterValue};

/// Convert `serde_json::Value` to `FilterValue`
#[inline]
pub(super) fn json_value_to_filter_value(value: &serde_json::Value) -> FilterValue {
    match value {
        serde_json::Value::Null => FilterValue::Null,
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
        // Arrays and objects should not convert to null - they're distinct values
        // For comparison purposes, we'll handle them specially in compare_values
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => FilterValue::Boolean(true), // Arrays and objects are truthy
    }
}

/// Evaluate expression to get its value
#[inline]
pub fn evaluate_expression(
    context: &serde_json::Value,
    expr: &FilterExpression,
) -> JsonPathResult<FilterValue> {
    let empty_context = HashSet::new();
    evaluate_expression_with_context(context, expr, &empty_context)
}

/// Evaluate expression with property context
#[inline]
pub fn evaluate_expression_with_context(
    context: &serde_json::Value,
    expr: &FilterExpression,
    existing_properties: &HashSet<String>,
) -> JsonPathResult<FilterValue> {
    match expr {
        FilterExpression::Current => Ok(json_value_to_filter_value(context)),
        FilterExpression::Property { path } => {
            super::properties::resolve_property_path_with_context(
                context,
                path,
                existing_properties,
            )
        }
        FilterExpression::JsonPath { selectors } => {
            super::selectors::SelectorEvaluator::evaluate_jsonpath_selectors(context, selectors)
        }
        FilterExpression::Literal { value } => Ok(value.clone()),
        FilterExpression::Function { name, args } => {
            FunctionEvaluator::evaluate_function_value(context, name, args, &|ctx, expr| {
                evaluate_expression_with_context(ctx, expr, existing_properties)
            })
        }
        _ => Err(deserialization_error(
            "complex expressions not supported in value context".to_string(),
            format!("{expr:?}"),
            "FilterValue",
        )),
    }
}
