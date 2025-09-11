//! Filter expression parsing for `JSONPath` predicates
//!
//! Handles parsing of complex filter expressions including comparisons,
//! logical operations, function calls, and property access patterns.

pub mod core;
pub mod expressions;
pub mod functions;
pub mod properties;

// Re-export the main FilterParser struct
pub use core::FilterParser;
