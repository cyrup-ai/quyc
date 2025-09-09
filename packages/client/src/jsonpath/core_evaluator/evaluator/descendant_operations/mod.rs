//! Descendant operations for JSONPath recursive descent processing
//!
//! Handles recursive descent (..) operations and descendant collection
//! with RFC 9535 compliance.

mod advanced;
mod analysis;
mod collection;
mod core;
mod filtering;

mod utilities;

// Re-export the main struct for public API compatibility
pub use core::DescendantOperations;
