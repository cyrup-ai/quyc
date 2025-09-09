//! JSONPath expression tokenizer implementation
//!
//! Decomposed tokenizer for JSONPath expressions with RFC 9535 compliance.
//! Handles lexical analysis converting raw strings into structured token sequences.

mod characters;
mod core;
mod numbers;
mod operators;
mod strings;

// Re-export main parser
pub use core::ExpressionParser;
