//! Core types and structures for `JSONPath` function evaluation
//!
//! Contains the main `FunctionEvaluator` struct and regex caching infrastructure
//! for RFC 9535 function extensions.

use crate::jsonpath::error::{JsonPathResult, invalid_expression_error};
use crate::jsonpath::parser::{FilterExpression, FilterValue};

/// Zero-allocation regex compilation cache for blazing-fast performance optimization
pub(super) struct RegexCache {
    pub(super) cache: std::sync::RwLock<std::collections::HashMap<String, regex::Regex>>,
}

impl RegexCache {
    pub(super) fn new() -> Self {
        Self {
            cache: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Get compiled regex from cache or compile and cache if not present
    pub(super) fn get_or_compile(&self, pattern: &str) -> Result<regex::Regex, regex::Error> {
        // Try read lock first for fast path
        if let Ok(cache) = self.cache.read()
            && let Some(regex) = cache.get(pattern) {
                return Ok(regex.clone());
            }

        // Compile new regex
        let regex = regex::Regex::new(pattern)?;

        // Store in cache with write lock
        if let Ok(mut cache) = self.cache.write()
            && cache.len() < 32 {
                // Limit cache size for memory efficiency
                cache.insert(pattern.to_string(), regex.clone());
            }

        Ok(regex)
    }
}

lazy_static::lazy_static! {
    pub(super) static ref REGEX_CACHE: RegexCache = RegexCache::new();
}

/// RFC 9535 Function Extensions Implementation
pub struct FunctionEvaluator;

impl FunctionEvaluator {
    /// Evaluate function calls to get their actual values (RFC 9535 Section 2.4)
    #[inline]
    pub fn evaluate_function_value(
        context: &serde_json::Value,
        name: &str,
        args: &[FilterExpression],
        expression_evaluator: &dyn Fn(
            &serde_json::Value,
            &FilterExpression,
        ) -> JsonPathResult<FilterValue>,
    ) -> JsonPathResult<FilterValue> {
        match name {
            "length" => Self::evaluate_length_function(context, args, expression_evaluator),
            "count" => Self::evaluate_count_function(context, args, expression_evaluator),
            "match" => Self::evaluate_match_function(context, args, expression_evaluator),
            "search" => Self::evaluate_search_function(context, args, expression_evaluator),
            "value" => Self::evaluate_value_function(context, args, expression_evaluator),
            _ => Err(invalid_expression_error(
                "",
                format!("unknown function: {name}"),
                None,
            )),
        }
    }

    /// Evaluate length function
    #[inline]
    pub fn evaluate_length_function(
        context: &serde_json::Value,
        args: &[FilterExpression],
        expression_evaluator: &dyn Fn(
            &serde_json::Value,
            &FilterExpression,
        ) -> JsonPathResult<FilterValue>,
    ) -> JsonPathResult<FilterValue> {
        super::function_evaluator::length::evaluate_length_function(
            context,
            args,
            expression_evaluator,
        )
    }

    /// Evaluate count function
    #[inline]
    pub fn evaluate_count_function(
        context: &serde_json::Value,
        args: &[FilterExpression],
        expression_evaluator: &dyn Fn(
            &serde_json::Value,
            &FilterExpression,
        ) -> JsonPathResult<FilterValue>,
    ) -> JsonPathResult<FilterValue> {
        super::function_evaluator::count::evaluate_count_function(
            context,
            args,
            expression_evaluator,
        )
    }

    /// Evaluate match function
    #[inline]
    pub fn evaluate_match_function(
        context: &serde_json::Value,
        args: &[FilterExpression],
        expression_evaluator: &dyn Fn(
            &serde_json::Value,
            &FilterExpression,
        ) -> JsonPathResult<FilterValue>,
    ) -> JsonPathResult<FilterValue> {
        super::function_evaluator::regex_functions::evaluate_match_function(
            context,
            args,
            expression_evaluator,
        )
    }

    /// Evaluate search function
    #[inline]
    pub fn evaluate_search_function(
        context: &serde_json::Value,
        args: &[FilterExpression],
        expression_evaluator: &dyn Fn(
            &serde_json::Value,
            &FilterExpression,
        ) -> JsonPathResult<FilterValue>,
    ) -> JsonPathResult<FilterValue> {
        super::function_evaluator::regex_functions::evaluate_search_function(
            context,
            args,
            expression_evaluator,
        )
    }

    /// Evaluate value function
    #[inline]
    pub fn evaluate_value_function(
        context: &serde_json::Value,
        args: &[FilterExpression],
        expression_evaluator: &dyn Fn(
            &serde_json::Value,
            &FilterExpression,
        ) -> JsonPathResult<FilterValue>,
    ) -> JsonPathResult<FilterValue> {
        super::function_evaluator::value::evaluate_value_function(
            context,
            args,
            expression_evaluator,
        )
    }

    /// Convert `serde_json::Value` to `FilterValue`
    #[inline]
    pub(super) fn json_value_to_filter_value(value: &serde_json::Value) -> FilterValue {
        match value {
            serde_json::Value::Null => FilterValue::Null,
            serde_json::Value::Bool(b) => FilterValue::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FilterValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    FilterValue::Number(f)
                } else {
                    FilterValue::Null
                }
            }
            serde_json::Value::String(s) => FilterValue::String(s.clone()),
            _ => FilterValue::Null, // Arrays and objects cannot be converted to FilterValue
        }
    }
}
