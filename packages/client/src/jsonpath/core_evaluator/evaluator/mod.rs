//! JSONPath core evaluator module
//!
//! This module provides the main JSONPath evaluation functionality decomposed into
//! logical components for better maintainability and organization.

pub mod core_types;
pub mod descendant_operations;
pub mod evaluation_engine;
pub mod property_operations;
pub mod timeout_handler;

// Re-export main types for convenience
pub use core_types::{CoreJsonPathEvaluator, JsonPathResult};
pub use descendant_operations::DescendantOperations;
pub use evaluation_engine::EvaluationEngine;
pub use property_operations::PropertyOperations;
pub use timeout_handler::{TimeoutConfig, TimeoutHandler};

// Main evaluator implementation combining all components
impl CoreJsonPathEvaluator {
    /// Evaluate JSONPath expression against JSON value using AST-based evaluation
    pub fn evaluate(&self, json: &serde_json::Value) -> JsonPathResult<Vec<serde_json::Value>> {
        // Add timeout protection for deep nesting patterns
        TimeoutHandler::evaluate_with_timeout(self, json, None)
    }

    /// Evaluate with custom timeout configuration
    pub fn evaluate_with_config(
        &self,
        json: &serde_json::Value,
        config: TimeoutConfig,
    ) -> JsonPathResult<Vec<serde_json::Value>> {
        TimeoutHandler::evaluate_with_timeout(self, json, Some(config))
    }
}