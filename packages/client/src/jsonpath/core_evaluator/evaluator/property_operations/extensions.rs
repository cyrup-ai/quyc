//! Extension traits for property operations
//!
//! Extends `CoreJsonPathEvaluator` with property operation methods.

use serde_json::Value;

use super::super::core_types::{CoreJsonPathEvaluator, JsonPathResult};
use super::core::PropertyOperations;

/// Extension trait for `CoreJsonPathEvaluator` to add property operations
impl CoreJsonPathEvaluator {
    /// Evaluate a property path on a JSON value (for nested property access)
    pub fn evaluate_property_path(&self, json: &Value, path: &str) -> JsonPathResult<Vec<Value>> {
        PropertyOperations::evaluate_property_path(json, path)
    }

    /// Find property recursively in JSON structure
    #[must_use] 
    pub fn find_property_recursive(&self, json: &Value, property: &str) -> Vec<Value> {
        PropertyOperations::find_property_recursive(json, property)
    }
}
