//! Utility functions for filter expression evaluation
//!
//! Contains helper functions for truthiness testing and other common
//! operations used throughout the filter evaluation system.

use crate::jsonpath::parser::FilterValue;

/// Filter utility functions
pub struct FilterUtils;

impl FilterUtils {
    /// Check if a filter value is "truthy" for boolean context
    #[inline]
    #[must_use] 
    pub fn is_truthy(value: &FilterValue) -> bool {
        match value {
            FilterValue::Boolean(b) => *b,
            FilterValue::Integer(i) => *i != 0,
            FilterValue::Number(f) => *f != 0.0 && !f.is_nan(),
            FilterValue::String(s) => !s.is_empty(),
            FilterValue::Null | FilterValue::Missing => false,
        }
    }
}
