//! Selector application logic for `JSONPath` evaluation
//!
//! Contains methods for applying individual selectors to JSON values.

use serde_json::Value;

use super::engine::{CoreJsonPathEvaluator, JsonPathResult};
use crate::jsonpath::parser::JsonSelector;

impl CoreJsonPathEvaluator {
    /// Apply a single selector to a JSON value
    pub fn apply_selector_to_value(
        &self,
        value: &Value,
        selector: &JsonSelector,
    ) -> JsonPathResult<Vec<Value>> {
        match selector {
            JsonSelector::Root => {
                // Root selector - return the value as-is
                Ok(vec![value.clone()])
            }
            JsonSelector::Child { name, .. } => {
                // Property access - get named property from object
                match value {
                    Value::Object(obj) => {
                        if let Some(prop_value) = obj.get(name) {
                            Ok(vec![prop_value.clone()])
                        } else {
                            Ok(vec![])
                        }
                    }
                    _ => Ok(vec![]), // Non-objects don't have properties
                }
            }
            JsonSelector::Index { index, from_end: _ } => {
                // Array index access
                match value {
                    Value::Array(arr) => {
                        let len = arr.len() as i64;
                        let actual_index = if index < &0 {
                            // Negative indexing from end
                            len + index
                        } else {
                            *index
                        };

                        if actual_index >= 0 && (actual_index as usize) < arr.len() {
                            // Safe cast: actual_index >= 0 check ensures non-negative value
                            Ok(vec![arr[actual_index as usize].clone()])
                        } else {
                            Ok(vec![])
                        }
                    }
                    _ => Ok(vec![]), // Non-arrays don't have indices
                }
            }
            JsonSelector::Slice { start, end, step } => {
                // Array slice access
                match value {
                    Value::Array(arr) => {
                        let len = arr.len() as i64;
                        let mut results = Vec::new();

                        // Normalize slice parameters
                        let start_idx = start.unwrap_or(0);
                        let end_idx = end.unwrap_or(len);
                        let step_size = step.unwrap_or(1);

                        if step_size == 0 {
                            return Err(crate::jsonpath::error::invalid_expression_error(
                                &self.expression,
                                "slice step cannot be zero",
                                None,
                            ));
                        }

                        let mut current = start_idx;
                        while (step_size > 0 && current < end_idx && current < len)
                            || (step_size < 0 && current > end_idx && current >= 0)
                        {
                            if current >= 0 && (current as usize) < arr.len() {
                                results.push(arr[current as usize].clone());
                            }
                            current += step_size;
                        }

                        Ok(results)
                    }
                    _ => Ok(vec![]), // Non-arrays don't support slicing
                }
            }
            JsonSelector::Wildcard => {
                // Wildcard selector - return all values
                match value {
                    Value::Object(obj) => Ok(obj.values().cloned().collect()),
                    Value::Array(arr) => Ok(arr.clone()),
                    _ => Ok(vec![]), // Primitives have no children
                }
            }
            JsonSelector::RecursiveDescent => {
                // This should be handled at a higher level
                let results = self.collect_descendants(value);
                Ok(results)
            }
            JsonSelector::Filter { expression } => {
                // Filter expression - apply filter to current context
                self.apply_filter_expression(value, &format!("{expression:?}"))
            }
            JsonSelector::Union { selectors } => {
                // Union of multiple selectors
                let mut results = Vec::new();
                for sub_selector in selectors {
                    let sub_results = self.apply_selector_to_value(value, sub_selector)?;
                    results.extend(sub_results);
                }
                Ok(results)
            }
        }
    }
}
