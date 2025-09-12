//! RFC 9535 Section 2.4.4: `length()` function implementation
//!
//! Returns number of characters in string, elements in array, or members in object

use crate::jsonpath::error::{JsonPathResult, invalid_expression_error};
use crate::jsonpath::parser::{FilterExpression, FilterValue};

/// RFC 9535 Section 2.4.4: `length()` function
/// Returns number of characters in string, elements in array, or members in object
///
/// # Errors
/// Returns `JsonPathError` if:
/// - Function arguments are invalid, missing, or of wrong type
/// - Expression evaluation fails on the provided context
/// - Length calculation encounters overflow or invalid data structures
#[inline]
pub fn evaluate_length_function(
    context: &serde_json::Value,
    args: &[FilterExpression],
    expression_evaluator: &dyn Fn(
        &serde_json::Value,
        &FilterExpression,
    ) -> JsonPathResult<FilterValue>,
) -> JsonPathResult<FilterValue> {
    if args.len() != 1 {
        return Err(invalid_expression_error(
            "",
            "length() function requires exactly one argument",
            None,
        ));
    }

    if let FilterExpression::Property { path } = &args[0] {
        let mut current = context;
        for segment in path {
            match current {
                serde_json::Value::Object(obj) => {
                    current = obj.get(segment).map_or(&serde_json::Value::Null, |v| v);
                }
                _ => return Ok(FilterValue::Null),
            }
        }

        let len = match current {
            serde_json::Value::Array(arr) => i64::try_from(arr.len()).unwrap_or(i64::MAX),
            serde_json::Value::Object(obj) => i64::try_from(obj.len()).unwrap_or(i64::MAX),
            serde_json::Value::String(s) => i64::try_from(s.chars().count()).unwrap_or(i64::MAX), // Unicode-aware
            _ => return Ok(FilterValue::Null), // Null and primitives return null per RFC
        };
        Ok(FilterValue::Integer(len))
    } else {
        let value = expression_evaluator(context, &args[0])?;
        match value {
            FilterValue::String(s) => Ok(FilterValue::Integer(i64::try_from(s.chars().count()).unwrap_or(i64::MAX))),
            FilterValue::Integer(_) | FilterValue::Number(_) | FilterValue::Boolean(_) => {
                Ok(FilterValue::Null) // Primitives return null per RFC
            }
            FilterValue::Null | FilterValue::Missing => Ok(FilterValue::Null), /* Null and missing properties have no length */
        }
    }
}


