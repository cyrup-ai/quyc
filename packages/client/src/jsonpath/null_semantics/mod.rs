//! RFC 9535 Null vs Missing Value Semantics (Section 2.6)
//!
//! The JSON `null` value is distinct from missing values. A query may select
//! a node whose value is `null`, and a missing member is different from a
//! member with a `null` value.
//!
//! This module provides utilities for correctly handling this distinction
//! throughout `JSONPath` evaluation.

#![allow(dead_code)]

use serde_json::Value as JsonValue;

// Module declarations matching actual files
mod array_access;
mod comparison;
mod conversion;
mod core;
mod property_access;

// Re-export main types and utilities
pub use property_access::PropertyAccessResult;

/// Utilities for handling RFC 9535 null vs missing value semantics
pub struct NullSemantics;

impl NullSemantics {
    /// Access a property with proper null vs missing distinction
    #[inline]
    #[must_use] 
    pub fn access_property(object: &JsonValue, property_name: &str) -> PropertyAccessResult {
        property_access::access_property(object, property_name)
    }

    /// Access a nested property path with null vs missing distinction
    #[inline]
    #[must_use] 
    pub fn access_property_path(root: &JsonValue, path: &[String]) -> PropertyAccessResult {
        property_access::access_property_path(root, path)
    }

    /// Array access with proper null vs missing distinction
    #[inline]
    #[must_use] 
    pub fn access_array_index(array: &JsonValue, index: i64) -> PropertyAccessResult {
        array_access::access_array_index(array, index)
    }

    /// Check if a value should be considered "present" for filter evaluation
    #[inline]
    #[must_use] 
    pub fn is_present(result: &PropertyAccessResult) -> bool {
        comparison::is_present(result)
    }

    /// Filter evaluation with proper null vs missing handling
    #[inline]
    #[must_use] 
    pub fn evaluate_existence_filter(context: &JsonValue, property_path: &[String]) -> bool {
        comparison::evaluate_existence_filter(context, property_path)
    }

    /// Comparison with null vs missing distinction
    #[inline]
    pub fn compare_with_null_semantics(
        left: &PropertyAccessResult,
        right: &PropertyAccessResult,
    ) -> crate::jsonpath::error::JsonPathResult<bool> {
        comparison::compare_with_null_semantics(left, right)
    }

    /// Convert `PropertyAccessResult` to Option<JsonValue> for compatibility
    #[inline]
    #[must_use] 
    pub fn to_option(result: &PropertyAccessResult) -> Option<JsonValue> {
        conversion::to_option(result)
    }

    /// Convert `PropertyAccessResult` to `JsonValue` with explicit missing representation
    #[inline]
    #[must_use] 
    pub fn to_json_with_missing_marker(result: &PropertyAccessResult) -> JsonValue {
        conversion::to_json_with_missing_marker(result)
    }

    /// Check if a `JsonValue` is the missing marker
    #[inline]
    #[must_use] 
    pub fn is_missing_marker(value: &JsonValue) -> bool {
        conversion::is_missing_marker(value)
    }

    /// Generate test results for different null vs missing scenarios
    #[inline]
    #[must_use] 
    pub fn generate_test_scenarios() -> Vec<(JsonValue, &'static str, PropertyAccessResult)> {
        comparison::generate_test_scenarios()
    }

    /// Validate that null vs missing semantics are correctly implemented
    #[inline]
    pub fn validate_implementation() -> crate::jsonpath::error::JsonPathResult<()> {
        comparison::validate_implementation()
    }
}
