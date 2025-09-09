//! Descendant collection operations for JSONPath evaluation
//!
//! Contains methods for collecting all descendants and applying selectors recursively.

use serde_json::Value;

use super::core_evaluator::{CoreJsonPathEvaluator, JsonPathResult};
use crate::jsonpath::parser::JsonSelector;

impl CoreJsonPathEvaluator {
    /// Collect all descendant values from a JSON structure
    pub fn collect_descendants(&self, json: &Value) -> Vec<Value> {
        let mut descendants = Vec::new();
        self.collect_descendants_impl(json, &mut descendants);
        descendants
    }

    fn collect_descendants_impl(&self, json: &Value, descendants: &mut Vec<Value>) {
        // Add the current value
        descendants.push(json.clone());

        // Recurse into children
        match json {
            Value::Object(obj) => {
                for value in obj.values() {
                    self.collect_descendants_impl(value, descendants);
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    self.collect_descendants_impl(value, descendants);
                }
            }
            _ => {
                // Leaf values have no descendants beyond themselves
            }
        }
    }

    /// Apply a selector to all descendants of a value
    pub fn apply_selector_to_descendants(
        &self,
        json: &Value,
        selector: &JsonSelector,
    ) -> JsonPathResult<Vec<Value>> {
        let descendants = self.collect_descendants(json);
        let mut results = Vec::new();

        for descendant in descendants {
            let mut selector_results = self.apply_selector_to_value(&descendant, selector)?;
            results.append(&mut selector_results);
        }

        Ok(results)
    }

    /// Apply multiple selectors recursively to descendants
    pub fn apply_selectors_recursively(
        &self,
        json: &Value,
        selectors: &[JsonSelector],
    ) -> JsonPathResult<Vec<Value>> {
        if selectors.is_empty() {
            return Ok(vec![json.clone()]);
        }

        let mut current_values = vec![json.clone()];

        for selector in selectors {
            let mut next_values = Vec::new();

            for value in current_values {
                let mut selector_results = self.apply_selector_to_value(&value, selector)?;
                next_values.append(&mut selector_results);
            }

            current_values = next_values;
        }

        Ok(current_values)
    }
}
