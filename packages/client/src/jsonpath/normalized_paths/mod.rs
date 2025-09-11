//! RFC 9535 Normalized Paths Implementation (Section 2.7)
//!
//! A Normalized Path is a `JSONPath` expression that uniquely identifies
//! a single node in a JSON value using a canonical syntax:
//! - Use bracket notation exclusively
//! - Use single quotes for member names  
//! - Use decimal integers for array indices (no leading zeros except for 0)
//! - No whitespace except where required for parsing

pub mod generator;
pub mod operations;
pub mod parser;
pub mod types;

// Re-export main types for convenience
pub use types::{NormalizedPath, NormalizedPathProcessor, PathSegment};