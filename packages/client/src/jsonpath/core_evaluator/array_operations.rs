//! Array operations for `JSONPath` evaluation
//!
//! This module handles array indexing, slicing, and related operations.

use serde_json::Value;

use crate::jsonpath::error::JsonPathError;

type JsonPathResult<T> = Result<T, JsonPathError>;

/// Array operations engine for `JSONPath` evaluation
pub struct ArrayOperations;

impl ArrayOperations {
    /// Apply index selector for array access
    ///
    /// # Errors
    /// 
    /// Returns `JsonPathError` if:
    /// - Index conversion fails due to overflow or underflow conditions
    /// - Array access violates bounds checking or safety constraints
    /// - Internal value processing encounters type conversion errors
    pub fn apply_index(arr: &[Value], index: i64, from_end: bool) -> JsonPathResult<Vec<Value>> {
        let mut results = Vec::new();

        let actual_index = if from_end && index < 0 {
            // Negative index - count from end (e.g., -1 means last element)
            let Ok(abs_index) = usize::try_from(-index) else {
                return Ok(results); // Index too large for usize
            };
            if abs_index <= arr.len() && abs_index > 0 {
                arr.len() - abs_index
            } else {
                return Ok(results); // Index out of bounds
            }
        } else if from_end && index > 0 {
            // Positive from_end index
            let Ok(index_usize) = usize::try_from(index) else {
                return Ok(results); // Index too large for usize
            };
            if index_usize <= arr.len() {
                arr.len() - index_usize
            } else {
                return Ok(results); // Index out of bounds
            }
        } else if index >= 0 {
            // Regular positive index - safe conversion with bounds check
            match usize::try_from(index) {
                Ok(idx) => idx,
                Err(_) => return Ok(results), // Index too large for usize
            }
        } else {
            // Negative index in non-from_end context is invalid
            return Ok(results);
        };

        if actual_index < arr.len() {
            results.push(arr[actual_index].clone());
        }

        Ok(results)
    }

    /// Apply slice selector for array slicing
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError::invalid_index` if:
    /// - `step` is zero (division by zero not allowed)
    pub fn apply_slice(
        arr: &[Value],
        start: Option<i64>,
        end: Option<i64>,
        step: i64,
    ) -> JsonPathResult<Vec<Value>> {
        if step == 0 {
            return Err(JsonPathError::invalid_index(
                "Step cannot be zero".to_string(),
            ));
        }

        let len = i64::try_from(arr.len()).unwrap_or(i64::MAX);
        let mut results = Vec::new();

        // Normalize start and end indices
        let start_idx = Self::normalize_index(start, len, step > 0);
        let end_idx = Self::normalize_index(end, len, step > 0);

        if step > 0 {
            // Forward iteration
            let mut i = start_idx;
            while i < end_idx && i < len {
                if i >= 0 {
                    // Safe cast: i is guaranteed >= 0 by the if condition and within bounds
                    if let Ok(idx) = usize::try_from(i) {
                        results.push(arr[idx].clone());
                    }
                }
                i += step;
            }
        } else {
            // Backward iteration
            let mut i = start_idx;
            while i > end_idx && i >= 0 {
                if i < len {
                    // Safe cast: i is guaranteed >= 0 by outer while condition and within bounds
                    if let Ok(idx) = usize::try_from(i) {
                        results.push(arr[idx].clone());
                    }
                }
                i += step; // step is negative
            }
        }

        Ok(results)
    }

    /// Normalize slice index according to Python-like semantics
    fn normalize_index(index: Option<i64>, len: i64, forward: bool) -> i64 {
        match index {
            Some(idx) => {
                if idx < 0 {
                    std::cmp::max(0, len + idx)
                } else {
                    std::cmp::min(idx, len)
                }
            }
            None => {
                if forward {
                    0 // Start from beginning for forward iteration
                } else {
                    len - 1 // Start from end for backward iteration
                }
            }
        }
    }

    /// Check if an array index is valid
    #[must_use] 
    pub fn is_valid_index(arr: &[Value], index: i64, from_end: bool) -> bool {
        if from_end && index < 0 {
            // Safe cast: -index is guaranteed positive since index < 0
            match usize::try_from(-index) {
                Ok(abs_index) => abs_index <= arr.len() && abs_index > 0,
                Err(_) => false, // Index too large for usize
            }
        } else if from_end && index > 0 {
            // Safe cast: index is guaranteed > 0 by condition
            match usize::try_from(index) {
                Ok(idx) => idx <= arr.len(),
                Err(_) => false, // Index too large for usize
            }
        } else {
            // Safe cast: index >= 0 check ensures non-negative value
            if index >= 0 {
                match usize::try_from(index) {
                    Ok(idx) => idx < arr.len(),
                    Err(_) => false, // Index too large for usize
                }
            } else {
                false
            }
        }
    }

    /// Get array length safely
    #[must_use] 
    pub fn safe_len(value: &Value) -> Option<usize> {
        match value {
            Value::Array(arr) => Some(arr.len()),
            _ => None,
        }
    }

    /// Check if a value is an array
    #[must_use] 
    pub fn is_array(value: &Value) -> bool {
        matches!(value, Value::Array(_))
    }

    /// Get array element safely
    #[must_use] 
    pub fn get_element(arr: &[Value], index: usize) -> Option<&Value> {
        arr.get(index)
    }

    /// Apply multiple indices to an array
    ///
    /// # Errors
    ///
    /// This function currently never returns an error, always returns `Ok(results)`
    pub fn apply_multiple_indices(arr: &[Value], indices: &[i64]) -> JsonPathResult<Vec<Value>> {
        let mut results = Vec::new();

        for &index in indices {
            if index >= 0
                && let Ok(index_usize) = usize::try_from(index)
                    && index_usize < arr.len() {
                        results.push(arr[index_usize].clone());
                    }
        }

        Ok(results)
    }

    /// Apply range operation (start:end)
    ///
    /// # Errors
    ///
    /// Returns `JsonPathError::invalid_index` if the underlying slice operation fails
    /// (though this is unlikely since step is fixed at 1)
    pub fn apply_range(arr: &[Value], start: i64, end: i64) -> JsonPathResult<Vec<Value>> {
        Self::apply_slice(arr, Some(start), Some(end), 1)
    }

    /// Get last N elements from array
    #[must_use] 
    pub fn get_last_n(arr: &[Value], n: usize) -> Vec<Value> {
        if n >= arr.len() {
            arr.to_vec()
        } else {
            arr[arr.len() - n..].to_vec()
        }
    }

    /// Get first N elements from array
    #[must_use] 
    pub fn get_first_n(arr: &[Value], n: usize) -> Vec<Value> {
        if n >= arr.len() {
            arr.to_vec()
        } else {
            arr[..n].to_vec()
        }
    }
}

// Tests moved to /tests/jsonpath/core_evaluator/array_operations_tests.rs
