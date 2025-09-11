//! Utility methods and helper functions for the type system
//!
//! Contains conversion helpers, value creation utilities, and other
//! support functions for the `JSONPath` function type system.

use super::core::{TypeSystem, TypedValue};
use crate::jsonpath::{
    ast::FilterValue,
    error::{JsonPathResult, invalid_expression_error},
};

impl TypeSystem {
    /// Convert `FilterValue` to `TypedValue`
    ///
    /// Bridges the gap between `FilterValue` (used in existing code)
    /// and `TypedValue` (used in the new type system).
    #[inline]
    #[must_use] 
    pub fn filter_value_to_typed_value(value: &FilterValue) -> TypedValue {
        match value {
            FilterValue::String(s) => TypedValue::Value(serde_json::Value::String(s.clone())),
            FilterValue::Number(n) => TypedValue::Value(serde_json::json!(*n)),
            FilterValue::Integer(i) => TypedValue::Value(serde_json::json!(*i)),
            FilterValue::Boolean(b) => TypedValue::Logical(*b),
            FilterValue::Null => TypedValue::Value(serde_json::Value::Null),
            FilterValue::Missing => TypedValue::Value(serde_json::Value::Null), /* Missing converts to null */
        }
    }

    /// Convert `TypedValue` to `FilterValue`
    ///
    /// Converts from the new type system back to the existing `FilterValue`
    /// for compatibility with existing code.
    #[inline]
    pub fn typed_value_to_filter_value(value: &TypedValue) -> JsonPathResult<FilterValue> {
        match value {
            TypedValue::Value(json_val) => match json_val {
                serde_json::Value::Null => Ok(FilterValue::Null),
                serde_json::Value::Bool(b) => Ok(FilterValue::Boolean(*b)),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Ok(FilterValue::Integer(i))
                    } else if let Some(f) = n.as_f64() {
                        Ok(FilterValue::Number(f))
                    } else {
                        Ok(FilterValue::Null)
                    }
                }
                serde_json::Value::String(s) => Ok(FilterValue::String(s.clone())),
                _ => Err(invalid_expression_error(
                    "",
                    "arrays and objects cannot be converted to FilterValue",
                    None,
                )),
            },
            TypedValue::Logical(b) => Ok(FilterValue::Boolean(*b)),
            TypedValue::Nodes(_) => Err(invalid_expression_error(
                "",
                "NodesType cannot be converted to FilterValue",
                None,
            )),
        }
    }

    /// Create a nodelist `TypedValue` from a vector of JSON values
    ///
    /// Helper function for creating `NodesType` values from `JSONPath` evaluation results.
    #[inline]
    #[must_use] 
    pub fn create_nodes_value(nodes: Vec<serde_json::Value>) -> TypedValue {
        TypedValue::Nodes(nodes)
    }

    /// Extract nodes from a `TypedValue`
    ///
    /// Returns the underlying node vector if the value is `NodesType`,
    /// otherwise returns an error.
    #[inline]
    pub fn extract_nodes(value: &TypedValue) -> JsonPathResult<&[serde_json::Value]> {
        match value {
            TypedValue::Nodes(nodes) => Ok(nodes),
            _ => Err(invalid_expression_error(
                "",
                "expected NodesType value",
                None,
            )),
        }
    }

    /// Check if a `TypedValue` is empty (for `NodesType`) or falsy (for other types)
    ///
    /// Used for optimizing filter expressions and short-circuit evaluation.
    #[inline]
    #[must_use] 
    pub fn is_empty_or_falsy(value: &TypedValue) -> bool {
        match value {
            TypedValue::Value(json_val) => !Self::value_to_logical(json_val),
            TypedValue::Logical(b) => !*b,
            TypedValue::Nodes(nodes) => nodes.is_empty(),
        }
    }

    /// Convert JSON value to logical type using test expression conversion
    ///
    /// RFC 9535: `ValueType` to `LogicalType` conversion uses the "truthiness" rules:
    /// - false and null are false
    /// - Numbers: zero is false, all others are true  
    /// - Strings: empty string is false, all others are true
    /// - Arrays and objects: always true (even if empty)
    #[inline]
    #[must_use] 
    pub fn value_to_logical(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Null => false,
            serde_json::Value::Bool(b) => *b,
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    i != 0
                } else if let Some(f) = n.as_f64() {
                    f != 0.0 && !f.is_nan()
                } else {
                    false
                }
            }
            serde_json::Value::String(s) => !s.is_empty(),
            serde_json::Value::Array(_) => true, // Always true, even if empty
            serde_json::Value::Object(_) => true, // Always true, even if empty
        }
    }
}
