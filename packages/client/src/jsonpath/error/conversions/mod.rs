//! Type conversions and compatibility implementations for JSONPath errors
//!
//! Provides From trait implementations for converting external error types
//! into JsonPathError variants, along with helper traits and utilities.

#![allow(dead_code)]

mod helper_traits;
mod helpers;
mod std_conversions;

// Re-export all conversion functionality for public API compatibility
// Only use helpers to avoid duplicate IntoJsonPathError trait

