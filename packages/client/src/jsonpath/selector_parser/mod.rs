//! JSONPath selector parsing implementation
//!
//! Decomposed selector parser for individual JSONPath selectors including
//! array indices, slices, filters, property access, and union patterns.

pub mod bracket;
pub mod core;
pub mod dot;
pub mod slice;

// Re-export main parser
pub use core::SelectorParser;
