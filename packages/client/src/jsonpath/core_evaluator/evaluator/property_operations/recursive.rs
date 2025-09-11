//! Recursive property operations
//!
//! Recursive property finding and pattern matching operations.

use serde_json::Value;

use super::core::PropertyOperations;

impl PropertyOperations {
    /// Find property recursively in JSON structure
    #[must_use] 
    pub fn find_property_recursive(json: &Value, property: &str) -> Vec<Value> {
        let mut results = Vec::new();
        Self::find_property_recursive_impl(json, property, &mut results);
        results
    }

    /// Internal implementation for recursive property finding
    fn find_property_recursive_impl(json: &Value, property: &str, results: &mut Vec<Value>) {
        match json {
            Value::Object(obj) => {
                // Check if this object has the property
                if let Some(value) = obj.get(property) {
                    results.push(value.clone());
                }
                // Recurse into all values
                for value in obj.values() {
                    Self::find_property_recursive_impl(value, property, results);
                }
            }
            Value::Array(arr) => {
                // Recurse into all array elements
                for value in arr {
                    Self::find_property_recursive_impl(value, property, results);
                }
            }
            _ => {
                // Leaf values - nothing to do
            }
        }
    }

    /// Check if a property exists at any depth
    #[must_use] 
    pub fn has_property_recursive(json: &Value, property: &str) -> bool {
        match json {
            Value::Object(obj) => {
                if obj.contains_key(property) {
                    return true;
                }
                // Check recursively in all values
                for value in obj.values() {
                    if Self::has_property_recursive(value, property) {
                        return true;
                    }
                }
                false
            }
            Value::Array(arr) => {
                // Check recursively in all array elements
                arr.iter()
                    .any(|value| Self::has_property_recursive(value, property))
            }
            _ => false,
        }
    }

    /// Count occurrences of a property at any depth
    #[must_use] 
    pub fn count_property_occurrences(json: &Value, property: &str) -> usize {
        let results = Self::find_property_recursive(json, property);
        results.len()
    }
}
