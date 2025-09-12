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
                let len = i64::try_from(arr.len()).unwrap_or(i64::MAX);
                len + index
            } else {
                index
            };

            if actual_index >= 0 {
                if let Ok(index_usize) = usize::try_from(actual_index) {
                    if index_usize < arr.len() {
                        let value = &arr[index_usize];
                        match value {
                            JsonValue::Null => PropertyAccessResult::NullValue,
                            v => PropertyAccessResult::Value(v.clone()),
                        }
                    } else {
                        // Out of bounds is missing, not null
                        PropertyAccessResult::Missing
                    }
                } else {
                    // Conversion failed - index too large
                    PropertyAccessResult::Missing
                }
            } else {
                // Negative index is missing, not null
                PropertyAccessResult::Missing
            }
        }
        _ => PropertyAccessResult::Missing, // Non-arrays don't have indices
    }
}
