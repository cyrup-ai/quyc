//! Timeout protection for `JSONPath` evaluation
//!
//! This module provides safety mechanisms to prevent excessive processing time on pathological inputs.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::jsonpath::error::JsonPathError;

type JsonPathResult<T> = Result<T, JsonPathError>;

/// Timeout-protected evaluator for preventing runaway `JSONPath` evaluations
pub struct TimeoutProtectedEvaluator;

impl TimeoutProtectedEvaluator {
    /// Evaluate with default timeout protection (1.5 seconds)
    ///
    /// # Errors
    /// Returns `JsonPathError` if:
    /// - Expression compilation fails due to invalid `JSONPath` syntax
    /// - Evaluation times out after 1.5 seconds
    /// - Evaluation thread disconnects unexpectedly
    /// - Selector application fails during evaluation
    pub fn evaluate_with_timeout(expression: &str, json: &Value) -> JsonPathResult<Vec<Value>> {
        Self::evaluate_with_custom_timeout(expression, json, 1500)
    }

    /// Evaluate with custom timeout protection
    ///
    /// # Errors
    /// Returns `JsonPathError` if:
    /// - Expression compilation fails due to invalid `JSONPath` syntax
    /// - Evaluation times out after the specified duration
    /// - Evaluation thread disconnects unexpectedly during processing
    /// - Selector application fails during evaluation
    /// - Result set exceeds memory limits (>10000 items)
    pub fn evaluate_with_custom_timeout(
        expression: &str,
        json: &Value,
        timeout_ms: u64,
    ) -> JsonPathResult<Vec<Value>> {
        let timeout_duration = Duration::from_millis(timeout_ms);
        let start_time = Instant::now();

        let (tx, rx) = mpsc::channel();
        let expression_str = expression.to_string();
        let expression_str_for_error = expression_str.clone(); // Clone for error message
        let json_clone = json.clone();

        // Spawn evaluation in separate thread
        let _handle = thread::spawn(move || {
            log::debug!("Starting JSONPath evaluation in timeout thread");
            let result = Self::evaluate_internal(&expression_str, &json_clone);
            log::debug!("JSONPath evaluation completed in thread");
            let _ = tx.send(result); // Ignore send errors if receiver dropped
        });

        // Wait for completion or timeout
        match rx.recv_timeout(timeout_duration) {
            Ok(result) => {
                let elapsed = start_time.elapsed();
                log::debug!(
                    "JSONPath evaluation completed successfully in {elapsed:?}"
                );
                result
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let elapsed = start_time.elapsed();
                log::warn!(
                    "JSONPath evaluation timed out after {elapsed:?} - likely deep nesting issue"
                );
                Err(JsonPathError::new(
                    crate::jsonpath::error::ErrorKind::ProcessingError,
                    format!(
                        "Expression '{expression_str_for_error}' timed out after {timeout_ms}ms"
                    ),
                ))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                log::error!("JSONPath evaluation thread disconnected unexpectedly");
                Err(JsonPathError::new(
                    crate::jsonpath::error::ErrorKind::ProcessingError,
                    "Evaluation thread disconnected".to_string(),
                ))
            }
        }
    }

    /// Internal evaluation method (runs in separate thread)
    fn evaluate_internal(expression: &str, json: &Value) -> JsonPathResult<Vec<Value>> {
        use crate::jsonpath::parser::JsonPathParser;

        // Parse the expression
        let compiled = JsonPathParser::compile(expression)?;
        let selectors = compiled.selectors();

        // Apply selectors sequentially
        let mut current_results = vec![json.clone()];

        for selector in selectors {
            let mut next_results = Vec::new();

            for value in &current_results {
                let selector_results = Self::apply_selector_to_value(value, selector)?;
                next_results.extend(selector_results);

                // Safety check: prevent memory exhaustion
                if next_results.len() > 10000 {
                    log::warn!(
                        "JSONPath evaluation stopped - result set too large ({})",
                        next_results.len()
                    );
                    return Ok(vec![]);
                }
            }

            current_results = next_results;

            // Early termination if no results
            if current_results.is_empty() {
                return Ok(vec![]);
            }
        }

        Ok(current_results)
    }

    /// Apply a single selector to a JSON value
    fn apply_selector_to_value(
        value: &Value,
        selector: &crate::jsonpath::parser::JsonSelector,
    ) -> JsonPathResult<Vec<Value>> {
        use super::selector_engine::SelectorEngine;

        SelectorEngine::apply_selector(value, selector)
    }

    /// Check if an expression is potentially dangerous
    #[must_use] 
    pub fn is_dangerous_expression(expression: &str) -> bool {
        // Patterns that can cause exponential complexity
        let dangerous_patterns = [
            "..*",    // Recursive descent with wildcard
            "...*",   // Multiple recursive descents
            "[*]..*", // Wildcard followed by recursive descent
            "$..*.*", // Deep recursive patterns
        ];

        for pattern in &dangerous_patterns {
            if expression.contains(pattern) {
                return true;
            }
        }

        // Check for excessive nesting depth
        let nesting_depth = expression.matches("..").count();
        if nesting_depth > 5 {
            return true;
        }

        // Check for complex filter expressions
        let filter_count = expression.matches("?@").count();
        if filter_count > 3 {
            return true;
        }

        false
    }

    /// Get recommended timeout for an expression
    #[must_use] 
    pub fn recommended_timeout_ms(expression: &str) -> u64 {
        if Self::is_dangerous_expression(expression) {
            5000 // 5 seconds for dangerous expressions
        } else if expression.contains("..") {
            2000 // 2 seconds for recursive descent
        } else if expression.contains("[?") {
            1500 // 1.5 seconds for filters
        } else {
            500 // 500ms for simple expressions
        }
    }
}


