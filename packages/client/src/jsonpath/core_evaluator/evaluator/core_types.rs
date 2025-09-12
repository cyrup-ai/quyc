//! Core types and structures for `JSONPath` evaluation
//!
//! Defines the main `CoreJsonPathEvaluator` struct and associated types.

// Removed unused import

use crate::jsonpath::error::JsonPathError;
use crate::jsonpath::parser::{JsonPathParser, JsonSelector};

/// Result type for `JSONPath` operations
pub type JsonPathResult<T> = Result<T, JsonPathError>;

/// Core `JSONPath` evaluator that works with parsed JSON according to RFC 9535
///
/// This evaluator supports the complete `JSONPath` specification with optimized performance
/// and protection against pathological inputs.
#[derive(Debug, Clone)]
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

    /// Create a temporary evaluator instance for internal use
    pub(crate) fn create_temp_evaluator(expression: &str) -> JsonPathResult<Self> {
        let compiled = JsonPathParser::compile(expression)?;
        Ok(Self {
            selectors: compiled.selectors().to_vec(),
            expression: expression.to_string(),
        })
    }
}


