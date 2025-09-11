//! `JSONPath` expression evaluation during streaming
//!
//! Provides path matching and navigation logic for `JSONPath` expressions
//! during incremental JSON parsing.

use crate::jsonpath::{ast::JsonSelector, error::JsonPathResult, expression::JsonPathExpression};

/// Result of path evaluation step
#[derive(Debug, Clone, PartialEq)]
pub enum PathEvaluationResult {
    /// Path matches current location - continue processing
    Match,
    /// Path doesn't match - skip current branch
    NoMatch,
    /// Need more data to determine match
    Pending,
    /// Reached target location - start extracting
    TargetReached,
}

/// Evaluate current JSON path against `JSONPath` expression
pub fn evaluate_path_step(
    expression: &JsonPathExpression,
    current_path: &[String],
    property_name: Option<&str>,
    is_array: bool,
    _array_index: Option<usize>,
) -> JsonPathResult<PathEvaluationResult> {
    // Simplified path evaluation - would need full JSONPath implementation
    // For now, just basic property matching

    if expression.selectors().is_empty() {
        return Ok(PathEvaluationResult::Match);
    }

    // Check if we're at the right depth
    if current_path.len() >= expression.selectors().len() {
        return Ok(PathEvaluationResult::TargetReached);
    }

    let current_selector = &expression.selectors()[current_path.len()];

    match current_selector {
        JsonSelector::Child { name, .. } => {
            if let Some(prop) = property_name {
                if prop == name {
                    Ok(PathEvaluationResult::Match)
                } else {
                    Ok(PathEvaluationResult::NoMatch)
                }
            } else {
                Ok(PathEvaluationResult::Pending)
            }
        }
        JsonSelector::Wildcard => Ok(PathEvaluationResult::Match),
        JsonSelector::Index { .. } => {
            if is_array {
                Ok(PathEvaluationResult::Match)
            } else {
                Ok(PathEvaluationResult::NoMatch)
            }
        }
        _ => {
            // Default case for other selector types
            Ok(PathEvaluationResult::Match)
        }
    }
}

/// Check if current location matches the target pattern
#[must_use] 
pub fn is_target_location(expression: &JsonPathExpression, current_path: &[String]) -> bool {
    current_path.len() == expression.selectors().len().saturating_sub(1)
}
