//! RFC 9535 JSONPath Function Extensions Implementation
//!
//! This module provides the complete implementation of all five built-in JSONPath functions
//! as specified in RFC 9535 Section 2.4, with comprehensive testing and production-quality code.

// Re-export the main evaluator struct and core functionality
pub use core::FunctionEvaluator;

// Module declarations
pub mod core;
pub mod count;
pub mod integration_tests;
pub mod length;
pub mod regex_functions;
pub mod string_counting;
pub mod value;
pub mod value_conversion;
