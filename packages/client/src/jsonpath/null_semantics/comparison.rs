//! Comparison and evaluation operations for null vs missing semantics
//!
//! Implements filter evaluation and comparison logic that correctly handles
//! the distinction between null values and missing properties.

use serde_json::Value as JsonValue;

use super::property_access::{PropertyAccessResult, access_property_path};
use crate::jsonpath::error::{JsonPathResult, invalid_expression_error};

/// Comparison utilities for null vs missing semantics
pub struct Comparison;

impl Comparison {
    /// Evaluate existence filter with null vs missing distinction
    #[inline]
    pub fn evaluate_existence_filter(
        context: &JsonValue,
        property_path: &[String],
    ) -> JsonPathResult<bool> {
        Ok(evaluate_existence_filter(context, property_path))
    }

    /// Compare values with null semantics
    #[inline]
    pub fn compare_with_null_semantics(
        left: &PropertyAccessResult,
        right: &PropertyAccessResult,
    ) -> JsonPathResult<bool> {
        compare_with_null_semantics(left, right)
    }
}

/// Check if a value should be considered "present" for filter evaluation
///
/// According to RFC 9535, null values are present but missing values are not.
/// This affects filter expression evaluation.
#[inline]
pub fn is_present(result: &PropertyAccessResult) -> bool {
    match result {
        PropertyAccessResult::NullValue | PropertyAccessResult::Value(_) => true, // null and non-null values are present
        PropertyAccessResult::Missing => false,  // missing is not present
    }
}

/// Filter evaluation with proper null vs missing handling
///
/// Evaluates filter expressions while correctly handling the distinction
/// between null values and missing properties.
#[inline]
pub fn evaluate_existence_filter(context: &JsonValue, property_path: &[String]) -> bool {
    let result = access_property_path(context, property_path);
    is_present(&result)
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
    match (left, right) {
        // Both null values
        (PropertyAccessResult::NullValue, PropertyAccessResult::NullValue) => Ok(true),

        // Both missing
        (PropertyAccessResult::Missing, PropertyAccessResult::Missing) => Ok(true),

        // Null vs missing (different)
        (PropertyAccessResult::NullValue, PropertyAccessResult::Missing) | 
        (PropertyAccessResult::Missing, PropertyAccessResult::NullValue) => Ok(false),

        // Value comparisons
        (PropertyAccessResult::Value(a), PropertyAccessResult::Value(b)) => Ok(a == b),

        // Value vs null (different unless value is explicitly null)
        (PropertyAccessResult::Value(JsonValue::Null), PropertyAccessResult::NullValue) | 
        (PropertyAccessResult::NullValue, PropertyAccessResult::Value(JsonValue::Null)) => Ok(true),
        (PropertyAccessResult::Value(_), PropertyAccessResult::NullValue) | 
        (PropertyAccessResult::NullValue, PropertyAccessResult::Value(_)) => Ok(false),

        // Value vs missing (different)
        (PropertyAccessResult::Value(_), PropertyAccessResult::Missing) | 
        (PropertyAccessResult::Missing, PropertyAccessResult::Value(_)) => Ok(false),
    }
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
    let test_cases = generate_test_scenarios();

    for (json, property, expected) in test_cases {
        let result = super::property_access::access_property(&json, property);
        if result != expected {
            return Err(invalid_expression_error(
                "",
                format!(
                    "null semantics validation failed: expected {expected:?}, got {result:?}"
                ),
                None,
            ));
        }
    }

    Ok(())
}
