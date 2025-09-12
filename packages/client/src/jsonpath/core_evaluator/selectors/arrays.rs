//! Array-specific selector operations
//!
//! Handles Index and Slice selectors for array access with support for
//! negative indexing, `from_end` indexing, and slice operations with step.

use serde_json::Value;

use super::super::evaluator::CoreJsonPathEvaluator;
use crate::jsonpath::error::JsonPathError;

type JsonPathResult<T> = Result<T, JsonPathError>;

/// Apply slice selector to array with proper bounds checking
///
/// # Errors
/// Returns `JsonPathError` if:
/// - Value is not an array when array slice is attempted
/// - Slice parameters are invalid (e.g., step is zero)
/// - Index calculations overflow or produce invalid ranges
/// - Memory limits are exceeded while collecting slice results
#[allow(clippy::cast_possible_truncation)]
pub fn apply_slice_to_array(
    array: &Value,
    start: Option<i64>,
    end: Option<i64>,
    step: Option<i64>,
) -> JsonPathResult<Vec<Value>> {
    match array {
        Value::Array(arr) => {
            let len = i64::try_from(arr.len()).unwrap_or(i64::MAX);
            let step = step.unwrap_or(1);

            if step == 0 {
                return Err(JsonPathError::invalid_expression(
                    "slice",
                    "step cannot be zero",
                    None,
                ));
            }

            let mut results = Vec::new();

            // Handle slice bounds
            let start_idx = start.unwrap_or(if step > 0 { 0 } else { len - 1 });
            let end_idx = end.unwrap_or(if step > 0 { len } else { -1 });

            // Normalize negative indices
            let start_norm = if start_idx < 0 {
                len + start_idx
            } else {
                start_idx
            };
            let end_norm = if end_idx < 0 { len + end_idx } else { end_idx };

            // Apply slice with step
            if step > 0 {
                let mut i = start_norm;
                while i < end_norm && i < len {
                    if i >= 0 {
                        // Safe conversion: i >= 0 check ensures non-negative value
                        if let Ok(i_usize) = usize::try_from(i)
                            && i_usize < arr.len() {
                                results.push(arr[i_usize].clone());
                            }
                    }
                    i += step;
                }
            } else {
                let mut i = start_norm;
                while i > end_norm && i >= 0 {
                    if i < len
                        && let Ok(idx) = usize::try_from(i)
                            && idx < arr.len() {
                                results.push(arr[idx].clone());
                            }
                    i += step;
                }
            }

            Ok(results)
        }
        _ => Ok(Vec::new()),
    }
}

/// Apply index selector for owned results
#[allow(clippy::cast_possible_truncation)]
pub fn apply_index_selector_owned(
    _evaluator: &CoreJsonPathEvaluator,
    value: &Value,
    index: i64,
    from_end: bool,
    results: &mut Vec<Value>,
) {
    if let Value::Array(arr) = value {
        let actual_index = if from_end && index < 0 {
            // Negative index - count from end (e.g., -1 means last element)
            let Ok(abs_index) = usize::try_from(-index) else {
                return; // Skip if conversion fails
            };
            if abs_index <= arr.len() && abs_index > 0 {
                arr.len() - abs_index
            } else {
                return; // Index out of bounds
            }
        } else if from_end && index > 0 {
            // Positive from_end index
            let Ok(index_usize) = usize::try_from(index) else {
                return; // Skip if conversion fails
            };
            if index_usize <= arr.len() {
                arr.len() - index_usize
            } else {
                return; // Index out of bounds
            }
        } else {
            // Regular positive index
            match usize::try_from(index) {
                Ok(idx) => idx,
                Err(_) => return, // Skip if conversion fails
            }
        };

        if actual_index < arr.len() {
            results.push(arr[actual_index].clone());
        }
    }
}

impl CoreJsonPathEvaluator {
    /// Apply slice to array with start, end, step parameters
    ///
    /// # Errors
    /// Returns `JsonPathError` if:
    /// - Slice parameters are invalid (e.g., step is zero)
    /// - Index calculations overflow or produce invalid ranges
    /// - Memory limits are exceeded while collecting slice results
    #[allow(clippy::cast_possible_truncation)]
    pub fn apply_slice_to_array(
        &self,
        arr: &[Value],
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    ) -> JsonPathResult<Vec<Value>> {
        let len = i64::try_from(arr.len()).unwrap_or(i64::MAX);
        let step = step.unwrap_or(1);

        if step == 0 {
            return Ok(vec![]); // Invalid step
        }

        let start = start.unwrap_or(if step > 0 { 0 } else { len - 1 });
        let end = end.unwrap_or(if step > 0 { len } else { -1 });

        // Normalize negative indices
        let start = if start < 0 {
            (len + start).max(0)
        } else {
            start.min(len)
        };
        let end = if end < 0 {
            (len + end).max(-1)
        } else {
            end.min(len)
        };

        let mut results = Vec::new();

        if step > 0 {
            let mut i = start;
            while i < end {
                if i >= 0 {
                    // Safe conversion: i >= 0 check ensures non-negative value
                    if let Ok(i_usize) = usize::try_from(i)
                        && i_usize < arr.len() {
                            results.push(arr[i_usize].clone());
                        }
                }
                i += step;
            }
        } else {
            let mut i = start;
            while i > end {
                if i >= 0 {
                    // Safe conversion: i >= 0 check ensures non-negative value
                    if let Ok(i_usize) = usize::try_from(i)
                        && i_usize < arr.len() {
                            results.push(arr[i_usize].clone());
                        }
                }
                i += step;
            }
        }

        Ok(results)
    }

    /// Apply index selector for array access
    #[allow(clippy::cast_possible_truncation)]
    pub fn apply_index_selector<'a>(
        &self,
        node: &'a Value,
        index: i64,
        from_end: bool,
        results: &mut Vec<&'a Value>,
    ) {
        if let Value::Array(arr) = node {
            let actual_index = if from_end && index < 0 {
                // Negative index - count from end (e.g., -1 means last element)
                let Ok(abs_index) = usize::try_from(-index) else {
                    return; // Skip if conversion fails
                };
                if abs_index <= arr.len() && abs_index > 0 {
                    arr.len() - abs_index
                } else {
                    return; // Index out of bounds
                }
            } else if from_end && index > 0 {
                // Positive from_end index
                let Ok(index_usize) = usize::try_from(index) else {
                    return; // Skip if conversion fails
                };
                if index_usize <= arr.len() {
                    arr.len() - index_usize
                } else {
                    return; // Index out of bounds
                }
            } else {
                // Regular positive index
                match usize::try_from(index) {
                    Ok(idx) => idx,
                    Err(_) => return, // Skip if conversion fails
                }
            };

            if actual_index < arr.len() {
                results.push(&arr[actual_index]);
            }
        }
    }

    /// Apply slice selector with start, end, step parameters for arrays
    #[allow(clippy::cast_possible_truncation)]
    pub fn apply_slice_selector_with_params<'a>(
        &self,
        node: &'a Value,
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
        results: &mut Vec<&'a Value>,
    ) {
        if let Value::Array(arr) = node {
            let len = i64::try_from(arr.len()).unwrap_or(i64::MAX);
            let step = step.unwrap_or(1);

            if step == 0 {
                return; // Invalid step
            }

            let start = start.unwrap_or(if step > 0 { 0 } else { len - 1 });
            let end = end.unwrap_or(if step > 0 { len } else { -1 });

            // Normalize negative indices
            let start = if start < 0 {
                (len + start).max(0)
            } else {
                start.min(len)
            };
            let end = if end < 0 {
                (len + end).max(-1)
            } else {
                end.min(len)
            };

            if step > 0 {
                let mut i = start;
                while i < end {
                    if i >= 0 {
                        // Safe conversion: i >= 0 check ensures non-negative value
                        if let Ok(i_usize) = usize::try_from(i)
                            && i_usize < arr.len() {
                                results.push(&arr[i_usize]);
                            }
                    }
                    i += step;
                }
            } else {
                let mut i = start;
                while i > end {
                    if i >= 0 {
                        // Safe conversion: i >= 0 check ensures non-negative value
                        if let Ok(i_usize) = usize::try_from(i)
                            && i_usize < arr.len() {
                                results.push(&arr[i_usize]);
                            }
                    }
                    i += step;
                }
            }
        }
    }
}
