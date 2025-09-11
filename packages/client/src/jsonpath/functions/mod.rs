//! RFC 9535 `JSONPath` Function Extensions (Section 2.4)
//!
//! Implements the five built-in function extensions:
//! - `length()` (2.4.4) - Returns length of strings, arrays, or objects
//! - `count()` (2.4.5) - Returns count of nodes in a nodelist
//! - `match()` (2.4.6) - Tests if string matches regular expression
//! - `search()` (2.4.7) - Tests if string contains match for regex
//! - `value()` (2.4.8) - Converts single-node nodelist to value
//!
//! This module provides decomposed functionality with logical separation:
//! - `regex_cache`: Regex compilation cache and `ReDoS` protection
//! - `function_evaluator`: Core function implementations
//! - `jsonpath_nodelist`: `JSONPath` selector evaluation logic

#![allow(dead_code)]

pub mod function_evaluator;
pub mod jsonpath_nodelist;
pub mod regex_cache;
pub mod types;

// Re-export all public types to maintain API compatibility
pub use function_evaluator::FunctionEvaluator;
pub use jsonpath_nodelist::JsonPathNodelistEvaluator;
pub use regex_cache::{REGEX_CACHE, RegexCache, execute_regex_with_timeout};
