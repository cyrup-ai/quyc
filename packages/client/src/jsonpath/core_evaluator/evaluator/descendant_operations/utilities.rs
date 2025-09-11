//! Utility operations for descendant processing
//!
//! Counting, depth calculation, and other utility functions for descendant analysis.

use serde_json::Value;

use super::core::DescendantOperations;

impl DescendantOperations {
    /// Count total number of descendants
    #[must_use] 
    pub fn count_descendants(node: &Value) -> usize {
        match node {
            Value::Object(obj) => obj.values().map(|v| 1 + Self::count_descendants(v)).sum(),
            Value::Array(arr) => arr.iter().map(|v| 1 + Self::count_descendants(v)).sum(),
            _ => 0,
        }
    }

    /// Get maximum depth of descendants
    #[must_use] 
    pub fn max_descendant_depth(node: &Value) -> usize {
        match node {
            Value::Object(obj) => obj
                .values()
                .map(|v| 1 + Self::max_descendant_depth(v))
                .max()
                .unwrap_or(0),
            Value::Array(arr) => arr
                .iter()
                .map(|v| 1 + Self::max_descendant_depth(v))
                .max()
                .unwrap_or(0),
            _ => 0,
        }
    }
}
