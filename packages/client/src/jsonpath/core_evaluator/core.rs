//! Core JSONPath evaluator with public API
//!
//! This module provides the main evaluator struct and public interface for JSONPath evaluation.

use serde_json::Value;

use super::timeout_protection::TimeoutProtectedEvaluator;
use crate::jsonpath::error::JsonPathError;
use crate::jsonpath::parser::{JsonPathParser, JsonSelector};

type JsonPathResult<T> = Result<T, JsonPathError>;

/// Core JSONPath evaluator that works with parsed JSON according to RFC 9535
///
/// This evaluator supports the complete JSONPath specification with optimized performance
/// and protection against pathological inputs.
pub struct CoreJsonPathEvaluator {
    /// The parsed selectors from the JSONPath expression
    pub(crate) selectors: Vec<JsonSelector>,
    /// The original expression string for debugging and error reporting
    pub(crate) expression: String,
}

impl CoreJsonPathEvaluator {
    /// Create new evaluator with JSONPath expression
    ///
    /// # Arguments
    /// * `expression` - JSONPath expression string (e.g., "$.store.book[*].author")
    ///
    /// # Returns
    /// * `JsonPathResult<Self>` - New evaluator instance or parse error
    ///
    /// # Example
    /// ```
    /// # use quyc::json_path::CoreJsonPathEvaluator;
    /// match CoreJsonPathEvaluator::new("$.store.book[*].author") {
    ///     Ok(evaluator) => {
    ///         // Use evaluator
    ///     }
    ///     Err(e) => eprintln!("Failed to create evaluator: {}", e),
    /// }
    /// ```
    pub fn new(expression: &str) -> JsonPathResult<Self> {
        // Compile the expression to get the parsed selectors
        let compiled = JsonPathParser::compile(expression)?;
        let selectors = compiled.selectors().to_vec();

        Ok(Self {
            selectors,
            expression: expression.to_string(),
        })
    }

    /// Evaluate JSONPath expression against JSON value using AST-based evaluation
    pub fn evaluate(&self, json: &Value) -> JsonPathResult<Vec<Value>> {
        // Add timeout protection for deep nesting patterns
        TimeoutProtectedEvaluator::evaluate_with_timeout(&self.expression, json)
    }

    /// Get the parsed selectors for this evaluator
    pub fn selectors(&self) -> &[JsonSelector] {
        &self.selectors
    }

    /// Get the original expression string
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Evaluate JSONPath expression against JSON value with custom timeout
    pub fn evaluate_with_custom_timeout(
        &self,
        json: &Value,
        timeout_ms: u64,
    ) -> JsonPathResult<Vec<Value>> {
        TimeoutProtectedEvaluator::evaluate_with_custom_timeout(&self.expression, json, timeout_ms)
    }

    /// Check if the expression is safe for evaluation (no deep nesting patterns)
    pub fn is_safe_expression(&self) -> bool {
        // Check for potentially problematic patterns
        let expr = &self.expression;

        // Patterns that can cause exponential complexity
        let dangerous_patterns = [
            "..*",    // Recursive descent with wildcard
            "...*",   // Multiple recursive descents
            "[*]..*", // Wildcard followed by recursive descent
        ];

        for pattern in &dangerous_patterns {
            if expr.contains(pattern) {
                return false;
            }
        }

        // Check for excessive nesting depth
        let nesting_depth = expr.matches("..").count();
        if nesting_depth > 3 {
            return false;
        }

        true
    }

    /// Get a summary of the evaluator's configuration
    pub fn summary(&self) -> String {
        format!(
            "JSONPath Evaluator: '{}' with {} selectors (safe: {})",
            self.expression,
            self.selectors.len(),
            self.is_safe_expression()
        )
    }
}

impl std::fmt::Debug for CoreJsonPathEvaluator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoreJsonPathEvaluator")
            .field("expression", &self.expression)
            .field("selector_count", &self.selectors.len())
            .field("is_safe", &self.is_safe_expression())
            .finish()
    }
}

impl std::fmt::Display for CoreJsonPathEvaluator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSONPath('{}')", self.expression)
    }
}
