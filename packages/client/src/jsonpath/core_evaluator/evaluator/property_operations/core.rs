//! Core property operations for JSONPath evaluation
//!
//! Basic property access and path evaluation operations.

use serde_json::Value;

use super::super::core_types::JsonPathResult;

/// Core operations for handling property access in JSONPath expressions
pub struct PropertyOperations;

impl PropertyOperations {
    /// Evaluate a property path on a JSON value (for nested property access)
    pub fn evaluate_property_path(json: &Value, path: &str) -> JsonPathResult<Vec<Value>> {
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

    /// Get property value with default fallback
    pub fn get_property_or_default(json: &Value, property: &str, default: Value) -> Value {
        match json {
            Value::Object(obj) => obj.get(property).cloned().unwrap_or(default),
            _ => default,
        }
    }
}
