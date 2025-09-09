//! Utility methods for property analysis and manipulation
//!
//! Contains helper methods for property name extraction and other
//! utility functions for working with JSON object properties.

use serde_json::Value;

use super::core::PropertyOperations;

impl PropertyOperations {
    /// Get all property names from an object
    pub fn get_property_names(json: &Value) -> Vec<String> {
        match json {
            Value::Object(obj) => obj.keys().cloned().collect(),
            _ => Vec::new(),
        }
    }
}
