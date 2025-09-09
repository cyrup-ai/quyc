//! Filter support utilities for JSONPath evaluation
//!
//! This module provides helper functions for filter evaluation and context management.

use std::collections::HashSet;

use serde_json::Value;

/// Support utilities for filter evaluation
pub struct FilterSupport;

impl FilterSupport {
    /// Collect all property names that exist across items in an array
    /// This provides context for filter evaluation
    pub fn collect_existing_properties(arr: &[Value]) -> std::collections::HashSet<String> {
        let mut properties = HashSet::new();

        for item in arr {
            if let Value::Object(obj) = item {
                for key in obj.keys() {
                    properties.insert(key.clone());
                }
            }
        }

        properties
    }

    /// Collect property names from a single object
    pub fn collect_object_properties(obj: &Value) -> HashSet<String> {
        let mut properties = HashSet::new();

        if let Value::Object(map) = obj {
            for key in map.keys() {
                properties.insert(key.clone());
            }
        }

        properties
    }

    /// Check if a property exists in a JSON object
    pub fn has_property(obj: &Value, property: &str) -> bool {
        match obj {
            Value::Object(map) => map.contains_key(property),
            _ => false,
        }
    }

    /// Get property value safely
    pub fn get_property<'a>(obj: &'a Value, property: &str) -> Option<&'a Value> {
        match obj {
            Value::Object(map) => map.get(property),
            _ => None,
        }
    }

    /// Check if an array contains objects with specific properties
    pub fn array_has_objects_with_property(arr: &[Value], property: &str) -> bool {
        arr.iter().any(|item| Self::has_property(item, property))
    }

    /// Count objects in array that have a specific property
    pub fn count_objects_with_property(arr: &[Value], property: &str) -> usize {
        arr.iter()
            .filter(|item| Self::has_property(item, property))
            .count()
    }

    /// Get all unique values for a property across array items
    pub fn collect_property_values(arr: &[Value], property: &str) -> Vec<Value> {
        let mut values = Vec::new();
        let mut seen = HashSet::new();

        for item in arr {
            if let Some(value) = Self::get_property(item, property) {
                let value_str = value.to_string();
                if !seen.contains(&value_str) {
                    seen.insert(value_str);
                    values.push(value.clone());
                }
            }
        }

        values
    }

    /// Check if a value matches a type pattern
    pub fn matches_type(value: &Value, type_name: &str) -> bool {
        match type_name.to_lowercase().as_str() {
            "null" => value.is_null(),
            "boolean" | "bool" => value.is_boolean(),
            "number" => value.is_number(),
            "string" => value.is_string(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            _ => false,
        }
    }

    /// Get the JSON type name for a value
    pub fn get_type_name(value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    /// Check if a value is considered "truthy" in filter context
    pub fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_f64().map_or(false, |f| f != 0.0),
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
        }
    }

    /// Compare two values for filter operations
    pub fn compare_values(left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                let a_f64 = a.as_f64()?;
                let b_f64 = b.as_f64()?;
                a_f64.partial_cmp(&b_f64)
            }
            (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
            (Value::Bool(a), Value::Bool(b)) => Some(a.cmp(b)),
            _ => None, // Cannot compare different types
        }
    }

    /// Check if a value contains another value (for arrays and objects)
    pub fn contains_value(container: &Value, target: &Value) -> bool {
        match container {
            Value::Array(arr) => arr.contains(target),
            Value::Object(obj) => obj.values().any(|v| v == target),
            Value::String(s) => {
                if let Value::String(target_str) = target {
                    s.contains(target_str)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}


