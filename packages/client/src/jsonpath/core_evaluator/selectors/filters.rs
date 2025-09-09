//! Filter selector operations
//!
//! Handles filter expression evaluation with context-aware property collection
//! for both owned and reference-based result collection.

use serde_json::Value;

use super::super::evaluator::CoreJsonPathEvaluator;
use crate::jsonpath::error::JsonPathError;
use crate::jsonpath::filter::FilterEvaluator;
use crate::jsonpath::parser::FilterExpression;

type JsonPathResult<T> = Result<T, JsonPathError>;

/// Apply filter selector using FilterEvaluator with owned results
pub fn apply_filter_selector_owned(
    evaluator: &CoreJsonPathEvaluator,
    node: &Value,
    expression: &FilterExpression,
    results: &mut Vec<Value>,
) -> JsonPathResult<()> {
    match node {
        Value::Array(arr) => {
            log::debug!(
                "apply_filter_selector_owned called on array with {} items",
                arr.len()
            );
            // Collect all property names that exist across items in this array
            let existing_properties = evaluator.collect_existing_properties(arr);

            for item in arr {
                if FilterEvaluator::evaluate_predicate_with_context(
                    item,
                    expression,
                    &existing_properties,
                )? {
                    results.push(item.clone());
                }
            }
        }
        Value::Object(_obj) => {
            // For objects, apply filter to the object itself
            // Create context with properties from this object
            let existing_properties: std::collections::HashSet<String> =
                std::collections::HashSet::new();

            if FilterEvaluator::evaluate_predicate_with_context(
                node,
                expression,
                &existing_properties,
            )? {
                results.push(node.clone());
            }
        }
        _ => {}
    }
    Ok(())
}

impl CoreJsonPathEvaluator {
    /// Collect all property names that exist across any item in the array
    pub fn collect_existing_properties(&self, arr: &[Value]) -> std::collections::HashSet<String> {
        let mut properties = std::collections::HashSet::new();

        for item in arr {
            if let Some(obj) = item.as_object() {
                for key in obj.keys() {
                    properties.insert(key.clone());
                }
            }
        }

        log::debug!("Collected existing properties: {:?}", properties);
        properties
    }

    /// Apply filter selector using FilterEvaluator
    pub fn apply_filter_selector<'a>(
        &self,
        node: &'a Value,
        expression: &FilterExpression,
        results: &mut Vec<&'a Value>,
    ) -> JsonPathResult<()> {
        match node {
            Value::Array(arr) => {
                tracing::debug!(
                    target: "quyc::jsonpath::selectors",
                    array_len = arr.len(),
                    "Applying filter selector to array"
                );
                // Collect all property names that exist across items in this array
                let existing_properties = self.collect_existing_properties(arr);

                for item in arr {
                    if FilterEvaluator::evaluate_predicate_with_context(
                        item,
                        expression,
                        &existing_properties,
                    )? {
                        results.push(item);
                    }
                }
            }
            Value::Object(_obj) => {
                // For objects, apply filter to the object itself
                // Create context with properties from this object
                let existing_properties: std::collections::HashSet<String> =
                    std::collections::HashSet::new();

                if FilterEvaluator::evaluate_predicate_with_context(
                    node,
                    expression,
                    &existing_properties,
                )? {
                    results.push(node);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
