//! Comparison operations with context-aware logic for filter expressions
//!
//! Contains logic for comparing FilterValues with proper handling of
//! missing vs null semantics and context-aware comparisons.

use std::collections::HashSet;

use super::property::MISSING_PROPERTY_CONTEXT;
use crate::jsonpath::error::JsonPathResult;
use crate::jsonpath::parser::{ComparisonOp, FilterValue};

/// Compare two filter values using the specified operator
#[inline]
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
        (FilterValue::Integer(a), FilterValue::Integer(b)) => Ok(match op {
            ComparisonOp::Equal => a == b,
            ComparisonOp::NotEqual => a != b,
            ComparisonOp::Less => a < b,
            ComparisonOp::LessEq => a <= b,
            ComparisonOp::Greater => a > b,
            ComparisonOp::GreaterEq => a >= b,
            _ => false,
        }),
        (FilterValue::Number(a), FilterValue::Number(b)) => Ok(match op {
            ComparisonOp::Equal => (a - b).abs() < f64::EPSILON,
            ComparisonOp::NotEqual => (a - b).abs() >= f64::EPSILON,
            ComparisonOp::Less => a < b,
            ComparisonOp::LessEq => a <= b,
            ComparisonOp::Greater => a > b,
            ComparisonOp::GreaterEq => a >= b,
            _ => false,
        }),
        (FilterValue::String(a), FilterValue::String(b)) => Ok(match op {
            ComparisonOp::Equal => a == b,
            ComparisonOp::NotEqual => a != b,
            ComparisonOp::Less => a < b,
            ComparisonOp::LessEq => a <= b,
            ComparisonOp::Greater => a > b,
            ComparisonOp::GreaterEq => a >= b,
            _ => false,
        }),
        (FilterValue::Boolean(a), FilterValue::Boolean(b)) => Ok(match op {
            ComparisonOp::Equal => a == b,
            ComparisonOp::NotEqual => a != b,
            _ => false,
        }),
        // RFC 9535: Missing properties context-aware comparison
        (FilterValue::Missing, FilterValue::Null) => {
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
                        ComparisonOp::Equal => false, // missing is never equal to null
                        ComparisonOp::NotEqual => exists_in_context, // missing != null only if property exists somewhere
                        _ => false,
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
        (FilterValue::Null, FilterValue::Missing) => {
            MISSING_PROPERTY_CONTEXT.with(|ctx| {
                if let Some((_, exists_in_context)) = ctx.borrow().clone() {
                    // Clear the context after use
                    *ctx.borrow_mut() = None;
                    Ok(match op {
                        ComparisonOp::Equal => false, // null is never equal to missing
                        ComparisonOp::NotEqual => exists_in_context, // null != missing only if property exists somewhere
                        _ => false,
                    })
                } else {
                    // Fallback: missing properties don't participate in comparisons
                    Ok(false)
                }
            })
        }
        // Other missing property comparisons always false
        (FilterValue::Missing, _) => Ok(false),
        (_, FilterValue::Missing) => Ok(false),
        // RFC 9535: Null value comparisons
        (FilterValue::Null, FilterValue::Null) => Ok(match op {
            ComparisonOp::Equal => true,
            ComparisonOp::NotEqual => false,
            _ => false,
        }),
        (FilterValue::Null, _) => Ok(match op {
            ComparisonOp::Equal => false,
            ComparisonOp::NotEqual => true,
            _ => false,
        }),
        (_, FilterValue::Null) => Ok(match op {
            ComparisonOp::Equal => false,
            ComparisonOp::NotEqual => true,
            _ => false,
        }),
        // Type coercion for number/integer comparisons
        (FilterValue::Integer(a), FilterValue::Number(b)) => compare_values(
            &FilterValue::Number(*a as f64),
            op,
            &FilterValue::Number(*b),
        ),
        (FilterValue::Number(a), FilterValue::Integer(b)) => compare_values(
            &FilterValue::Number(*a),
            op,
            &FilterValue::Number(*b as f64),
        ),
        _ => Ok(false), // Other cross-type comparisons are false
    }
}
