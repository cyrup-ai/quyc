//! Core filter evaluation logic
//!
//! Handles the main evaluation entry points for filter expressions including
//! predicate evaluation and expression evaluation with context support.

use std::collections::HashSet;

use super::comparison::ValueComparator;
use super::property::PropertyResolver;
use super::selectors::SelectorEvaluator;
use super::utils::FilterUtils;
use crate::jsonpath::error::{JsonPathResult, deserialization_error};

/// Missing property context constant for filter evaluation
pub const MISSING_PROPERTY_CONTEXT: &str = "__MISSING_PROPERTY__";

/// Check if a value is truthy for filter evaluation
pub fn is_truthy(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        serde_json::Value::String(s) => !s.is_empty(),
        serde_json::Value::Array(arr) => !arr.is_empty(),
        serde_json::Value::Object(obj) => !obj.is_empty(),
    }
}

/// Evaluate JSONPath selectors for filter expressions
pub fn evaluate_jsonpath_selectors(
    context: &serde_json::Value,
    selectors: &[crate::jsonpath::ast::JsonSelector],
) -> JsonPathResult<Vec<serde_json::Value>> {
    // Use existing SelectorEngine to apply selectors
    use crate::jsonpath::core_evaluator::selector_engine::SelectorEngine;
    match SelectorEngine::apply_selectors(context, selectors) {
        Ok(results) => Ok(results),
        Err(e) => Err(deserialization_error(
            "selector evaluation failed".to_string(),
            e.to_string(),
            "JSONPath selectors"
        ))
    }
}
use crate::jsonpath::functions::FunctionEvaluator;
use crate::jsonpath::parser::{FilterExpression, FilterValue, LogicalOp};

/// Filter Expression Evaluator
pub struct FilterEvaluator;

impl FilterEvaluator {
    /// Evaluate filter predicate against JSON context
    #[inline]
    pub fn evaluate_predicate(
        context: &serde_json::Value,
        expr: &FilterExpression,
    ) -> JsonPathResult<bool> {
        // Use empty context for backward compatibility
        let empty_context = HashSet::new();
        Self::evaluate_predicate_with_context(context, expr, &empty_context)
    }

    /// Evaluate filter predicate with property existence context
    #[inline]
    pub fn evaluate_predicate_with_context(
        context: &serde_json::Value,
        expr: &FilterExpression,
        existing_properties: &HashSet<String>,
    ) -> JsonPathResult<bool> {
        tracing::debug!(
            target: "quyc::jsonpath::filter",
            context = %serde_json::to_string(context).unwrap_or("invalid".to_string()),
            expr = ?expr,
            "evaluate_predicate called"
        );
        match expr {
            FilterExpression::Property { path } => {
                // RFC 9535: Property access in filter context checks existence and truthiness
                // For @.author, this should return false if the object doesn't have an 'author' property
                tracing::debug!(
                    target: "quyc::jsonpath::filter",
                    path = ?path,
                    "Evaluating property filter"
                );
                PropertyResolver::property_exists_and_truthy(context, path)
            }
            FilterExpression::Comparison {
                left,
                operator,
                right,
            } => {
                let left_val =
                    Self::evaluate_expression_with_context(context, left, existing_properties)?;
                let right_val =
                    Self::evaluate_expression_with_context(context, right, existing_properties)?;
                ValueComparator::compare_values_with_context(
                    &left_val,
                    *operator,
                    &right_val,
                    existing_properties,
                )
            }
            FilterExpression::Logical {
                left,
                operator,
                right,
            } => {
                let left_result =
                    Self::evaluate_predicate_with_context(context, left, existing_properties)?;
                let right_result =
                    Self::evaluate_predicate_with_context(context, right, existing_properties)?;
                Ok(match operator {
                    LogicalOp::And => left_result && right_result,
                    LogicalOp::Or => left_result || right_result,
                })
            }
            FilterExpression::Function { name, args } => {
                let value = FunctionEvaluator::evaluate_function_value(
                    context,
                    name,
                    args,
                    &|ctx, expr| {
                        Self::evaluate_expression_with_context(ctx, expr, existing_properties)
                    },
                )?;
                Ok(FilterUtils::is_truthy(&value))
            }
            _ => Ok(FilterUtils::is_truthy(
                &Self::evaluate_expression_with_context(context, expr, existing_properties)?,
            )),
        }
    }

    /// Evaluate expression to get its value
    #[inline]
    pub fn evaluate_expression(
        context: &serde_json::Value,
        expr: &FilterExpression,
    ) -> JsonPathResult<FilterValue> {
        let empty_context = HashSet::new();
        Self::evaluate_expression_with_context(context, expr, &empty_context)
    }

    /// Evaluate expression with property context
    #[inline]
    pub fn evaluate_expression_with_context(
        context: &serde_json::Value,
        expr: &FilterExpression,
        existing_properties: &HashSet<String>,
    ) -> JsonPathResult<FilterValue> {
        match expr {
            FilterExpression::Current => Ok(PropertyResolver::json_value_to_filter_value(context)),
            FilterExpression::Property { path } => {
                PropertyResolver::resolve_property_path_with_context(
                    context,
                    path,
                    existing_properties,
                )
            }
            FilterExpression::JsonPath { selectors } => {
                SelectorEvaluator::evaluate_jsonpath_selectors(context, selectors)
            }
            FilterExpression::Literal { value } => Ok(value.clone()),
            FilterExpression::Function { name, args } => {
                FunctionEvaluator::evaluate_function_value(context, name, args, &|ctx, expr| {
                    Self::evaluate_expression_with_context(ctx, expr, existing_properties)
                })
            }
            _ => Err(deserialization_error(
                "complex expressions not supported in value context".to_string(),
                format!("{:?}", expr),
                "FilterValue",
            )),
        }
    }
}
