//! Array access operations with null vs missing distinction
//!
//! Implements array index access while maintaining null vs missing semantics.
//! Out-of-bounds access is considered missing, not null.

use serde_json::Value as JsonValue;

use super::property_access::PropertyAccessResult;

/// Array access utilities
pub struct ArrayAccess;

impl ArrayAccess {
    /// Access array index with null vs missing distinction
    #[inline]
    pub fn access_array_index(array: &JsonValue, index: i64) -> PropertyAccessResult {
        access_array_index(array, index)
    }
}

/// Array access with proper null vs missing distinction
///
/// Handles array index access while maintaining null vs missing semantics.
/// Out-of-bounds access is considered missing, not null.
#[inline]
#[allow(clippy::cast_possible_truncation)]
pub fn access_array_index(array: &JsonValue, index: i64) -> PropertyAccessResult {
    match array {
        JsonValue::Array(arr) => {
            let actual_index = if index < 0 {
                // Negative indices count from the end
                let len = arr.len() as i64;
                len + index
            } else {
                index
            };

            if actual_index >= 0 && (actual_index as usize) < arr.len() {
                let value = &arr[actual_index as usize];
                match value {
                    JsonValue::Null => PropertyAccessResult::NullValue,
                    v => PropertyAccessResult::Value(v.clone()),
                }
            } else {
                // Out of bounds is missing, not null
                PropertyAccessResult::Missing
            }
        }
        _ => PropertyAccessResult::Missing, // Non-arrays don't have indices
    }
}
