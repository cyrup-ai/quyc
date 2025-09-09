//! Property access operations for JSONPath evaluation
//!
//! Contains methods for property path evaluation and recursive property finding.

use serde_json::Value;

use super::core_evaluator::{CoreJsonPathEvaluator, JsonPathResult};

impl CoreJsonPathEvaluator {
    /// Evaluate a property path on a JSON value (for nested property access)
    pub fn evaluate_property_path(&self, json: &Value, path: &str) -> JsonPathResult<Vec<Value>> {
        // Handle simple property access for now
        let properties: Vec<&str> = path.split('.').collect();
        let mut current = vec![json.clone()];

        for property in properties {
            if property.is_empty() {
                continue;
            }

            let mut next = Vec::new();
            for value in current {
                if let Value::Object(obj) = value {
                    if let Some(prop_value) = obj.get(property) {
                        next.push(prop_value.clone());
                    }
                }
            }
            current = next;
        }

        Ok(current)
    }

    /// Find property recursively in JSON structure
    pub fn find_property_recursive(&self, json: &Value, property: &str) -> Vec<Value> {
        let mut results = Vec::new();
        self.find_property_recursive_impl(json, property, &mut results);
        results
    }

    fn find_property_recursive_impl(&self, json: &Value, property: &str, results: &mut Vec<Value>) {
        match json {
            Value::Object(obj) => {
                // Check if this object has the property
                if let Some(value) = obj.get(property) {
                    results.push(value.clone());
                }
                // Recurse into all values
                for value in obj.values() {
                    self.find_property_recursive_impl(value, property, results);
                }
            }
            Value::Array(arr) => {
                // Recurse into all array elements
                for value in arr {
                    self.find_property_recursive_impl(value, property, results);
                }
            }
            _ => {
                // Leaf values - nothing to do
            }
        }
    }
}
