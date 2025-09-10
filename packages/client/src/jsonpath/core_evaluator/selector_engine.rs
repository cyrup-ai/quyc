//! Selector application engine for JSONPath evaluation
//!
//! This module handles the core logic for applying individual selectors to JSON values.

use serde_json::Value;

use super::super::error::JsonPathError;
use super::super::parser::{FilterExpression, JsonSelector};
use super::array_operations::ArrayOperations;
use super::engine::CoreJsonPathEvaluator;
use super::filter_support::FilterSupport;
use crate::jsonpath::FilterEvaluator;

type JsonPathResult<T> = Result<T, JsonPathError>;

/// Engine for applying individual selectors to JSON values
pub struct SelectorEngine;

impl SelectorEngine {
    /// Apply a single selector to a JSON value, returning owned values
    pub fn apply_selector(value: &Value, selector: &JsonSelector) -> JsonPathResult<Vec<Value>> {
        let mut results = Vec::new();

        match selector {
            JsonSelector::Root => {
                // Root selector returns the value itself
                results.push(value.clone());
            }
            JsonSelector::Child { name, .. } => {
                if let Value::Object(obj) = value {
                    if let Some(child_value) = obj.get(name) {
                        results.push(child_value.clone());
                    }
                }
            }
            JsonSelector::RecursiveDescent => {
                // Collect all descendants
                match CoreJsonPathEvaluator::new("$..") {
                    Ok(evaluator) => {
                        let descendants = evaluator.collect_descendants(value);
                        results.extend(descendants);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            JsonSelector::Index { index, from_end } => {
                if let Value::Array(arr) = value {
                    let array_results = ArrayOperations::apply_index(arr, *index, *from_end)?;
                    results.extend(array_results);
                }
            }
            JsonSelector::Slice { start, end, step } => {
                if let Value::Array(arr) = value {
                    let slice_results =
                        ArrayOperations::apply_slice(arr, *start, *end, step.unwrap_or(1))?;
                    results.extend(slice_results);
                }
            }
            JsonSelector::Wildcard => {
                Self::apply_wildcard(value, &mut results);
            }
            JsonSelector::Filter { expression } => {
                Self::apply_filter(value, expression, &mut results)?;
            }
            JsonSelector::Union { selectors } => {
                for selector in selectors {
                    let union_results = Self::apply_selector(value, selector)?;
                    results.extend(union_results);
                }
            }
        }

        Ok(results)
    }

    /// Apply wildcard selector to get all children
    fn apply_wildcard(value: &Value, results: &mut Vec<Value>) {
        match value {
            Value::Object(obj) => {
                for child_value in obj.values() {
                    results.push(child_value.clone());
                }
            }
            Value::Array(arr) => {
                for child_value in arr {
                    results.push(child_value.clone());
                }
            }
            _ => {} // Primitives have no children
        }
    }

    /// Apply filter selector using FilterEvaluator
    fn apply_filter(
        value: &Value,
        expression: &FilterExpression,
        results: &mut Vec<Value>,
    ) -> JsonPathResult<()> {
        // RFC 9535 Section 2.3.5.2: Filter selector tests children of input value
        match value {
            Value::Array(arr) => {
                // For arrays: collect existing properties first for context-aware evaluation
                let existing_properties = FilterSupport::collect_existing_properties(arr);

                log::debug!(
                    "Array filter - collected {} existing properties: {:?}",
                    existing_properties.len(),
                    existing_properties
                );

                // For arrays: test each element (child) against filter with context
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
                    value,
                    expression,
                    &existing_properties,
                )? {
                    results.push(value.clone());
                }
            }
            _ => {
                // For primitives, the filter doesn't apply (no children to test)
                // This is correct per RFC 9535 - filters only apply to structured values
            }
        }
        Ok(())
    }

    /// Apply multiple selectors in sequence
    pub fn apply_selectors(
        initial_value: &Value,
        selectors: &[JsonSelector],
    ) -> JsonPathResult<Vec<Value>> {
        let mut current_results = vec![initial_value.clone()];

        for selector in selectors {
            let mut next_results = Vec::new();

            for value in &current_results {
                let selector_results = Self::apply_selector(value, selector)?;
                next_results.extend(selector_results);

                // Safety check: prevent memory exhaustion
                if next_results.len() > 10000 {
                    log::warn!(
                        "Selector application stopped - result set too large ({})",
                        next_results.len()
                    );
                    return Ok(vec![]);
                }
            }

            current_results = next_results;

            // Early termination if no results
            if current_results.is_empty() {
                return Ok(vec![]);
            }
        }

        Ok(current_results)
    }

    /// Check if a selector is potentially expensive
    pub fn is_expensive_selector(selector: &JsonSelector) -> bool {
        match selector {
            JsonSelector::RecursiveDescent => true,
            JsonSelector::Wildcard => true,
            JsonSelector::Filter { .. } => true,
            JsonSelector::Slice { .. } => true,
            JsonSelector::Union { .. } => true,
            _ => false,
        }
    }

    /// Estimate the complexity of a selector
    pub fn selector_complexity(selector: &JsonSelector) -> u32 {
        match selector {
            JsonSelector::Root => 1,
            JsonSelector::Child { .. } => 1,
            JsonSelector::Index { .. } => 1,
            JsonSelector::Wildcard => 10,
            JsonSelector::Slice { .. } => 5,
            JsonSelector::Filter { .. } => 20,
            JsonSelector::RecursiveDescent => 50,
            JsonSelector::Union { selectors } => {
                selectors
                    .iter()
                    .map(|s| Self::selector_complexity(s))
                    .sum::<u32>()
                    + 5
            }
        }
    }
}


