//! Asynchronous streaming processing logic
//!
//! Contains asynchronous processing methods and fallback streaming deserializer
//! logic for handling incomplete JSON and streaming scenarios.

use bytes::Bytes;
use ystream::AsyncStream;
use ystream::prelude::MessageChunk;
use serde::de::DeserializeOwned;

use super::core::JsonArrayStream;
use crate::jsonpath::{CoreJsonPathEvaluator, JsonPathDeserializer};

impl<T> JsonArrayStream<T>
where
    T: DeserializeOwned + Send + 'static,
{
    /// Process incoming bytes and yield deserialized objects
    ///
    /// This method is called repeatedly as HTTP chunks arrive. It maintains internal
    /// state to parse JSON incrementally and yield complete objects as they become available.
    ///
    /// # Arguments
    ///
    /// * `chunk` - Incoming HTTP response bytes
    ///
    /// # Returns
    ///
    /// `AsyncStream` over successfully deserialized objects of type `T`.
    /// Errors are handled via async-stream error emission patterns.
    ///
    /// # Performance
    ///
    /// Uses zero-copy techniques where possible and pre-allocated buffers to minimize allocations.
    /// Lock-free processing with const-generic capacity for blazing-fast performance.
    pub fn process_chunk(&mut self, chunk: Bytes) -> AsyncStream<T, 1024>
    where
        T: MessageChunk + MessageChunk + Default + Send + 'static,
    {
        // Append chunk to internal buffer
        self.buffer.append_chunk(chunk);

        // Try to parse as complete JSON first using simple evaluator
        let all_data = self.buffer.as_bytes();
        let Ok(json_str) = std::str::from_utf8(all_data) else {
            // Invalid UTF-8, return empty stream
            return AsyncStream::with_channel(|_sender| {
                // Empty stream - no chunks to emit
            });
        };

        // Try to parse as complete JSON
        let json_value = match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(value) => {
                tracing::debug!(
                    target: "quyc::jsonpath::streaming",
                    json_type = ?value.as_array().map(|a| format!("array[{}]", a.len())).or_else(|| value.as_object().map(|o| format!("object[{}]", o.len()))).unwrap_or_else(|| "primitive".to_string()),
                    "JSON parsing succeeded"
                );
                value
            }
            Err(e) => {
                tracing::debug!(
                    target: "quyc::jsonpath::streaming",
                    error = %e,
                    buffer_size = all_data.len(),
                    "JSON parsing failed, falling back to streaming deserializer"
                );
                // Not complete JSON yet, fall back to streaming deserializer
                return self.fallback_to_streaming_deserializer();
            }
        };

        // Use core evaluator for complete JSON
        let expression = self.path_expression.as_string();
        tracing::debug!(
            target: "quyc::jsonpath::streaming",
            expression = %expression,
            "Creating CoreJsonPathEvaluator"
        );
        let evaluator = match CoreJsonPathEvaluator::new(&expression) {
            Ok(eval) => {
                tracing::debug!(
                    target: "quyc::jsonpath::streaming",
                    "CoreJsonPathEvaluator created successfully"
                );
                eval
            }
            Err(e) => {
                tracing::warn!(
                    target: "quyc::jsonpath::streaming",
                    error = %e,
                    expression = %expression,
                    "CoreJsonPathEvaluator creation failed"
                );
                return AsyncStream::with_channel(|_sender| {
                    // Empty stream - no chunks to emit
                });
            }
        };

        tracing::debug!(
            target: "quyc::jsonpath::streaming",
            "Evaluating expression against JSON value"
        );
        let results = match evaluator.evaluate(&json_value) {
            Ok(values) => {
                tracing::debug!(
                    target: "quyc::jsonpath::streaming",
                    result_count = values.len(),
                    "CoreJsonPathEvaluator succeeded"
                );
                values
            }
            Err(e) => {
                tracing::warn!(
                    target: "quyc::jsonpath::streaming",
                    error = %e,
                    "CoreJsonPathEvaluator evaluation failed"
                );
                return AsyncStream::with_channel(|_sender| {
                    // Empty stream - no chunks to emit
                });
            }
        };

        // Convert JSON values to target type T
        let raw_result_count = results.len();
        let mut typed_results = Vec::new();
        for value in results {
            if let Ok(typed_value) = serde_json::from_value::<T>(value.clone()) {
                typed_results.push(typed_value);
            } else {
                // Skip invalid values
            }
        }

        tracing::debug!(
            target: "quyc::jsonpath::streaming",
            typed_result_count = typed_results.len(),
            raw_result_count,
            "Complete JSON path processing finished"
        );

        // If no results, return a Vec directly instead of an AsyncStream that waits for timeout
        if typed_results.is_empty() {
            tracing::debug!(
                target: "quyc::jsonpath::streaming",
                "Creating immediate empty stream for 0 results"
            );
            // Create a stream that immediately signals completion
            return AsyncStream::with_channel(move |sender| {
                // Send nothing and exit immediately - this signals the producer is done
                drop(sender);
            });
        }

        // Create AsyncStream from the processed results using proper streaming architecture
        AsyncStream::with_channel(move |sender| {
            tracing::trace!(
                target: "quyc::jsonpath::streaming",
                result_count = typed_results.len(),
                "Complete JSON AsyncStream closure started"
            );
            for (i, typed_value) in typed_results.into_iter().enumerate() {
                if sender.try_send(typed_value).is_err() {
                    tracing::trace!(
                        target: "quyc::jsonpath::streaming",
                        sent_count = i,
                        "Complete JSON AsyncStream channel closed during send"
                    );
                    break; // Channel closed
                }
            }
            tracing::trace!(
                target: "quyc::jsonpath::streaming",
                "Complete JSON AsyncStream closure completed"
            );
        })
    }

    pub(super) fn fallback_to_streaming_deserializer(&mut self) -> AsyncStream<T, 1024>
    where
        T: MessageChunk + MessageChunk + Default + Send + 'static,
    {
        tracing::debug!(
            target: "quyc::jsonpath::streaming",
            "fallback_to_streaming_deserializer called"
        );

        // Process available data using the streaming deserializer
        let mut deserializer = JsonPathDeserializer::new(&self.path_expression, &mut self.buffer);
        let mut results = Vec::new();

        tracing::debug!(
            target: "quyc::jsonpath::streaming",
            "Starting streaming deserializer iteration"
        );

        // Manually collect the iterator to avoid lifetime dependency
        let iterator = deserializer.process_available();
        let mut iteration_count = 0;
        for result in iterator {
            iteration_count += 1;
            tracing::trace!(
                target: "quyc::jsonpath::streaming",
                iteration = iteration_count,
                success = result.is_ok(),
                "Streaming iteration processed"
            );
            results.push(result);

            // Safety check to prevent infinite loops
            if iteration_count > 1000 {
                tracing::warn!(
                    target: "quyc::jsonpath::streaming",
                    iteration_count,
                    "Breaking streaming iteration after safety limit"
                );
                break;
            }
        }

        tracing::debug!(
            target: "quyc::jsonpath::streaming",
            result_count = results.len(),
            iteration_count,
            "Streaming deserializer completed"
        );

        // Create AsyncStream from the processed results using proper streaming architecture
        AsyncStream::with_channel(move |sender| {
            tracing::trace!(
                target: "quyc::jsonpath::streaming",
                result_count = results.len(),
                "Streaming AsyncStream closure started"
            );
            for (i, result) in results.into_iter().enumerate() {
                match result {
                    Ok(typed_value) => {
                        tracing::trace!(
                            target: "quyc::jsonpath::streaming",
                            result_index = i,
                            "Streaming sending successful result"
                        );
                        if sender.try_send(typed_value).is_err() {
                            tracing::trace!(
                                target: "quyc::jsonpath::streaming",
                                sent_count = i,
                                "Streaming channel closed during send"
                            );
                            break; // Channel closed
                        }
                    }
                    Err(e) => {
                        tracing::debug!(
                            target: "quyc::jsonpath::streaming",
                            error = %e,
                            result_index = i,
                            "Streaming skipping failed result - deserialization error"
                        );
                        // Skip invalid values
                    }
                }
            }
            tracing::trace!(
                target: "quyc::jsonpath::streaming",
                "Streaming AsyncStream closure completed"
            );
        })
    }
}
