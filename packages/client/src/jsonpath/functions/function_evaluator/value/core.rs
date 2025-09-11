//! RFC 9535 Section 2.4.8: `value()` function implementation
//!
//! Converts single-node nodelist to value (errors on multi-node or empty)

use super::super::core::FunctionEvaluator;
use crate::jsonpath::functions::jsonpath_nodelist::JsonPathNodelistEvaluator;
use crate::jsonpath::error::{JsonPathResult, constructors::invalid_expression_error};
use crate::jsonpath::parser::{FilterExpression, FilterValue};

/// RFC 9535 Section 2.4.8: `value()` function
/// Converts single-node nodelist to value (errors on multi-node or empty)
#[inline]
pub fn evaluate_value_function(
    context: &serde_json::Value,
    args: &[FilterExpression],
    expression_evaluator: &dyn Fn(
        &serde_json::Value,
        &FilterExpression,
    ) -> JsonPathResult<FilterValue>,
) -> JsonPathResult<FilterValue> {
    if args.len() != 1 {
        return Err(invalid_expression_error(
            "",
            "value() function requires exactly one argument",
            None,
        ));
    }

    match &args[0] {
        FilterExpression::JsonPath { selectors } => {
            evaluate_jsonpath_expression(context, selectors)
        }
        FilterExpression::Property { path } => evaluate_property_expression(context, path),
        FilterExpression::Current => evaluate_current_expression(context),
        FilterExpression::Literal { value } => evaluate_literal_expression(value),
        _ => {
            // For other expressions, evaluate directly (they produce single values)
            expression_evaluator(context, &args[0])
        }
    }
}

/// Evaluate `JSONPath` expression and validate nodelist size
fn evaluate_jsonpath_expression(
    context: &serde_json::Value,
    selectors: &[crate::jsonpath::parser::JsonSelector],
) -> JsonPathResult<FilterValue> {
    // Use proper JsonPathNodelistEvaluator to evaluate the JSONPath selectors
    let nodelist = JsonPathNodelistEvaluator::evaluate_jsonpath_nodelist(context, selectors)?;

    if nodelist.is_empty() {
        return Err(invalid_expression_error(
            "",
            "value() function requires non-empty nodelist",
            None,
        ));
    }

    if nodelist.len() > 1 {
        return Err(invalid_expression_error(
            "",
            "value() function requires single-node nodelist",
            None,
        ));
    }

    // Safe to unwrap since we verified length == 1
    Ok(FunctionEvaluator::json_value_to_filter_value(&nodelist[0]))
}

/// Evaluate property access expression
fn evaluate_property_expression(
    context: &serde_json::Value,
    path: &[String],
) -> JsonPathResult<FilterValue> {
    // Property access produces exactly one node or null
    let mut current = context;
    for segment in path {
        match current {
            serde_json::Value::Object(obj) => {
                current = obj.get(segment).map_or(&serde_json::Value::Null, |v| v);
            }
            _ => return Ok(FilterValue::Null),
        }
    }
    Ok(FunctionEvaluator::json_value_to_filter_value(current))
}

/// Evaluate current context expression
fn evaluate_current_expression(context: &serde_json::Value) -> JsonPathResult<FilterValue> {
    // Current context produces exactly one node
    Ok(FunctionEvaluator::json_value_to_filter_value(context))
}

/// Evaluate literal expression
fn evaluate_literal_expression(value: &FilterValue) -> JsonPathResult<FilterValue> {
    // Literal produces exactly one value
    Ok(value.clone())
}
