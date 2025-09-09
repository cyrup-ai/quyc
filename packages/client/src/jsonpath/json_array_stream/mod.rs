//! High-performance JSONPath streaming deserializer for Http3
//!
//! This module provides blazing-fast, zero-allocation JSONPath expression evaluation
//! over streaming HTTP responses. It enables streaming individual array elements from
//! nested JSON structures like OpenAI's `{"data": [...]}` format.
//!
//! # Features
//!
//! - Full JSONPath specification support
//! - Zero-allocation streaming deserialization
//! - Lock-free concurrent processing
//! - Comprehensive error handling and recovery
//! - Integration with Http3 builder pattern
//!
//! # Examples
//!
//! ```rust
//! use quyc::Http3;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Model {
//!     id: String,
//!     object: String,
//! }
//!
//! // Stream individual models from OpenAI's {"data": [...]} response
//! Http3::json()
//!     .array_stream("$.data[*]")
//!     .bearer_auth(&api_key)
//!     .get("https://api.openai.com/v1/models")
//!     .on_chunk(|model: Model| {
//!         Ok => model.into(),
//!         Err(e) => BadChunk::from_err(e)
//!     })
//!     .collect_or_else(|error| Model::default());
//! ```

mod core;
mod processing;
mod streaming;
mod utilities;

// Re-export main types
pub use core::JsonArrayStream;

pub use utilities::StreamStats;


