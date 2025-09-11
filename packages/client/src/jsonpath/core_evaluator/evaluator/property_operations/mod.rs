//! Property-based operations for `JSONPath` evaluation
//!
//! Handles property path evaluation and recursive property finding operations.

mod core;
mod extensions;
mod pattern_matching;
mod recursive;

mod utilities;

// Re-export the main struct for public API compatibility
pub use core::PropertyOperations;
