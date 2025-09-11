//! Core JSON byte processing logic
//!
//! Contains the main entry points and core logic for processing individual JSON bytes
//! during streaming, including state transitions and basic byte reading.

use serde::de::DeserializeOwned;

use super::super::iterator::JsonPathIterator;
use super::super::byte_processor::{JsonByteProcessor, SharedByteProcessor};
use crate::jsonpath::error::JsonPathResult;

/// Result of processing a single JSON byte
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonProcessResult {
    /// Continue processing more bytes
    Continue,
    /// Complete JSON object found and ready for deserialization
    ObjectFound,
    /// Need more data to continue parsing
    NeedMoreData,
    /// Processing complete (end of stream)
    Complete,
}

impl<T> JsonPathIterator<'_, '_, T>
where
    T: DeserializeOwned,
{
    /// Read next byte from streaming buffer using shared byte processor
    #[inline]
    pub(super) fn read_next_byte(&mut self) -> JsonPathResult<Option<u8>> {
        let mut processor = SharedByteProcessor::new(
            self.deserializer.buffer,
            self.deserializer.buffer_position
        );
        let result = processor.read_next_byte();
        self.deserializer.buffer_position = processor.position();
        self.bytes_consumed += processor.bytes_consumed();
        result
    }

    /// Process single JSON byte using shared processor
    #[inline]
    pub(super) fn process_json_byte(&mut self, byte: u8) -> JsonPathResult<JsonProcessResult> {
        match &self.deserializer.state {
            super::super::core::DeserializerState::Initial => Ok(self.process_initial_byte(byte)),
            super::super::core::DeserializerState::Navigating => self.process_navigating_byte(byte),
            super::super::core::DeserializerState::ProcessingArray => Ok(self.process_array_byte(byte)),
            super::super::core::DeserializerState::ProcessingObject => {
                self.process_object_byte(byte)
            }
            super::super::core::DeserializerState::Complete => Ok(JsonProcessResult::Complete),
        }
    }

    /// Skip over JSON string content including escaped characters
    pub(super) fn skip_string_content(&mut self) -> JsonPathResult<()> {
        let mut escaped = false;
        let mut bytes_processed = 0;
        const MAX_STRING_BYTES: usize = 1024 * 1024; // 1MB limit for JSON strings

        while let Some(byte) = self.read_next_byte()? {
            bytes_processed += 1;

            // Prevent infinite loops on malformed JSON
            if bytes_processed > MAX_STRING_BYTES {
                return Err(crate::jsonpath::error::json_parse_error(
                    "JSON string too long or unterminated".to_string(),
                    self.bytes_consumed,
                    "string parsing".to_string(),
                ));
            }
            if self.deserializer.in_target_array {
                self.deserializer.object_buffer.push(byte);
            }

            if escaped {
                escaped = false;
            } else {
                match byte {
                    b'"' => return Ok(()), // End of string
                    b'\\' => escaped = true,
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
