//! `JSONPath` selector application with zero-allocation patterns
//!
//! Decomposed selector handling for individual selector types:
//! child, index, wildcard, filter, slice, union with both owned and reference-based operations.

pub mod arrays;
pub mod core;
pub mod filters;
pub mod wildcards;

// Re-export core functionality
