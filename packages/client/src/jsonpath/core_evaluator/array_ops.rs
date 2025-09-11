//! Array operations for `JSONPath` evaluation
//!
//! Specialized array handling: access, slicing, union selectors with zero-allocation patterns.

use serde_json::Value;

use super::evaluator::CoreJsonPathEvaluator;
use crate::jsonpath::error::JsonPathError;

type JsonPathResult<T> = Result<T, JsonPathError>;

impl CoreJsonPathEvaluator {
    /// Evaluate array access expressions with comprehensive selector support
    pub fn evaluate_array_access(&self, json: &Value, expr: &str) -> JsonPathResult<Vec<Value>> {
        // Core parsing for array access patterns
        if let Some(captures) = self.parse_array_expression(expr) {
            let (path, selector) = captures;

            // Navigate to the array - collect intermediate results to avoid lifetime issues
            if path == "$" {
                match json {
                    Value::Array(arr) => self.apply_array_selector(arr, &selector),
                    _ => Ok(vec![]),
                }
            } else if path.starts_with("$.") {
                let property_results = self.evaluate_property_path(json, &path[2..])?;
                if property_results.len() == 1 {
                    match &property_results[0] {
                        Value::Array(arr) => self.apply_array_selector(arr, &selector),
                        _ => Ok(vec![]),
                    }
                } else {
                    Ok(vec![])
                }
            } else if let Some(property) = path.strip_prefix("$..") {
                // Handle recursive descent to array
                let candidates = self.find_property_recursive(json, property);
                if candidates.len() == 1 {
                    match &candidates[0] {
                        Value::Array(arr) => self.apply_array_selector(arr, &selector),
                        _ => Ok(vec![]),
                    }
                } else {
                    Ok(vec![])
                }
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }

    /// Parse array expression into path and selector components
    #[must_use] 
    pub fn parse_array_expression(&self, expr: &str) -> Option<(String, String)> {
        if let Some(bracket_start) = expr.rfind('[')
            && let Some(bracket_end) = expr[bracket_start..].find(']') {
                let path = expr[..bracket_start].to_string();
                let selector = expr[bracket_start + 1..bracket_start + bracket_end].to_string();
                return Some((path, selector));
            }
        None
    }

    /// Apply array selector with comprehensive pattern matching
    pub fn apply_array_selector(
        &self,
        arr: &[Value],
        selector: &str,
    ) -> JsonPathResult<Vec<Value>> {
        if selector == "*" {
            // Wildcard - return all elements
            Ok(arr.to_vec())
        } else if let Ok(index) = selector.parse::<i64>() {
            // Index selector
            let actual_index = if index < 0 {
                // Negative index - count from end (e.g., -1 means last element)
                // Safe cast: -index is guaranteed positive since index < 0
                let abs_index = (-index) as usize;
                if abs_index <= arr.len() && abs_index > 0 {
                    arr.len() - abs_index
                } else {
                    return Ok(vec![]); // Index out of bounds
                }
            } else {
                // Safe cast: index >= 0 guaranteed by else branch
                index as usize
            };

            if actual_index < arr.len() {
                Ok(vec![arr[actual_index].clone()])
            } else {
                Ok(vec![])
            }
        } else if selector.contains(':') {
            // Slice selector
            self.apply_slice_selector(arr, selector)
        } else if selector.contains(',') {
            // Union selector
            self.apply_union_selector(arr, selector)
        } else {
            // Unsupported selector
            Ok(vec![])
        }
    }

    /// Apply slice selector with colon notation (start:end:step)
    pub fn apply_slice_selector(
        &self,
        arr: &[Value],
        selector: &str,
    ) -> JsonPathResult<Vec<Value>> {
        let parts: Vec<&str> = selector.split(':').collect();
        if parts.len() < 2 {
            return Ok(vec![]);
        }

        let start = if parts[0].is_empty() {
            0
        } else {
            parts[0].parse::<i64>().unwrap_or(0)
        };
        let end = if parts[1].is_empty() {
            arr.len() as i64
        } else {
            parts[1].parse::<i64>().unwrap_or(arr.len() as i64)
        };

        let start_idx = if start < 0 {
            // Safe cast: max(0) ensures non-negative result
            (arr.len() as i64 + start).max(0) as usize
        } else {
            // Safe cast: start >= 0 guaranteed by else branch
            start as usize
        };
        let end_idx = if end < 0 {
            // Safe cast: max(0) ensures non-negative result
            (arr.len() as i64 + end).max(0) as usize
        } else {
            // Safe cast: end >= 0 guaranteed by else branch
            (end as usize).min(arr.len())
        };

        if start_idx < end_idx {
            Ok(arr[start_idx..end_idx].to_vec())
        } else {
            Ok(vec![])
        }
    }

    /// Apply union selector with comma-separated indices
    pub fn apply_union_selector(
        &self,
        arr: &[Value],
        selector: &str,
    ) -> JsonPathResult<Vec<Value>> {
        let indices: Vec<&str> = selector.split(',').collect();
        let mut results = Vec::new();

        for idx_str in indices {
            let idx_str = idx_str.trim();
            if let Ok(index) = idx_str.parse::<i64>() {
                let actual_index = if index < 0 {
                    // Negative index - count from end (e.g., -1 means last element)
                    // Safe cast: -index is guaranteed positive since index < 0
                    let abs_index = (-index) as usize;
                    if abs_index <= arr.len() && abs_index > 0 {
                        arr.len() - abs_index
                    } else {
                        continue; // Skip out of bounds indices
                    }
                } else {
                    // Safe cast: index >= 0 guaranteed by else branch
                    index as usize
                };

                if actual_index < arr.len() {
                    results.push(arr[actual_index].clone());
                }
            }
        }

        Ok(results)
    }

    /// Evaluate wildcard selector for comprehensive collection
    pub fn evaluate_wildcard(&self, json: &Value) -> JsonPathResult<Vec<Value>> {
        match json {
            Value::Object(obj) => Ok(obj.values().cloned().collect()),
            Value::Array(arr) => Ok(arr.clone()),
            _ => Ok(vec![]),
        }
    }

    /// Collect all values recursively for comprehensive traversal
    #[must_use] 
    pub fn collect_all_values(&self, json: &Value) -> Vec<Value> {
        let mut results = Vec::new();
        self.collect_all_values_recursive(json, &mut results);
        results
    }

    /// Recursive implementation for value collection
    pub fn collect_all_values_recursive(&self, json: &Value, results: &mut Vec<Value>) {
        match json {
            Value::Object(obj) => {
                for value in obj.values() {
                    results.push(value.clone());
                    self.collect_all_values_recursive(value, results);
                }
            }
            Value::Array(arr) => {
                for value in arr {
                    results.push(value.clone());
                    self.collect_all_values_recursive(value, results);
                }
            }
            _ => {}
        }
    }

    /// Evaluate property with array wildcards (e.g., $.store.book[*].author)
    pub fn evaluate_property_with_array_wildcards(
        &self,
        json: &Value,
        expr: &str,
    ) -> JsonPathResult<Vec<Value>> {
        // Handle expressions like $.store.book[*].author

        // Split the expression into parts around [*]
        let parts: Vec<&str> = expr.split("[*]").collect();
        if parts.len() != 2 {
            return Ok(vec![]); // More complex patterns not supported yet
        }

        let before_wildcard = parts[0]; // "$.store.book"
        let after_wildcard = parts[1]; // ".author"

        // Navigate to the array location
        let array_value = if before_wildcard == "$" {
            json
        } else if before_wildcard.starts_with("$.") {
            let path_parts: Vec<&str> = before_wildcard[2..].split('.').collect();
            let mut current = json;

            for part in path_parts {
                match current {
                    Value::Object(obj) => {
                        if let Some(value) = obj.get(part) {
                            current = value;
                        } else {
                            return Ok(vec![]); // Property not found
                        }
                    }
                    _ => return Ok(vec![]), // Can't access property on non-object
                }
            }
            current
        } else {
            return Ok(vec![]);
        };

        // Apply wildcard to array and then continue with remaining path
        match array_value {
            Value::Array(arr) => {
                let mut results = Vec::new();
                for item in arr {
                    if after_wildcard.is_empty() {
                        // No property after wildcard, return the array item itself
                        results.push(item.clone());
                    } else if let Some(property_path) = after_wildcard.strip_prefix('.') {
                        // Property access after wildcard
                        // Remove leading dot
                        let property_results = self.evaluate_property_path(item, property_path)?;
                        results.extend(property_results);
                    }
                }
                Ok(results)
            }
            _ => Ok(vec![]), // Not an array
        }
    }
}
