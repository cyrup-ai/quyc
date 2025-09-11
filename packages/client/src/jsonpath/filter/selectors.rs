//! `JSONPath` selector evaluation for filter expressions
//!
//! Handles evaluation of complex `JSONPath` selectors within filter contexts,
//! including child access, wildcards, and array indexing.

use super::property::PropertyResolver;
use crate::jsonpath::error::JsonPathResult;
use crate::jsonpath::parser::{FilterValue, JsonSelector};

/// `JSONPath` selector evaluation utilities
pub struct SelectorEvaluator;

impl SelectorEvaluator {
    /// Evaluate complex `JSONPath` selectors relative to current context
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn evaluate_jsonpath_selectors(
        context: &serde_json::Value,
        selectors: &[JsonSelector],
    ) -> JsonPathResult<FilterValue> {
        let mut current = context;

        for selector in selectors {
            match selector {
                JsonSelector::Child { name, .. } => {
                    if let Some(obj) = current.as_object() {
                        if let Some(value) = obj.get(name) {
                            current = value;
                        } else {
                            return Ok(FilterValue::Missing);
                        }
                    } else {
                        return Ok(FilterValue::Missing);
                    }
                }
                JsonSelector::Wildcard => {
                    // For wildcard, return the array itself converted to a suitable representation
                    if current.is_array() {
                        return Ok(PropertyResolver::json_value_to_filter_value(current));
                    } 
                    return Ok(FilterValue::Missing);
                }
                JsonSelector::Index { index, from_end } => {
                    if let Some(arr) = current.as_array() {
                        let actual_index = if *from_end {
                            arr.len().saturating_sub((*index).unsigned_abs() as usize)
                        } else {
                            *index as usize
                        };

                        if let Some(value) = arr.get(actual_index) {
                            current = value;
                        } else {
                            return Ok(FilterValue::Missing);
                        }
                    } else {
                        return Ok(FilterValue::Missing);
                    }
                }
                _ => {
                    // For complex selectors, return the current value
                    return Ok(PropertyResolver::json_value_to_filter_value(current));
                }
            }
        }

        Ok(PropertyResolver::json_value_to_filter_value(current))
    }
}
