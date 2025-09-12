//! Core `DescendantOperations` struct and basic traversal methods
//!
//! Contains the main struct definition and fundamental recursive descent operations
//! for `JSONPath` processing with RFC 9535 compliance.

use serde_json::Value;

use super::super::core_types::JsonPathResult;
use crate::jsonpath::parser::JsonSelector;

/// Operations for handling descendant traversal in `JSONPath` expressions
pub struct DescendantOperations;

impl DescendantOperations {
    /// Apply descendant segment recursively for RFC 9535 compliance
    ///
    /// # Errors
    /// Returns `JsonPathError` if:
    /// - Selector application fails on any descendant node
    /// - Memory limits are exceeded during recursive traversal
    /// - Invalid selector operations are encountered
    pub fn apply_descendant_segment_recursive(
        node: &Value,
        remaining_selectors: &[JsonSelector],
        results: &mut Vec<Value>,
    ) -> JsonPathResult<()> {
        // Apply selectors to current node
        let mut current_results = vec![node.clone()];

        for selector in remaining_selectors {
            let mut next_results = Vec::new();
            for current_value in &current_results {
                let intermediate_results = Self::apply_selector_to_value(current_value, selector)?;
                next_results.extend(intermediate_results);
            }
            current_results = next_results;
        }

        results.extend(current_results);

        // Recursively apply to descendants
        match node {
            Value::Object(obj) => {
                for value in obj.values() {
                    Self::apply_descendant_segment_recursive(value, remaining_selectors, results)?;
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    Self::apply_descendant_segment_recursive(value, remaining_selectors, results)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Apply a single selector to a JSON value (helper method)
    fn apply_selector_to_value(
        value: &Value,
        selector: &JsonSelector,
    ) -> JsonPathResult<Vec<Value>> {
        use crate::jsonpath::core_evaluator::selector_engine::SelectorEngine;
        SelectorEngine::apply_selector(value, selector)
    }

    /// Check if a value has any descendants
    #[must_use] 
    pub fn has_descendants(node: &Value) -> bool {
        match node {
            Value::Object(obj) => !obj.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            _ => false,
        }
    }
}
