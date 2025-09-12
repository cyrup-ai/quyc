use serde::de::DeserializeOwned;

use super::types::JsonPathDeserializer;

impl<T> JsonPathDeserializer<'_, T>
where
    T: DeserializeOwned,
{
    /// Read next byte from buffer with position tracking
    ///
    /// # Errors
    /// Returns `JsonPathError` if buffer access fails or position tracking encounters errors
    #[inline]
    pub fn read_next_byte(&mut self) -> crate::jsonpath::error::JsonPathResult<Option<u8>> {
        // Check if we've reached the end of available data
        if self.buffer_position >= self.buffer.len() {
            return Ok(None); // No more data available
        }

        // Read byte at current position
        match self.buffer.get_byte_at(self.buffer_position) {
            Some(byte) => {
                self.buffer_position += 1;
                Ok(Some(byte))
            }
            None => Ok(None), // Position beyond buffer bounds
        }
    }
}
