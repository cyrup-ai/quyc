//! Filter expression evaluation for `JSONPath`
//!
//! Contains methods for evaluating filter expressions on JSON values.

use serde_json::Value;

use super::engine::{CoreJsonPathEvaluator, JsonPathResult};

impl CoreJsonPathEvaluator {
    /// Apply filter expression to a value
    pub(crate) fn apply_filter_expression(
        &self,
        value: &Value,
        filter_expr: &str,
    ) -> JsonPathResult<Vec<Value>> {
        // Simple filter implementation for basic expressions
        match value {
            Value::Array(arr) => {
                let mut results = Vec::new();
                for (index, item) in arr.iter().enumerate() {
                    if self.evaluate_filter_on_item(item, filter_expr, index)? {
                        results.push(item.clone());
                    }
                }
                Ok(results)
            }
            Value::Object(obj) => {
                let mut results = Vec::new();
                for (key, item) in obj {
                    if self.evaluate_filter_on_object_item(item, filter_expr, key)? {
                        results.push(item.clone());
                    }
                }
                Ok(results)
            }
            _ => Ok(vec![]), // Primitives can't be filtered
        }
    }

    /// Evaluate filter expression on array item
    fn evaluate_filter_on_item(
        &self,
        item: &Value,
        filter_expr: &str,
        _index: usize,
    ) -> JsonPathResult<bool> {
        // Basic filter evaluation - can be extended for more complex expressions
        if filter_expr.contains("@.") {
            // Property-based filter
            let property_name = filter_expr.trim_start_matches("@.").trim();
            if let Value::Object(obj) = item {
                return Ok(obj.contains_key(property_name));
            }
        } else if filter_expr.contains("==") {
            // Equality filter
            let parts: Vec<&str> = filter_expr.split("==").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim().trim_matches('"').trim_matches('\'');

                if left == "@" {
                    // Compare entire item
                    return Ok(item.as_str() == Some(right));
                } else if left.starts_with("@.") {
                    // Compare property
                    let prop_name = left.trim_start_matches("@.");
                    if let Value::Object(obj) = item
                        && let Some(prop_value) = obj.get(prop_name) {
                            return Ok(prop_value.as_str() == Some(right));
                        }
                }
            }
        }

        // Default: no match
        Ok(false)
    }

    /// Evaluate filter expression on object item  
    fn evaluate_filter_on_object_item(
        &self,
        item: &Value,
        filter_expr: &str,
        key: &str,
    ) -> JsonPathResult<bool> {
        // Object filter evaluation with proper key context
        if filter_expr.contains("@.") {
            let property_name = filter_expr.trim_start_matches("@.").trim();
            if property_name == key {
                return Ok(true);
            }
            if let Value::Object(obj) = item {
                return Ok(obj.contains_key(property_name));
            }
        }
        // Fallback to item evaluation for other expressions
        self.evaluate_filter_on_item(item, filter_expr, 0)
    }
}
