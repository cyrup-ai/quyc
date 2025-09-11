//! `JSONPath` Filter Expression Evaluation
//!
//! Handles evaluation of filter expressions including:
//! - Property access (@.property, @.nested.property)
//! - Comparisons (==, !=, <, <=, >, >=)
//! - Logical operations (&&, ||)
//! - Function calls (`length()`, `count()`, `match()`, `search()`, `value()`)

#![allow(dead_code)]

mod comparison;
mod comparisons;
mod conversions;
mod core;
mod properties;
mod property;
mod selectors;
mod utils;

// Re-export the main evaluator for backward compatibility
pub use core::FilterEvaluator;

pub use comparison::ValueComparator;
// Re-export internal modules for advanced usage
pub use property::PropertyResolver;
pub use selectors::SelectorEvaluator;
pub use utils::FilterUtils;
