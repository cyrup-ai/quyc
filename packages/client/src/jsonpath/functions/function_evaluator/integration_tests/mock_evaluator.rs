//! Mock evaluator for integration tests
//!
//! Provides a simple mock implementation for testing function evaluator behavior

use crate::jsonpath::error::JsonPathResult;
use crate::jsonpath::parser::{FilterExpression, FilterValue};

/// Mock evaluator that returns literals as-is and null for other expressions
///
/// # Errors
///
/// This mock implementation never returns errors - always succeeds with either
/// the literal value or `FilterValue::Null` for non-literal expressions.
pub fn mock_evaluator(
    _context: &serde_json::Value,
    expr: &FilterExpression,
) -> JsonPathResult<FilterValue> {
    match expr {
        FilterExpression::Literal { value } => Ok(value.clone()),
        _ => Ok(FilterValue::Null),
    }
}
