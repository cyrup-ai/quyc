//! Conversion utilities for null vs missing semantics
//!
//! Provides utilities for converting between `PropertyAccessResult` and other
//! types while maintaining or handling the null vs missing distinction.

use serde_json::Value as JsonValue;

use super::property_access::PropertyAccessResult;

/// Conversion utilities for null vs missing semantics
pub struct Conversion;

impl Conversion {
    /// Convert `PropertyAccessResult` to Option<JsonValue>
    #[inline]
    pub fn to_option(result: &PropertyAccessResult) -> Option<JsonValue> {
        to_option(result)
    }

    /// Convert `PropertyAccessResult` to `JsonValue` with missing marker
    #[inline]
    pub fn to_json_with_missing_marker(result: &PropertyAccessResult) -> JsonValue {
        to_json_with_missing_marker(result)
    }

    /// Check if a `JsonValue` is a missing marker
    #[inline]
    pub fn is_missing_marker(value: &JsonValue) -> bool {
        is_missing_marker(value)
    }
}

/// Convert `PropertyAccessResult` to Option<JsonValue> for compatibility
///
/// Used when interacting with existing code that expects Option<JsonValue>.
/// Note: This loses the null vs missing distinction.
#[inline]
pub fn to_option(result: &PropertyAccessResult) -> Option<JsonValue> {
    match result {
        PropertyAccessResult::NullValue => Some(JsonValue::Null),
        PropertyAccessResult::Value(v) => Some(v.clone()),
        PropertyAccessResult::Missing => None,
    }
}

/// Convert `PropertyAccessResult` to `JsonValue` with explicit missing representation
///
/// For cases where the distinction needs to be preserved, this converts
/// missing values to a special sentinel value that can be detected later.
#[inline]
pub fn to_json_with_missing_marker(result: &PropertyAccessResult) -> JsonValue {
    match result {
        PropertyAccessResult::NullValue => JsonValue::Null,
        PropertyAccessResult::Value(v) => v.clone(),
        PropertyAccessResult::Missing => {
            // Use a special object to represent missing values
            // This should never appear in actual JSON data
            serde_json::json!({"__jsonpath_missing__": true})
        }
    }
}

/// Check if a `JsonValue` is the missing marker
#[inline]
pub fn is_missing_marker(value: &JsonValue) -> bool {
    matches!(
        value,
        JsonValue::Object(obj) if obj.len() == 1 &&
            obj.get("__jsonpath_missing__") == Some(&JsonValue::Bool(true))
    )
}
