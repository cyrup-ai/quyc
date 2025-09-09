//! Iterator implementation for JSONPath deserializer
//!
//! Provides iterator interface for streaming JSON deserialization
//! with JSONPath expression evaluation.

use serde_json::Value;

use crate::jsonpath::error::JsonPathResult;

/// Iterator for JSONPath deserialization results
pub struct JsonPathIterator<'a, T> {
    buffer: Vec<Value>,
    position: usize,
    finished: bool,
    _phantom: std::marker::PhantomData<(&'a (), T)>,
}

impl<'a, T> JsonPathIterator<'a, T> {
    /// Create a new iterator with initial buffer
    pub fn new(initial_buffer: Vec<Value>) -> Self {
        Self {
            buffer: initial_buffer,
            position: 0,
            finished: false,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create an empty iterator
    pub fn empty() -> Self {
        Self {
            buffer: Vec::new(),
            position: 0,
            finished: true,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add more values to the iterator buffer
    pub fn extend(&mut self, values: Vec<Value>) {
        self.buffer.extend(values);
    }

    /// Mark the iterator as finished (no more values will be added)
    pub fn finish(&mut self) {
        self.finished = true;
    }

    /// Check if the iterator has more values
    pub fn has_next(&self) -> bool {
        self.position < self.buffer.len() || !self.finished
    }

    /// Get the next value if available
    pub fn next_value(&mut self) -> Option<Value> {
        if self.position < self.buffer.len() {
            let value = self.buffer[self.position].clone();
            self.position += 1;
            Some(value)
        } else {
            None
        }
    }

    /// Reset the iterator to the beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Get the current position in the buffer
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the total number of buffered values
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the iterator buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl<'a, T> Iterator for JsonPathIterator<'a, T> {
    type Item = JsonPathResult<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_value().map(Ok)
    }
}
