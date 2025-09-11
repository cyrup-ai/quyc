//! Core `JSONPath` evaluator module
//!
//! This module provides the main `JSONPath` evaluation functionality decomposed into
//! logical submodules for maintainability and clarity.

pub mod array_operations;
pub mod array_ops;
pub mod core;
pub mod engine;
pub mod descendant_operations;
pub mod evaluator;
pub mod filter_evaluation;
pub mod filter_support;
pub mod property_operations;
pub mod recursive_descent;
pub mod selector_application;
pub mod selector_engine;
pub mod selectors;
pub mod timeout_evaluation;
pub mod timeout_protection;

// Re-export the main types and functions
pub use engine::{CoreJsonPathEvaluator, JsonPathResult};
