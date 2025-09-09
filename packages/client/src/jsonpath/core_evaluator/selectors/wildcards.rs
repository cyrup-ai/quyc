//! Wildcard selector operations
//!
//! Handles wildcard (*) selector application for both objects and arrays
//! with performance limits and owned/reference-based result collection.

use serde_json::Value;

use super::super::evaluator::CoreJsonPathEvaluator;

/// Apply wildcard selector with result limit for performance - owned version
pub fn apply_wildcard_owned(
    _evaluator: &CoreJsonPathEvaluator,
    value: &Value,
    results: &mut Vec<Value>,
    max_results: usize,
) {
    match value {
        Value::Object(obj) => {
            // Wildcard on object returns all object values
            for child_value in obj.values() {
                if results.len() >= max_results {
                    break;
                }
                results.push(child_value.clone());
            }
        }
        Value::Array(arr) => {
            // Wildcard on array returns all array elements
            for child_value in arr {
                if results.len() >= max_results {
                    break;
                }
                results.push(child_value.clone());
            }
        }
        _ => {
            // Primitives have no children - wildcard returns nothing
        }
    }
}

impl CoreJsonPathEvaluator {
    /// Apply wildcard selector to get all children
    pub fn apply_wildcard_selector<'a>(&self, node: &'a Value, results: &mut Vec<&'a Value>) {
        match node {
            Value::Object(obj) => {
                for value in obj.values() {
                    results.push(value);
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    results.push(value);
                }
            }
            _ => {}
        }
    }
}
