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

pub mod buffer;
pub mod core_evaluator;
pub mod deserializer;
pub mod error;
pub mod filter;
pub mod functions;
pub mod json_array_stream;
pub mod normalized_paths;
pub mod null_semantics;
pub mod parser;
pub mod safe_parsing;
pub mod state_machine;
pub mod stats;
pub mod stream_processor;
pub mod type_system;

// Decomposed parser modules
pub mod ast;
pub mod compiler;
pub mod expression;
pub mod filter_parser;
pub mod selector_parser;
pub mod tokenizer;
pub mod tokens;

pub use self::{
    buffer::StreamBuffer,
    core_evaluator::CoreJsonPathEvaluator,
    deserializer::{JsonPathDeserializer, JsonPathIterator, StreamingDeserializer},
    error::{JsonPathError, JsonPathResult},
    filter::FilterEvaluator,
    functions::FunctionEvaluator,
    json_array_stream::{JsonArrayStream, StreamStats},
    parser::{
        ComparisonOp,
        ComplexityMetrics,
        FilterExpression,
        FilterValue,
        FunctionSignature,
        // RFC 9535 Implementation Types
        FunctionType,
        JsonPathExpression,
        JsonPathParser,
        JsonSelector,
        NormalizedPath,
        NormalizedPathProcessor,
        NullSemantics,
        PathSegment,
        PropertyAccessResult,
        TypeSystem,
        TypedValue,
    },
    safe_parsing::{SafeParsingContext, SafeStringBuffer, Utf8Handler, Utf8RecoveryStrategy},
    state_machine::{JsonStreamState, StreamStateMachine},
    stream_processor::JsonStreamProcessor,
};
// Re-export JsonBuffer from telemetry for backward compatibility
pub use crate::telemetry::jsonpath::JsonBuffer;

use ystream::{AsyncStream, AsyncStreamSender};
use ystream::prelude::MessageChunk;
use serde::de::DeserializeOwned;
use crate::http::response::{HttpResponse, HttpChunk};

/// Process an HTTP response through JSONPath streaming infrastructure
/// 
/// This is the main entry point that coordinates all JSONPath components:
/// - Extracts body stream from HttpResponse
/// - Creates streaming buffer with 8KB initial capacity  
/// - Initializes state machine for incremental parsing
/// - Compiles JSONPath expression once
/// - Processes chunks as they arrive
/// - Deserializes matching objects in real-time
/// - Emits them through AsyncStream
///
/// # Arguments
/// * `response` - The HTTP response to process
/// * `jsonpath_expr` - JSONPath expression to match objects (e.g., "$.data[*]")
///
/// # Returns
/// AsyncStream of deserialized objects matching the JSONPath expression
///
/// # Type Parameters
/// * `T` - The type to deserialize matching JSON objects into
pub fn process_response<T>(response: HttpResponse, jsonpath_expr: &str) -> AsyncStream<T, 1024>
where
    T: DeserializeOwned + MessageChunk + Default + Send + 'static,
{
    let jsonpath_expr = jsonpath_expr.to_string();
    
    AsyncStream::with_channel(move |sender: AsyncStreamSender<T>| {
        // 1. Extract body stream from response
        let body_stream = response.into_body_stream();
        
        // 2. Create JSONPath components with the expression
        let mut json_array_stream = JsonArrayStream::<T>::new(&jsonpath_expr);
        
        // 3. Compile JSONPath expression once
        let compiled_expr = match JsonPathParser::compile(&jsonpath_expr) {
            Ok(expr) => expr,
            Err(e) => {
                log::error!("JSONPath compilation failed: {} for expression: {}", e, jsonpath_expr);
                // Return early on invalid JSONPath
                return;
            }
        };
        
        // 4. Initialize state machine with compiled expression
        json_array_stream.initialize_state(compiled_expr.clone());
        
        // 5. Create evaluator for matching
        let evaluator = match CoreJsonPathEvaluator::new(&jsonpath_expr) {
            Ok(eval) => eval,
            Err(e) => {
                log::error!("Failed to create JSONPath evaluator: {}", e);
                return;
            }
        };
        
        // 6. Process each body chunk through the infrastructure
        let mut total_bytes_processed = 0usize;
        
        for body_chunk in body_stream {
            // Get the data from the body chunk
            let chunk_data = body_chunk.data;
            let chunk_len = chunk_data.len();
            
            // Feed chunk to buffer
            json_array_stream.append_chunk(chunk_data);
            
            // Get current buffer content as a Vec to avoid borrowing conflicts
            let buffer_bytes_vec = json_array_stream.buffer_as_bytes().to_vec();
            
            // Process through state machine to find object boundaries
            let boundaries = json_array_stream.process_bytes(
                &buffer_bytes_vec,
                total_bytes_processed
            );
            
            total_bytes_processed += chunk_len;
            
            // For each complete object found
            for boundary in boundaries {
                // Extract object bytes from buffer
                let object_bytes = &buffer_bytes_vec[boundary.start..boundary.end];
                
                // Try to parse as JSON value
                match serde_json::from_slice::<serde_json::Value>(object_bytes) {
                    Ok(json_value) => {
                        // Evaluate JSONPath against this object
                        match evaluator.evaluate(&json_value) {
                            Ok(matches) => {
                                // Process each matching value
                                for matched_value in matches {
                                    // Deserialize to target type
                                    match serde_json::from_value::<T>(matched_value) {
                                        Ok(typed_obj) => {
                                            // Emit through AsyncStream
                                            if sender.send(typed_obj).is_err() {
                                                // Stream closed by consumer
                                                return;
                                            }
                                        }
                                        Err(e) => {
                                            log::debug!("Failed to deserialize matched object: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::debug!("JSONPath evaluation error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::debug!("Failed to parse JSON object: {}", e);
                        // Continue processing - might be incomplete JSON
                    }
                }
                
                // Clear processed bytes from buffer
                json_array_stream.consume_bytes(boundary.end);
            }
        }
        
        // Final check for any remaining complete JSON in buffer
        let remaining_bytes = json_array_stream.buffer_as_bytes();
        if !remaining_bytes.is_empty() {
            // Try to parse remaining content as complete JSON
            match serde_json::from_slice::<serde_json::Value>(remaining_bytes) {
                Ok(json_value) => {
                    // Evaluate JSONPath one last time
                    if let Ok(matches) = evaluator.evaluate(&json_value) {
                        for matched_value in matches {
                            if let Ok(typed_obj) = serde_json::from_value::<T>(matched_value) {
                                let _ = sender.send(typed_obj);
                            }
                        }
                    }
                }
                Err(_) => {
                    // Incomplete JSON at end - this is expected for streaming
                }
            }
        }
    })
}

/// Process an HTTP response for single JSON object deserialization
/// 
/// This function handles regular JSON responses (not JSONPath streaming) by:
/// - Extracting the body stream from HttpResponse
/// - Accumulating all chunks into a complete JSON document
/// - Deserializing the complete JSON to the target type
/// - Emitting the result through AsyncStream
///
/// # Arguments
/// * `response` - The HTTP response to process
///
/// # Returns
/// AsyncStream containing the single deserialized object
///
/// # Type Parameters
/// * `T` - The type to deserialize the JSON response into
pub fn process_json_response<T>(response: HttpResponse) -> AsyncStream<T, 1024>
where
    T: DeserializeOwned + MessageChunk + Default + Send + 'static,
{
    AsyncStream::with_channel(move |sender: AsyncStreamSender<T>| {
        // Extract body stream from response
        let body_stream = response.into_body_stream();
        let mut body_buffer = Vec::new();
        
        // Accumulate all body chunks
        for body_chunk in body_stream {
            body_buffer.extend_from_slice(&body_chunk.data);
        }
        
        // Deserialize complete JSON
        match serde_json::from_slice::<T>(&body_buffer) {
            Ok(deserialized) => {
                // Emit the deserialized object
                if sender.send(deserialized).is_err() {
                    // Stream closed by consumer
                    return;
                }
            }
            Err(e) => {
                // Create error chunk for deserialization failure
                let error_chunk = T::bad_chunk(format!("JSON deserialization failed: {}", e));
                let _ = sender.send(error_chunk);
            }
        }
    })
}

/// Process an HTTP response for raw chunk streaming
/// 
/// This function converts HttpResponse body chunks to HttpChunk format by:
/// - Extracting the body stream from HttpResponse
/// - Converting each HttpBodyChunk to HttpChunk::Body
/// - Emitting an HttpChunk::End marker when complete
/// - Emitting chunks as they arrive for true streaming
///
/// # Arguments
/// * `response` - The HTTP response to process
///
/// # Returns
/// AsyncStream of HttpChunk objects for raw response processing
pub fn process_raw_response(response: HttpResponse) -> AsyncStream<HttpChunk, 1024> {
    AsyncStream::with_channel(move |sender: AsyncStreamSender<HttpChunk>| {
        // Extract body stream from response
        let body_stream = response.into_body_stream();
        
        // Stream each body chunk as HttpChunk
        for body_chunk in body_stream {
            let http_chunk = HttpChunk::Body(body_chunk.data);
            if sender.send(http_chunk).is_err() {
                // Stream closed by consumer
                return;
            }
        }
        
        // Emit end marker
        let _ = sender.send(HttpChunk::End);
    })
}


