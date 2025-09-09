//! Array selector processing for JSONPath expressions
//!
//! Handles array index evaluation, slicing, and filtering operations
//! during streaming JSON processing.

/// Array selector evaluation result
#[derive(Debug, Clone, PartialEq)]
pub enum ArraySelectorResult {
    /// Match found at specific index
    Match(usize),
    /// No match for current element
    NoMatch,
    /// Continue processing (wildcard or range)
    Continue,
}

/// Process array selector against current index
pub fn evaluate_array_selector(
    selector: &str,
    current_index: usize,
    array_length: Option<usize>,
) -> ArraySelectorResult {
    match selector {
        "*" => ArraySelectorResult::Continue,
        "-1" => {
            if let Some(len) = array_length {
                if current_index == len.saturating_sub(1) {
                    ArraySelectorResult::Match(current_index)
                } else {
                    ArraySelectorResult::NoMatch
                }
            } else {
                ArraySelectorResult::Continue
            }
        }
        index_str => {
            if let Ok(target_index) = index_str.parse::<usize>() {
                if current_index == target_index {
                    ArraySelectorResult::Match(current_index)
                } else {
                    ArraySelectorResult::NoMatch
                }
            } else {
                ArraySelectorResult::NoMatch
            }
        }
    }
}

/// Check if array index matches slice expression (e.g., "1:3", "::2")
pub fn matches_slice(slice_expr: &str, current_index: usize, _array_length: Option<usize>) -> bool {
    // Basic slice implementation - can be expanded for full slice syntax
    if slice_expr.contains(':') {
        let parts: Vec<&str> = slice_expr.split(':').collect();
        match parts.len() {
            2 => {
                let start = parts[0].parse::<usize>().unwrap_or(0);
                let end = parts[1].parse::<usize>().unwrap_or(usize::MAX);
                current_index >= start && current_index < end
            }
            _ => false,
        }
    } else {
        false
    }
}
