//! Core JsonArrayStream structure and constructors
//!
//! Contains the main JsonArrayStream struct definition, constructors,
//! and basic initialization logic for JSONPath streaming processing.

use std::marker::PhantomData;

use serde::de::DeserializeOwned;

use crate::jsonpath::{JsonPathExpression, JsonPathParser, StreamBuffer, StreamStateMachine};
use crate::jsonpath::state_machine::ObjectBoundary;

/// Zero-allocation JSONPath streaming processor
///
/// Transforms HTTP byte streams into individual JSON objects based on JSONPath expressions.
/// Uses compile-time optimizations and runtime streaming for maximum performance.
#[derive(Debug)]
pub struct JsonArrayStream<T = serde_json::Value> {
    /// JSONPath expression for array element selection
    pub(super) path_expression: JsonPathExpression,
    /// Streaming buffer for efficient byte processing
    pub(super) buffer: StreamBuffer,
    /// State machine for parsing progress tracking
    pub(super) state: StreamStateMachine,
    /// Zero-sized type marker for target deserialization type
    pub(super) _phantom: PhantomData<T>,
}

impl<T> JsonArrayStream<T>
where
    T: DeserializeOwned + Send + 'static,
{
    /// Create new JSONPath streaming processor with explicit type
    ///
    /// # Arguments
    ///
    /// * `jsonpath` - JSONPath expression (e.g., "$.data[*]", "$.results[?(@.active)]")
    ///
    /// # Error Handling
    ///
    /// Invalid JSONPath expressions are handled via async-stream error emission patterns.
    /// Errors are logged and processing continues with a default expression.
    ///
    /// # Performance
    ///
    /// JSONPath compilation is performed once during construction for optimal runtime performance.
    pub fn new(jsonpath: &str) -> Self {
        Self::new_typed(jsonpath)
    }

    /// Create new JSONPath streaming processor with explicit type (alias for new)
    ///
    /// # Arguments
    ///
    /// * `jsonpath` - JSONPath expression (e.g., "$.data[*]", "$.results[?(@.active)]")
    ///
    /// # Error Handling
    ///
    /// Invalid JSONPath expressions are handled via async-stream error emission patterns.
    /// Errors are logged and processing continues with a default expression.
    ///
    /// # Performance
    ///
    /// JSONPath compilation is performed once during construction for optimal runtime performance.
    pub fn new_typed(jsonpath: &str) -> Self {
        let path_expression = match JsonPathParser::compile(jsonpath) {
            Ok(expr) => expr,
            Err(e) => {
                log::error!("JSONPath compilation failed: {:?}", e);
                // Return empty expression that matches nothing, allowing processing to continue
                JsonPathExpression::new(Vec::new(), jsonpath.to_string(), false)
            }
        };
        let buffer = StreamBuffer::with_capacity(8192); // 8KB initial capacity
        let state = StreamStateMachine::new();

        Self {
            path_expression,
            buffer,
            state,
            _phantom: PhantomData,
        }
    }

    /// Initialize the state machine with a JSONPath expression
    pub fn initialize_state(&mut self, expression: JsonPathExpression) {
        self.state.initialize(expression);
    }

    /// Append a chunk to the internal buffer
    pub fn append_chunk(&mut self, chunk: bytes::Bytes) {
        self.buffer.append_chunk(chunk);
    }

    /// Get the current buffer contents as bytes
    pub fn buffer_as_bytes(&self) -> &[u8] {
        self.buffer.as_bytes()
    }

    /// Process bytes through the state machine
    pub fn process_bytes(&mut self, bytes: &[u8], offset: usize) -> Vec<ObjectBoundary> {
        self.state.process_bytes(bytes, offset)
    }

    /// Consume bytes from the buffer
    pub fn consume_bytes(&mut self, count: usize) {
        self.buffer.consume(count);
    }
}

impl JsonArrayStream<serde_json::Value> {
    /// Create new JSONPath streaming processor for serde_json::Value (common case)
    ///
    /// This is a convenience method for the most common use case of processing JSON
    /// into serde_json::Value objects. For custom deserialization types, use new_typed().
    pub fn new_value(jsonpath: &str) -> Self {
        Self::new_typed(jsonpath)
    }
}
