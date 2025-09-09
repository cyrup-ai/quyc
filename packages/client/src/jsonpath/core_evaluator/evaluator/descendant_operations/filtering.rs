//! Filtering and predicate-based operations
//!
//! Contains methods for filtering descendants based on predicates
//! and conditional collection operations.

use serde_json::Value;

use super::core::DescendantOperations;

impl DescendantOperations {
    /// Filter descendants by predicate
    pub fn filter_descendants<F>(node: &Value, predicate: F, results: &mut Vec<Value>)
    where
        F: Fn(&Value) -> bool + Copy,
    {
        match node {
            Value::Object(obj) => {
                for value in obj.values() {
                    if predicate(value) {
                        results.push(value.clone());
                    }
                    Self::filter_descendants(value, predicate, results);
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    if predicate(value) {
                        results.push(value.clone());
                    }
                    Self::filter_descendants(value, predicate, results);
                }
            }
            _ => {}
        }
    }
}
