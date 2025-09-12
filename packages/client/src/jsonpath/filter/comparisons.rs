//! Comparison operations with context-aware logic for filter expressions
//!
//! Contains logic for comparing `FilterValues` with proper handling of
//! missing vs null semantics and context-aware comparisons.

#![allow(clippy::cast_precision_loss)]

use std::collections::HashSet;

use super::property::MISSING_PROPERTY_CONTEXT;
use crate::jsonpath::error::JsonPathResult;
use crate::jsonpath::parser::{ComparisonOp, FilterValue};

/// Compare two filter values using the specified operator
#[inline]
#[allow(clippy::cast_precision_loss)]
pub fn compare_values(
    left: &FilterValue,
    op: ComparisonOp,
    right: &FilterValue,
) -> JsonPathResult<bool> {
    let empty_context = HashSet::new();
    compare_values_with_context(left, op, right, &empty_context)
}

/// Compare two filter values with property existence context
#[inline]
pub fn compare_values_with_context(
    left: &FilterValue,
    op: ComparisonOp,
    right: &FilterValue,
    _existing_properties: &HashSet<String>,
) -> JsonPathResult<bool> {
    match (left, right) {
        (FilterValue::Integer(a), FilterValue::Integer(b)) => Ok(compare_integers(*a, *b, op)),
        (FilterValue::Number(a), FilterValue::Number(b)) => Ok(compare_numbers(*a, *b, op)),
        (FilterValue::String(a), FilterValue::String(b)) => Ok(compare_strings(a, b, op)),
        (FilterValue::Boolean(a), FilterValue::Boolean(b)) => Ok(compare_booleans(*a, *b, op)),
        (FilterValue::Missing, FilterValue::Null) => compare_missing_vs_null(op),
        (FilterValue::Null, FilterValue::Missing) => compare_null_vs_missing(op),
        (FilterValue::Missing, _) | (_, FilterValue::Missing) => Ok(false),
        (FilterValue::Null, FilterValue::Null) => Ok(compare_null_values(op)),
        (FilterValue::Null, _) | (_, FilterValue::Null) => Ok(compare_null_with_value(op)),
        (FilterValue::Integer(a), FilterValue::Number(b)) => compare_integer_with_number(*a, *b, op),
        (FilterValue::Number(a), FilterValue::Integer(b)) => compare_number_with_integer(*a, *b, op),
        _ => Ok(false), // Other cross-type comparisons are false
    }
}

/// Compare two integer values
#[inline]
fn compare_integers(a: i64, b: i64, op: ComparisonOp) -> bool {
    match op {
        ComparisonOp::Equal => a == b,
        ComparisonOp::NotEqual => a != b,
        ComparisonOp::Less => a < b,
        ComparisonOp::LessEq => a <= b,
        ComparisonOp::Greater => a > b,
        ComparisonOp::GreaterEq => a >= b,
        _ => false,
    }
}

/// Compare two floating point numbers with epsilon tolerance
#[inline]
fn compare_numbers(a: f64, b: f64, op: ComparisonOp) -> bool {
    match op {
        ComparisonOp::Equal => (a - b).abs() < f64::EPSILON,
        ComparisonOp::NotEqual => (a - b).abs() >= f64::EPSILON,
        ComparisonOp::Less => a < b,
        ComparisonOp::LessEq => a <= b,
        ComparisonOp::Greater => a > b,
        ComparisonOp::GreaterEq => a >= b,
        _ => false,
    }
}

/// Compare two string values
#[inline]
fn compare_strings(a: &str, b: &str, op: ComparisonOp) -> bool {
    match op {
        ComparisonOp::Equal => a == b,
        ComparisonOp::NotEqual => a != b,
        ComparisonOp::Less => a < b,
        ComparisonOp::LessEq => a <= b,
        ComparisonOp::Greater => a > b,
        ComparisonOp::GreaterEq => a >= b,
        _ => false,
    }
}

/// Compare two boolean values
#[inline]
fn compare_booleans(a: bool, b: bool, op: ComparisonOp) -> bool {
    match op {
        ComparisonOp::Equal => a == b,
        ComparisonOp::NotEqual => a != b,
        _ => false,
    }
}

/// RFC 9535: Context-aware comparison of missing property vs null
fn compare_missing_vs_null(op: ComparisonOp) -> JsonPathResult<bool> {
    MISSING_PROPERTY_CONTEXT.with(|ctx| {
        let context_info = ctx.borrow().clone();
        tracing::debug!(
            target: "quyc::jsonpath::filter",
            context = ?context_info,
            op = ?op,
            "Missing vs Null comparison with RFC 9535 context"
        );
        if let Some((property_name, exists_in_context)) = context_info {
            // Clear the context after use
            *ctx.borrow_mut() = None;
            let result = match op {
                ComparisonOp::NotEqual => exists_in_context, // missing != null only if property exists somewhere
                _ => false, // missing is never equal to null (and false for other comparisons)
            };
            tracing::debug!(
                target: "quyc::jsonpath::filter",
                property = %property_name,
                exists_in_context = exists_in_context,
                result = result,
                "Context-aware comparison result"
            );
            Ok(result)
        } else {
            // Fallback: missing properties don't participate in comparisons
            tracing::debug!(
                target: "quyc::jsonpath::filter",
                "No context available for missing property comparison, returning false"
            );
            Ok(false)
        }
    })
}

/// RFC 9535: Context-aware comparison of null vs missing property
fn compare_null_vs_missing(op: ComparisonOp) -> JsonPathResult<bool> {
    MISSING_PROPERTY_CONTEXT.with(|ctx| {
        if let Some((_, exists_in_context)) = ctx.borrow().clone() {
            // Clear the context after use
            *ctx.borrow_mut() = None;
            Ok(match op {
                ComparisonOp::NotEqual => exists_in_context, // null != missing only if property exists somewhere
                _ => false, // null is never equal to missing (and false for other comparisons)
            })
        } else {
            // Fallback: missing properties don't participate in comparisons
            Ok(false)
        }
    })
}

/// Compare two null values
#[inline]
fn compare_null_values(op: ComparisonOp) -> bool {
    match op {
        ComparisonOp::Equal => true,
        _ => false, // NotEqual and other operations are false for null == null
    }
}

/// Compare null with non-null value
#[inline]
fn compare_null_with_value(op: ComparisonOp) -> bool {
    match op {
        ComparisonOp::NotEqual => true,
        _ => false, // Equal and other operations are false for null comparisons
    }
}

/// Type coercion: Compare integer with number, preserving precision
fn compare_integer_with_number(a: i64, b: f64, op: ComparisonOp) -> JsonPathResult<bool> {
    // Use safe integer to f64 conversion that preserves precision when possible
    if a.abs() > (1i64 << 53) {
        // For very large integers, compare as strings to maintain exact precision
        compare_values(
            &FilterValue::String(a.to_string()),
            op,
            &FilterValue::String(b.to_string()),
        )
    } else {
        // Safe to convert to f64 without precision loss
        compare_values(&FilterValue::Number(a as f64), op, &FilterValue::Number(b))
    }
}

/// Type coercion: Compare number with integer, preserving precision
fn compare_number_with_integer(a: f64, b: i64, op: ComparisonOp) -> JsonPathResult<bool> {
    // Use safe integer to f64 conversion that preserves precision when possible
    if b.abs() > (1i64 << 53) {
        // For very large integers, compare as strings to maintain exact precision
        compare_values(
            &FilterValue::String(a.to_string()),
            op,
            &FilterValue::String(b.to_string()),
        )
    } else {
        // Safe to convert to f64 without precision loss
        compare_values(&FilterValue::Number(a), op, &FilterValue::Number(b as f64))
    }
}
