//! Property resolution and existence checking for filter expressions
//!
//! Contains logic for resolving property paths, checking property existence,
//! and handling missing vs null property semantics in filter contexts.

use std::collections::HashSet;

use super::property::MISSING_PROPERTY_CONTEXT;
use crate::jsonpath::error::JsonPathResult;
use crate::jsonpath::parser::FilterValue;

/// Check if property path exists and is truthy in filter context
/// This is the correct semantics for [?@.property] filters  
#[inline]
pub(super) fn property_exists_and_truthy(
    context: &serde_json::Value,
    path: &[String],
) -> JsonPathResult<bool> {
    tracing::debug!(
        target: "quyc::jsonpath::filter",
        context = %serde_json::to_string(context).unwrap_or("invalid".to_string()),
        path = ?path,
        "property_exists_and_truthy called"
    );
    let mut current = context;

    for property in path {
        tracing::trace!(
            target: "quyc::jsonpath::filter",
            property = %property,
            current = %serde_json::to_string(current).unwrap_or("invalid".to_string()),
            "Checking property in current context"
        );
        if let Some(obj) = current.as_object() {
            if let Some(value) = obj.get(property) {
                tracing::trace!(
                    target: "quyc::jsonpath::filter",
                    property = %property,
                    value = %serde_json::to_string(value).unwrap_or("invalid".to_string()),
                    "Found property"
                );
                current = value;
            } else {
                // Property doesn't exist - return false
                tracing::debug!(
                    target: "quyc::jsonpath::filter",
                    property = %property,
                    "Property does not exist, returning false"
                );
                return Ok(false);
            }
        } else {
            // Current value is not an object - can't access properties
            tracing::debug!(
                target: "quyc::jsonpath::filter",
                "Current value is not an object, returning false"
            );
            return Ok(false);
        }
    }

    // Property exists - check if it's truthy
    let _filter_value = super::conversions::json_value_to_filter_value(current);
    let result = super::core::is_truthy(current);
    tracing::debug!(
        target: "quyc::jsonpath::filter",
        result = result,
        "Property path exists, evaluated truthiness"
    );
    Ok(result)
}

/// Resolve property path with context about which properties exist
#[inline]
pub(super) fn resolve_property_path_with_context(
    context: &serde_json::Value,
    path: &[String],
    existing_properties: &HashSet<String>,
) -> JsonPathResult<FilterValue> {
    let mut current = context;

    for (i, property) in path.iter().enumerate() {
        if let Some(obj) = current.as_object() {
            if let Some(value) = obj.get(property) {
                current = value;
            } else {
                // RFC 9535: Missing properties are distinct from null values
                // For top-level properties, we need to consider context
                if i == 0 && !path.is_empty() {
                    let exists_in_context = existing_properties.contains(property);
                    tracing::debug!(
                        target: "quyc::jsonpath::filter",
                        property = %property,
                        exists_in_context = exists_in_context,
                        "Property is missing, storing context for RFC 9535 semantics"
                    );
                    // Store property name for context-aware comparison
                    MISSING_PROPERTY_CONTEXT.with(|ctx| {
                        *ctx.borrow_mut() = Some((property.clone(), exists_in_context));
                    });
                }
                return Ok(FilterValue::Missing);
            }
        } else {
            return Ok(FilterValue::Missing);
        }
    }

    Ok(super::conversions::json_value_to_filter_value(current))
}
