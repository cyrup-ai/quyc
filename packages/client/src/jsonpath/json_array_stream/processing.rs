//! Synchronous processing logic for JsonArrayStream
//!
//! Contains synchronous processing methods for complete JSON documents,
//! including chunk processing and JSON evaluation logic.

use bytes::Bytes;
use serde::de::DeserializeOwned;

use super::core::JsonArrayStream;
use crate::jsonpath::CoreJsonPathEvaluator;

impl<T> JsonArrayStream<T>
where
    T: DeserializeOwned + Send + 'static,
{
    /// Process incoming bytes and return results as Vec for complete JSON
    ///
    /// This method processes complete JSON immediately without streaming timeouts.
    /// Used internally when JSON parsing succeeds to avoid AsyncStream timeout issues.
    pub fn process_chunk_sync(&mut self, chunk: Bytes) -> Vec<T> {
        // Append chunk to internal buffer
        self.buffer.append_chunk(chunk);

        // Try to parse as complete JSON first using simple evaluator
        let all_data = self.buffer.as_bytes();
        let json_str = match std::str::from_utf8(all_data) {
            Ok(s) => s,
            Err(_) => return Vec::new(), // Invalid UTF-8
        };

        // Try to parse as complete JSON
        let json_value = match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(value) => value,
            Err(_) => return Vec::new(), // Not complete JSON
        };

        // Use core evaluator for complete JSON
        let expression = self.path_expression.as_string();
        let evaluator = match CoreJsonPathEvaluator::new(&expression) {
            Ok(eval) => eval,
            Err(_) => return Vec::new(),
        };

        let results = match evaluator.evaluate(&json_value) {
            Ok(values) => values,
            Err(_) => return Vec::new(),
        };

        // Convert JSON values to target type T
        let mut typed_results = Vec::new();
        for value in results {
            match serde_json::from_value::<T>(value.clone()) {
                Ok(typed_value) => {
                    typed_results.push(typed_value);
                }
                Err(_) => {
                    // Skip invalid values
                }
            }
        }

        typed_results
    }
}
