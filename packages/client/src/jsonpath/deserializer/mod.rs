//! High-performance streaming JSON deserializer with `JSONPath` navigation
//!
//! This module provides the core streaming deserializer that combines `JSONPath` expression
//! evaluation with incremental JSON parsing to yield individual objects from nested arrays
//! as HTTP response bytes arrive.

pub mod assembly;
pub mod byte_processor;
pub mod core;
pub mod iterator;
pub mod path_evaluator;
pub mod processor;
pub mod recursive;
pub mod selector_matcher;
pub mod streaming;

// Re-export main types for backward compatibility
pub use core::JsonPathDeserializer;

// Re-export processor types for API compatibility
pub use byte_processor::JsonProcessResult;
pub use iterator::JsonPathIterator;
pub use streaming::StreamingDeserializer;
