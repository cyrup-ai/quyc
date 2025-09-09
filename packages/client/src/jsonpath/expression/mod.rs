//! JsonPathExpression modules
//!
//! Decomposed JsonPathExpression implementation with logical separation of concerns:
//!
//! - `core`: Main JsonPathExpression struct and basic accessors
//! - `complexity`: Sophisticated complexity metrics and scoring algorithms
//! - `evaluation`: Depth-based evaluation and selector matching logic
//!
//! All modules maintain zero-allocation patterns and production-quality performance.

pub mod complexity;
pub mod core;
pub mod evaluation;

// Re-export the main type for backward compatibility
pub use core::JsonPathExpression;


