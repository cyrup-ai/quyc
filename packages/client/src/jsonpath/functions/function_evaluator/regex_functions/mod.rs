//! Regex functions module
//!
//! RFC 9535 Section 2.4.6 & 2.4.7: match() and search() function implementations
//! with ReDoS protection and comprehensive testing

mod core;

// Re-export the main functions for public API compatibility
pub use core::{evaluate_match_function, evaluate_search_function};
