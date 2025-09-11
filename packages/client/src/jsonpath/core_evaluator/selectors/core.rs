//! Core selector application logic
//!
//! Handles the main selector application method and basic selector types
//! including Root, Child, `RecursiveDescent`, and Union selectors.

use serde_json::Value;

use super::super::evaluator::CoreJsonPathEvaluator;
use crate::jsonpath::error::JsonPathError;
use crate::jsonpath::parser::JsonSelector;

type JsonPathResult<T> = Result<T, JsonPathError>;

impl CoreJsonPathEvaluator {
    /// Apply a single selector to a JSON value, returning owned values
    pub fn apply_selector_to_value(
        &self,
        value: &Value,
        selector: &JsonSelector,
    ) -> JsonPathResult<Vec<Value>> {
        let mut results = Vec::new();

        match selector {
            JsonSelector::Root => {
                // Root selector returns the value itself
                results.push(value.clone());
            }
            JsonSelector::Child { name, .. } => {
                if let Value::Object(obj) = value
                    && let Some(child_value) = obj.get(name) {
                        results.push(child_value.clone());
                    }
            }
            JsonSelector::RecursiveDescent => {
                // Collect all descendants
                self.collect_all_descendants_owned(value, &mut results);
            }
            JsonSelector::Index { index, from_end } => {
                use super::arrays;
                arrays::apply_index_selector_owned(self, value, *index, *from_end, &mut results);
            }
            JsonSelector::Wildcard => {
                use super::wildcards;
                wildcards::apply_wildcard_owned(self, value, &mut results, 1000); // Reasonable limit
            }
            JsonSelector::Filter { expression } => {
                use super::filters;
                filters::apply_filter_selector_owned(self, value, expression, &mut results)?;
            }
            JsonSelector::Slice { start, end, step } => {
                if let Value::Array(arr) = value {
                    let slice_results = self.apply_slice_to_array(arr, *start, *end, *step)?;
                    results.extend(slice_results);
                }
            }
            JsonSelector::Union { selectors } => {
                // Apply each selector in the union and collect all results
                // RFC 9535: Union preserves order and duplicates
                for union_selector in selectors {
                    let union_results = self.apply_selector_to_value(value, union_selector)?;
                    results.extend(union_results);
                }
            }
        }

        Ok(results)
    }

    /// Collect all descendants using recursive descent (..) - owned version
    pub fn collect_all_descendants_owned(&self, node: &Value, results: &mut Vec<Value>) {
        match node {
            Value::Object(obj) => {
                for value in obj.values() {
                    results.push(value.clone());
                    self.collect_all_descendants_owned(value, results);
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    results.push(value.clone());
                    self.collect_all_descendants_owned(value, results);
                }
            }
            _ => {}
        }
    }

    /// Apply child selector to a node - handles object property access
    pub fn apply_child_selector<'a>(
        &self,
        node: &'a Value,
        name: &str,
        results: &mut Vec<&'a Value>,
    ) {
        if let Value::Object(obj) = node
            && let Some(value) = obj.get(name) {
                results.push(value);
            }
    }

    /// Collect all descendants using recursive descent (..)
    pub fn collect_all_descendants<'a>(&self, node: &'a Value, results: &mut Vec<&'a Value>) {
        match node {
            Value::Object(obj) => {
                for value in obj.values() {
                    results.push(value);
                    self.collect_all_descendants(value, results);
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    results.push(value);
                    self.collect_all_descendants(value, results);
                }
            }
            _ => {}
        }
    }
}
