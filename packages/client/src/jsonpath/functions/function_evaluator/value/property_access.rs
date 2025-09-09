//! Property access evaluation for value() function

use super::super::core::FunctionEvaluator;
use crate::jsonpath::parser::FilterValue;

/// Property access evaluator for value() function
pub struct PropertyAccessEvaluator;

impl PropertyAccessEvaluator {
    /// Evaluate property path access
    pub fn evaluate_property_path(
        context: &serde_json::Value,
        path: &[String],
    ) -> crate::jsonpath::error::JsonPathResult<FilterValue> {
        let mut current = context;
        for segment in path {
            match current {
                serde_json::Value::Object(obj) => {
                    current = obj.get(segment).map_or(&serde_json::Value::Null, |v| v);
                }
                _ => return Ok(FilterValue::Null),
            }
        }
        Ok(FunctionEvaluator::json_value_to_filter_value(current))
    }
}
