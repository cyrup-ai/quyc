//! RFC 9535 Section 2.4.5: count() function implementation
//!
//! Returns number of nodes in nodelist produced by argument expression

use crate::jsonpath::error::{JsonPathResult, invalid_expression_error};
use crate::jsonpath::parser::{FilterExpression, FilterValue};

/// RFC 9535 Section 2.4.5: count() function  
/// Returns number of nodes in nodelist produced by argument expression
#[inline]
pub fn evaluate_count_function(
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
            "count() function requires exactly one argument",
            None,
        ));
    }

    let count = match &args[0] {
        FilterExpression::Property { path } => {
            let mut current = context;
            for segment in path {
                match current {
                    serde_json::Value::Object(obj) => {
                        current = obj.get(segment).map_or(&serde_json::Value::Null, |v| v);
                    }
                    _ => return Ok(FilterValue::Integer(0)),
                }
            }

            match current {
                serde_json::Value::Array(arr) => arr.len() as i64,
                serde_json::Value::Object(obj) => obj.len() as i64,
                serde_json::Value::Null => 0,
                _ => 1, // Single value counts as 1
            }
        }
        _ => match expression_evaluator(context, &args[0])? {
            FilterValue::Null => 0,
            _ => 1,
        },
    };
    Ok(FilterValue::Integer(count))
}
