//! General-purpose streaming deserializer utilities
//!
//! Provides simplified streaming JSON deserialization without JSONPath navigation
//! for use cases that don't require complex path-based filtering.

use std::io::Read;

use serde::de::DeserializeOwned;
use serde_json::{Deserializer, StreamDeserializer, de::IoRead};

/// Streaming deserializer for general use
///
/// Provides a simplified interface for streaming JSON deserialization without
/// JSONPath navigation. Useful for streaming simple arrays directly.
pub struct StreamingDeserializer<R: Read, T> {
    inner: StreamDeserializer<'static, IoRead<R>, T>,
}

impl<R: Read, T> StreamingDeserializer<R, T>
where
    T: DeserializeOwned,
{
    /// Create new streaming deserializer from reader
    ///
    /// # Arguments
    ///
    /// * `reader` - Source of JSON bytes
    #[inline]
    pub fn new(reader: R) -> Self {
        let inner = Deserializer::from_reader(reader).into_iter::<T>();
        Self { inner }
    }
}

impl<R: Read, T> Iterator for StreamingDeserializer<R, T>
where
    T: DeserializeOwned,
{
    type Item = Result<T, serde_json::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
