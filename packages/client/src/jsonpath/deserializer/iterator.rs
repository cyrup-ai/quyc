//! Iterator implementation for streaming JSON objects
//!
//! Provides the JsonPathIterator that yields deserialized objects as they
//! become available during streaming JSON parsing.

use serde::de::DeserializeOwned;
use serde_json::{StreamDeserializer, de::IoRead};

use super::{core::JsonPathDeserializer, processor::JsonProcessResult};
use crate::jsonpath::{buffer::BufferReader, error::JsonPathResult};

/// Iterator over streaming JSON objects matching JSONPath expression
pub struct JsonPathIterator<'iter, 'data, T> {
    pub(super) deserializer: &'iter mut JsonPathDeserializer<'data, T>,
    pub(super) stream_deserializer:
        Option<StreamDeserializer<'static, IoRead<BufferReader<'data>>, T>>,
    pub(super) bytes_consumed: usize,
}

impl<'iter, 'data, T> JsonPathIterator<'iter, 'data, T>
where
    T: DeserializeOwned,
{
    /// Create new iterator over streaming JSON objects
    #[inline]
    pub(super) fn new(deserializer: &'iter mut JsonPathDeserializer<'data, T>) -> Self {
        Self {
            deserializer,
            stream_deserializer: None,
            bytes_consumed: 0,
        }
    }

    /// Advance to next complete JSON object matching JSONPath
    ///
    /// Incrementally parses JSON structure while evaluating JSONPath expressions
    /// to identify individual array elements for deserialization.
    #[inline]
    fn advance_to_next_object(&mut self) -> JsonPathResult<Option<T>> {
        // Fast path: if we have a stream deserializer, try to get next object
        if let Some(ref mut stream_deser) = self.stream_deserializer {
            match stream_deser.next() {
                Some(Ok(obj)) => return Ok(Some(obj)),
                Some(Err(e)) => {
                    // Reset stream deserializer on error and continue parsing
                    self.stream_deserializer = None;
                    return Err(crate::jsonpath::error::json_parse_error(
                        format!("JSON deserialization failed: {}", e),
                        self.bytes_consumed,
                        "streaming array element".to_string(),
                    ));
                }
                None => {
                    // Stream deserializer exhausted, continue parsing
                    self.stream_deserializer = None;
                }
            }
        }

        // Parse JSON incrementally to find next object matching JSONPath
        // Add bounds check to prevent infinite loops with filter expressions
        let mut iteration_count = 0;
        let max_iterations = 10000; // Reasonable limit for JSON parsing

        loop {
            iteration_count += 1;
            if iteration_count > max_iterations {
                log::warn!(
                    "JsonPath iterator exceeded maximum iterations - likely infinite loop in filter evaluation"
                );
                return Ok(None); // Exit gracefully rather than hanging
            }

            let byte = match self.deserializer.read_next_byte()? {
                Some(b) => b,
                None => return Ok(None), // No more data available
            };

            match self.deserializer.process_json_byte(byte)? {
                JsonProcessResult::ObjectFound => {
                    // Found complete object, attempt deserialization
                    match self.deserializer.deserialize_current_object() {
                        Ok(Some(obj)) => return Ok(Some(obj)),
                        Ok(None) => {}, // Continue parsing if no object available
                        Err(e) => return Err(e), // Return deserialization error to caller
                    }
                }
                JsonProcessResult::Continue => {
                    // Continue parsing
                }
                JsonProcessResult::NeedMoreData => {
                    // Need more bytes to complete parsing
                    return Ok(None);
                }
                JsonProcessResult::Complete => {
                    // Processing complete (end of stream)
                    return Ok(None);
                }
            }
        }
    }
}

impl<'iter, 'data, T> Iterator for JsonPathIterator<'iter, 'data, T>
where
    T: DeserializeOwned,
{
    type Item = JsonPathResult<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.advance_to_next_object() {
            Ok(Some(obj)) => Some(Ok(obj)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
