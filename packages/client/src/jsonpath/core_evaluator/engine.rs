//! Core `JSONPath` evaluator struct and basic functionality
//!
//! Contains the main `CoreJsonPathEvaluator` struct definition and basic construction methods.

use serde_json::Value;

use crate::jsonpath::error::JsonPathError;
use crate::jsonpath::parser::{JsonPathParser, JsonSelector};

pub type JsonPathResult<T> = Result<T, JsonPathError>;

/// Core `JSONPath` evaluator that works with parsed JSON according to RFC 9535
///
/// This evaluator supports the complete `JSONPath` specification with optimized performance
/// and protection against pathological inputs.
pub struct CoreJsonPathEvaluator {
    /// The parsed selectors from the `JSONPath` expression
    pub(crate) selectors: Vec<JsonSelector>,
    /// The original expression string for debugging and error reporting
    pub(crate) expression: String,
}

impl CoreJsonPathEvaluator {
    /// Create new evaluator with `JSONPath` expression
    ///
    /// # Errors
    /// Returns `JsonPathError` if:
    /// - The expression has invalid `JSONPath` syntax
    /// - The expression contains unsupported features
    /// - Compilation of the expression fails
    pub fn new(expression: &str) -> JsonPathResult<Self> {
        // Compile the expression to get the parsed selectors
        let compiled = JsonPathParser::compile(expression)?;
        let selectors = compiled.selectors().to_vec();

        Ok(Self {
            selectors,
            expression: expression.to_string(),
        })
    }

    /// Evaluate `JSONPath` expression against JSON value using AST-based evaluation
    ///
    /// # Errors
    /// Returns `JsonPathError` if:
    /// - Evaluation times out due to complex expression patterns
    /// - The JSON structure causes evaluation failures
    /// - Memory limits are exceeded during evaluation
    pub fn evaluate(&self, json: &Value) -> JsonPathResult<Vec<Value>> {
        // Add timeout protection for deep nesting patterns
        self.evaluate_with_timeout(json)
    }

    /// Get the original expression string
    #[must_use] 
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Get the parsed selectors
    #[must_use] 
    pub fn selectors(&self) -> &[JsonSelector] {
        &self.selectors
    }
}
