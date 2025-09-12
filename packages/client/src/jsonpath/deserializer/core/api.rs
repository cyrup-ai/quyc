//! Public API methods for `JsonPathDeserializer`
//!
//! Contains the main public interface methods for streaming JSON deserialization
//! and buffer management operations.

use serde::de::DeserializeOwned;

use super::super::iterator::JsonPathIterator;
use super::types::JsonPathDeserializer;

impl<'a, T> JsonPathDeserializer<'a, T>
where
    T: DeserializeOwned,
{
    /// Process next chunk of JSON data through the deserializer
    ///
    /// # Arguments
    ///
    /// * `chunk` - JSON data chunk to process
    ///
    /// # Returns
    ///
    /// Iterator over deserialized values matching the `JSONPath` expression
    pub fn process_chunk(&mut self, chunk: &[u8]) -> JsonPathIterator<'_, 'a, T> {
        self.buffer
            .append_chunk(&bytes::Bytes::copy_from_slice(chunk));
        JsonPathIterator::new(self)
    }
}
