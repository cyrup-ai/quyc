//! Pattern-based property matching and wildcard support
//!
//! Contains methods for finding properties based on patterns and wildcards,
//! enabling flexible property discovery in JSON structures.

use serde_json::Value;

use super::core::PropertyOperations;

impl PropertyOperations {
    /// Find all properties matching a pattern
    pub fn find_properties_matching(json: &Value, pattern: &str) -> Vec<(String, Value)> {
        let mut results = Vec::new();
        Self::find_properties_matching_impl(json, pattern, "", &mut results);
        results
    }

    /// Internal implementation for pattern-based property finding
    fn find_properties_matching_impl(
        json: &Value,
        pattern: &str,
        current_path: &str,
        results: &mut Vec<(String, Value)>,
    ) {
        match json {
            Value::Object(obj) => {
                for (key, value) in obj {
                    let new_path = if current_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", current_path, key)
                    };

                    // Check if key matches pattern (simple wildcard support)
                    if Self::matches_pattern(key, pattern) {
                        results.push((new_path.clone(), value.clone()));
                    }

                    // Recurse into nested structures
                    Self::find_properties_matching_impl(value, pattern, &new_path, results);
                }
            }
            Value::Array(arr) => {
                for (index, value) in arr.iter().enumerate() {
                    let new_path = if current_path.is_empty() {
                        format!("[{}]", index)
                    } else {
                        format!("{}[{}]", current_path, index)
                    };

                    // Recurse into array elements
                    Self::find_properties_matching_impl(value, pattern, &new_path, results);
                }
            }
            _ => {
                // Leaf values - nothing to do
            }
        }
    }

    /// Simple pattern matching with wildcard support
    pub fn matches_pattern(text: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.contains('*') {
            // Simple wildcard matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return text.starts_with(prefix) && text.ends_with(suffix);
            }
        }

        text == pattern
    }
}
