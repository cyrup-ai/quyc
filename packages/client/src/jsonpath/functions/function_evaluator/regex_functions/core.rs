//! Core regex function implementations
//!
//! RFC 9535 Section 2.4.6 & 2.4.7: match() and search() functions
//! with ReDoS protection and timeout handling

use super::super::super::regex_cache::{REGEX_CACHE, execute_regex_with_timeout};
use crate::jsonpath::error::{JsonPathResult, invalid_expression_error};
use crate::jsonpath::parser::{FilterExpression, FilterValue};

/// RFC 9535 Section 2.4.6: match() function
/// Tests if string matches regular expression (anchored match)
/// Includes ReDoS protection with 1-second timeout
#[inline]
pub fn evaluate_match_function(
    context: &serde_json::Value,
    args: &[FilterExpression],
    expression_evaluator: &dyn Fn(
        &serde_json::Value,
        &FilterExpression,
    ) -> JsonPathResult<FilterValue>,
) -> JsonPathResult<FilterValue> {
    if args.len() != 2 {
        return Err(invalid_expression_error(
            "",
            "match() function requires exactly two arguments",
            None,
        ));
    }

    let string_val = expression_evaluator(context, &args[0])?;
    let pattern_val = expression_evaluator(context, &args[1])?;

    if let (FilterValue::String(s), FilterValue::String(pattern)) = (string_val, pattern_val) {
        match REGEX_CACHE.get_or_compile(&pattern) {
            Ok(re) => {
                // ReDoS protection: Use timeout for regex execution
                execute_regex_with_timeout(move || re.is_match(&s))
                    .map(FilterValue::Boolean)
                    .map_err(|e| invalid_expression_error("", &e, None))
            }
            Err(_) => Err(invalid_expression_error(
                "",
                &format!("invalid regex pattern: {}", pattern),
                None,
            )),
        }
    } else {
        Ok(FilterValue::Boolean(false))
    }
}

/// RFC 9535 Section 2.4.7: search() function
/// Tests if string contains match for regular expression (unanchored search)
/// Includes ReDoS protection with 1-second timeout
#[inline]
pub fn evaluate_search_function(
    context: &serde_json::Value,
    args: &[FilterExpression],
    expression_evaluator: &dyn Fn(
        &serde_json::Value,
        &FilterExpression,
    ) -> JsonPathResult<FilterValue>,
) -> JsonPathResult<FilterValue> {
    if args.len() != 2 {
        return Err(invalid_expression_error(
            "",
            "search() function requires exactly two arguments",
            None,
        ));
    }

    let string_val = expression_evaluator(context, &args[0])?;
    let pattern_val = expression_evaluator(context, &args[1])?;

    if let (FilterValue::String(s), FilterValue::String(pattern)) = (string_val, pattern_val) {
        match REGEX_CACHE.get_or_compile(&pattern) {
            Ok(re) => {
                // ReDoS protection: Use timeout for regex execution
                execute_regex_with_timeout(move || re.find(&s).is_some())
                    .map(FilterValue::Boolean)
                    .map_err(|e| invalid_expression_error("", &e, None))
            }
            Err(_) => Err(invalid_expression_error(
                "",
                &format!("invalid regex pattern: {}", pattern),
                None,
            )),
        }
    } else {
        Ok(FilterValue::Boolean(false))
    }
}
