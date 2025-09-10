//! Recursive descent operations for JSONPath evaluation
//!
//! Handles recursive descent (..) operations in JSONPath expressions
//! with proper depth tracking and cycle detection.

use serde_json::Value;

use crate::jsonpath::error::{JsonPathError, JsonPathResult};

/// Recursive descent evaluator for JSONPath expressions
pub struct RecursiveDescentEvaluator;

/// Recursive descent engine for JSONPath evaluation
pub struct RecursiveDescentEngine;

impl RecursiveDescentEvaluator {
    /// Apply recursive descent to find all matching nodes
    pub fn apply_recursive_descent(
        value: &Value,
        remaining_selectors: &[crate::jsonpath::ast::JsonSelector],
        max_depth: usize,
    ) -> JsonPathResult<Vec<Value>> {
        let mut results = Vec::new();
        Self::recursive_descent_impl(value, remaining_selectors, 0, max_depth, &mut results)?;
        Ok(results)
    }

    /// Internal recursive implementation with depth tracking
    fn recursive_descent_impl(
        current: &Value,
        selectors: &[crate::jsonpath::ast::JsonSelector],
        current_depth: usize,
        max_depth: usize,
        results: &mut Vec<Value>,
    ) -> JsonPathResult<()> {
        // Prevent infinite recursion
        if current_depth > max_depth {
            return Err(JsonPathError::invalid_expression(
                "recursive_descent",
                "maximum recursion depth exceeded",
                Some(current_depth),
            ));
        }

        // If no more selectors, add current value
        if selectors.is_empty() {
            results.push(current.clone());
            return Ok(());
        }

        // Try to match remaining selectors at current level
        if let Ok(matches) = Self::try_match_selectors(current, selectors) {
            results.extend(matches);
        }

        // Recursively descend into children
        match current {
            Value::Object(obj) => {
                for value in obj.values() {
                    Self::recursive_descent_impl(
                        value,
                        selectors,
                        current_depth + 1,
                        max_depth,
                        results,
                    )?;
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    Self::recursive_descent_impl(
                        value,
                        selectors,
                        current_depth + 1,
                        max_depth,
                        results,
                    )?;
                }
            }
            _ => {} // Leaf nodes have no children to descend into
        }

        Ok(())
    }

    /// Try to match selectors at current level
    fn try_match_selectors(
        value: &Value,
        selectors: &[crate::jsonpath::ast::JsonSelector],
    ) -> JsonPathResult<Vec<Value>> {
        // Use existing SelectorEngine to apply selectors at current level
        use crate::jsonpath::core_evaluator::selector_engine::SelectorEngine;
        SelectorEngine::apply_selectors(value, selectors)
    }
}
