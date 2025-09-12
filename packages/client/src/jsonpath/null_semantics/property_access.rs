//! Property access operations with null vs missing distinction
//!
//! Implements property access that maintains proper distinction between
//! null values and missing properties according to RFC 9535.

use serde_json::Value as JsonValue;

/// Property access utilities for null vs missing semantics
pub struct PropertyAccess;

impl PropertyAccess {
    /// Access a single property with null vs missing distinction
    #[inline]
    pub fn access_property(object: &JsonValue, property_name: &str) -> PropertyAccessResult {
        match object {
            JsonValue::Object(map) => match map.get(property_name) {
                Some(JsonValue::Null) => PropertyAccessResult::NullValue,
                Some(value) => PropertyAccessResult::Value(value.clone()),
                None => PropertyAccessResult::Missing,
            },
            _ => PropertyAccessResult::Missing,
        }
    }

    /// Access a property path with null vs missing distinction
    #[inline]
    pub fn access_property_path(object: &JsonValue, path: &[String]) -> PropertyAccessResult {
        // Inline implementation of property path access
        let mut current = object;

        for (index, property) in path.iter().enumerate() {
            match current {
                JsonValue::Object(obj) => {
                    match obj.get(property) {
                        Some(JsonValue::Null) => {
                            // If this is the final property in the path, return null
                            if index == path.len() - 1 {
                                return PropertyAccessResult::NullValue;
                            } 
                            // Cannot traverse through null to access deeper properties
                            return PropertyAccessResult::Missing;
                        }
                        Some(value) => {
                            current = value;
                        }
                        None => {
                            // Property missing at this level
                            return PropertyAccessResult::Missing;
                        }
                    }
                }
                _ => {
                    // Current value is not an object, cannot access properties
                    return PropertyAccessResult::Missing;
                }
            }
        }

        // If we've traversed the entire path, return the final value
        match current {
            JsonValue::Null => PropertyAccessResult::NullValue,
            value => PropertyAccessResult::Value(value.clone()),
        }
    }
}

/// Represents the result of a property access that distinguishes between
/// null values and missing properties according to RFC 9535 Section 2.6
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyAccessResult {
    /// Property exists and has a null value
    NullValue,
    /// Property exists and has a non-null value
    Value(JsonValue),
    /// Property does not exist (missing)
    Missing,
}

impl PropertyAccessResult {
    /// Check if this result represents a present value (null or non-null)
    #[inline]
    #[must_use] 
    pub fn is_present(&self) -> bool {
        match self {
            PropertyAccessResult::NullValue => true, // null is present
            PropertyAccessResult::Value(_) => true,  // non-null values are present
            PropertyAccessResult::Missing => false,  // missing is not present
        }
    }

    /// Check if this result represents a missing property
    #[inline]
    #[must_use] 
    pub fn is_missing(&self) -> bool {
        matches!(self, PropertyAccessResult::Missing)
    }

    /// Check if this result represents a null value
    #[inline]
    #[must_use] 
    pub fn is_null(&self) -> bool {
        matches!(self, PropertyAccessResult::NullValue)
    }

    /// Get the `JsonValue` if present, otherwise return None
    #[inline]
    #[must_use] 
    pub fn value(&self) -> Option<&JsonValue> {
        match self {
            PropertyAccessResult::Value(v) => Some(v),
            PropertyAccessResult::NullValue | PropertyAccessResult::Missing => None, // Explicitly null or missing
        }
    }

    /// Get the `JsonValue` with null preserved
    #[inline]
    #[must_use] 
    pub fn value_with_null(&self) -> Option<JsonValue> {
        match self {
            PropertyAccessResult::Value(v) => Some(v.clone()),
            PropertyAccessResult::NullValue => Some(JsonValue::Null),
            PropertyAccessResult::Missing => None,
        }
    }
}

/// Access a property with proper null vs missing distinction
///
/// Returns `PropertyAccessResult` to distinguish between:
/// - A property that exists with null value
/// - A property that does not exist (missing)
/// - A property that exists with a non-null value
#[inline]
pub fn access_property(object: &JsonValue, property_name: &str) -> PropertyAccessResult {
    match object {
        JsonValue::Object(obj) => match obj.get(property_name) {
            Some(JsonValue::Null) => PropertyAccessResult::NullValue,
            Some(value) => PropertyAccessResult::Value(value.clone()),
            None => PropertyAccessResult::Missing,
        },
        _ => PropertyAccessResult::Missing, // Non-objects don't have properties
    }
}

/// Access a nested property path with null vs missing distinction
///
/// Follows a property path through nested objects, maintaining proper
/// distinction between null values and missing properties at each level.
#[inline]
pub fn access_property_path(root: &JsonValue, path: &[String]) -> PropertyAccessResult {
    let mut current = root;

    for (index, property) in path.iter().enumerate() {
        match current {
            JsonValue::Object(obj) => {
                match obj.get(property) {
                    Some(JsonValue::Null) => {
                        // If this is the final property in the path, return null
                        if index == path.len() - 1 {
                            return PropertyAccessResult::NullValue;
                        } 
                        // Cannot traverse through null to access deeper properties
                        return PropertyAccessResult::Missing;
                    }
                    Some(value) => {
                        current = value;
                    }
                    None => {
                        // Property missing at this level
                        return PropertyAccessResult::Missing;
                    }
                }
            }
            _ => {
                // Current value is not an object, cannot access properties
                return PropertyAccessResult::Missing;
            }
        }
    }

    // If we've traversed the entire path, return the final value
    match current {
        JsonValue::Null => PropertyAccessResult::NullValue,
        value => PropertyAccessResult::Value(value.clone()),
    }
}
