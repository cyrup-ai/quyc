//! Core NullSemantics struct and main operations

use serde_json::Value as JsonValue;

use super::array_access::ArrayAccess;
use super::comparison::Comparison;
use super::conversion::Conversion;
use super::property_access::{PropertyAccess, PropertyAccessResult};
use crate::jsonpath::error::{JsonPathResult, invalid_expression_error};

/// Utilities for handling RFC 9535 null vs missing value semantics
pub struct NullSemantics;

impl NullSemantics {
    /// Access a property with proper null vs missing distinction
    ///
    /// Returns PropertyAccessResult to distinguish between:
    /// - A property that exists with null value
    /// - A property that does not exist (missing)
    /// - A property that exists with a non-null value
    #[inline]
    pub fn access_property(object: &JsonValue, property_name: &str) -> PropertyAccessResult {
        PropertyAccess::access_property(object, property_name)
    }

    /// Access a nested property path with null vs missing distinction
    ///
    /// Follows a property path through nested objects, maintaining proper
    /// distinction between null values and missing properties at each level.
    #[inline]
    pub fn access_property_path(root: &JsonValue, path: &[String]) -> PropertyAccessResult {
        PropertyAccess::access_property_path(root, path)
    }

    /// Check if a value should be considered "present" for filter evaluation
    ///
    /// According to RFC 9535, null values are present but missing values are not.
    /// This affects filter expression evaluation.
    #[inline]
    pub fn is_present(result: &PropertyAccessResult) -> bool {
        result.is_present()
    }

    /// Convert PropertyAccessResult to Option<JsonValue> for compatibility
    ///
    /// Used when interacting with existing code that expects Option<JsonValue>.
    /// Note: This loses the null vs missing distinction.
    #[inline]
    pub fn to_option(result: &PropertyAccessResult) -> Option<JsonValue> {
        Conversion::to_option(result)
    }

    /// Convert PropertyAccessResult to JsonValue with explicit missing representation
    ///
    /// For cases where the distinction needs to be preserved, this converts
    /// missing values to a special sentinel value that can be detected later.
    #[inline]
    pub fn to_json_with_missing_marker(result: &PropertyAccessResult) -> JsonValue {
        Conversion::to_json_with_missing_marker(result)
    }

    /// Check if a JsonValue is the missing marker
    #[inline]
    pub fn is_missing_marker(value: &JsonValue) -> bool {
        Conversion::is_missing_marker(value)
    }

    /// Array access with proper null vs missing distinction
    ///
    /// Handles array index access while maintaining null vs missing semantics.
    /// Out-of-bounds access is considered missing, not null.
    #[inline]
    pub fn access_array_index(array: &JsonValue, index: i64) -> PropertyAccessResult {
        ArrayAccess::access_array_index(array, index)
    }

    /// Filter evaluation with proper null vs missing handling
    ///
    /// Evaluates filter expressions while correctly handling the distinction
    /// between null values and missing properties.
    #[inline]
    pub fn evaluate_existence_filter(context: &JsonValue, property_path: &[String]) -> bool {
        Comparison::evaluate_existence_filter(context, property_path).unwrap_or(false)
    }

    /// Comparison with null vs missing distinction
    ///
    /// Handles comparisons involving null values and missing properties
    /// according to RFC 9535 semantics.
    #[inline]
    pub fn compare_with_null_semantics(
        left: &PropertyAccessResult,
        right: &PropertyAccessResult,
    ) -> JsonPathResult<bool> {
        Comparison::compare_with_null_semantics(left, right)
    }

    /// Generate test results for different null vs missing scenarios
    ///
    /// Used for testing and validation to ensure proper handling of edge cases.
    #[inline]
    pub fn generate_test_scenarios() -> Vec<(JsonValue, &'static str, PropertyAccessResult)> {
        vec![
            // Null value present
            (
                serde_json::json!({"a": null}),
                "a",
                PropertyAccessResult::NullValue,
            ),
            // Property missing
            (serde_json::json!({}), "a", PropertyAccessResult::Missing),
            // Non-null value present
            (
                serde_json::json!({"a": "value"}),
                "a",
                PropertyAccessResult::Value(JsonValue::String("value".to_string())),
            ),
            // Root level property with null in nested structure
            (
                serde_json::json!({"nested": null, "obj": {"other": "value"}}),
                "nested", // Access root.nested
                PropertyAccessResult::NullValue,
            ),
            // Root level missing property
            (
                serde_json::json!({"obj": {"nested": null}}),
                "missing", // Access root.missing
                PropertyAccessResult::Missing,
            ),
        ]
    }

    /// Validate that null vs missing semantics are correctly implemented
    ///
    /// Runs validation tests to ensure the implementation correctly distinguishes
    /// between null values and missing properties in various scenarios.
    #[inline]
    pub fn validate_implementation() -> JsonPathResult<()> {
        let test_cases = Self::generate_test_scenarios();

        for (json, property, expected) in test_cases {
            let result = Self::access_property(&json, property);
            if result != expected {
                return Err(invalid_expression_error(
                    "",
                    &format!(
                        "null semantics validation failed: expected {:?}, got {:?}",
                        expected, result
                    ),
                    None,
                ));
            }
        }

        Ok(())
    }
}
