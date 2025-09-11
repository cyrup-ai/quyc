//! State-specific JSON byte processors
//!
//! Contains specialized byte processing logic for each JSON parsing state,
//! including initial, navigating, array, and object processing states.

use serde::de::DeserializeOwned;

use super::super::iterator::JsonPathIterator;
use super::core::JsonProcessResult;
use crate::jsonpath::error::JsonPathResult;

impl<T> JsonPathIterator<'_, '_, T>
where
    T: DeserializeOwned,
{
    /// Process byte when parser is in initial state
    #[allow(dead_code)]
    #[inline]
    pub(super) fn process_initial_byte(&mut self, byte: u8) -> JsonProcessResult {
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => JsonProcessResult::Continue, // Skip whitespace
            b'{' => {
                self.deserializer.transition_to_processing_object();
                self.deserializer.object_nesting =
                    self.deserializer.object_nesting.saturating_add(1);
                if self.matches_root_object_path() {
                    JsonProcessResult::Continue
                } else {
                    JsonProcessResult::Continue
                }
            }
            b'[' => {
                self.deserializer.transition_to_processing_array();
                self.deserializer.current_depth = self.deserializer.current_depth.saturating_add(1);
                // Push current array index to stack for nested arrays
                self.deserializer
                    .array_index_stack
                    .push(self.deserializer.current_array_index);
                self.deserializer.current_array_index = 0; // Reset for new array
                if self.matches_root_array_path() {
                    self.deserializer.in_target_array = true;
                }
                JsonProcessResult::Continue
            }
            _ => JsonProcessResult::Continue,
        }
    }

    /// Process byte when navigating through JSON structure
    pub(super) fn process_navigating_byte(
        &mut self,
        byte: u8,
    ) -> JsonPathResult<JsonProcessResult> {
        // Check for recursive descent mode entry before processing
        if !self.deserializer.streaming_state.in_recursive_descent && self.should_enter_recursive_descent() {
            self.enter_recursive_descent_mode();
        }

        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => Ok(JsonProcessResult::Continue), // Skip whitespace
            b'{' => {
                self.deserializer.object_nesting =
                    self.deserializer.object_nesting.saturating_add(1);
                self.deserializer.transition_to_processing_object();

                // Update breadcrumbs for recursive descent tracking
                if self.deserializer.streaming_state.in_recursive_descent {
                    self.update_breadcrumbs(None); // Object entry
                }

                if self.matches_current_path() && self.deserializer.in_target_array {
                    self.deserializer.object_buffer.clear();
                    self.deserializer.object_buffer.push(byte);
                    Ok(JsonProcessResult::Continue)
                } else {
                    Ok(JsonProcessResult::Continue)
                }
            }
            b'[' => {
                self.deserializer.current_depth = self.deserializer.current_depth.saturating_add(1);
                self.deserializer.transition_to_processing_array();

                // Update breadcrumbs for recursive descent tracking
                if self.deserializer.streaming_state.in_recursive_descent {
                    self.update_breadcrumbs(None); // Array entry
                }

                // Push current array index to stack for nested arrays
                self.deserializer
                    .array_index_stack
                    .push(self.deserializer.current_array_index);
                self.deserializer.current_array_index = 0; // Reset for new array
                if self.matches_current_path() {
                    self.deserializer.in_target_array = true;
                }
                Ok(JsonProcessResult::Continue)
            }
            b']' => {
                if self.deserializer.in_target_array {
                    self.deserializer.in_target_array = false;
                }
                self.deserializer.current_depth = self.deserializer.current_depth.saturating_sub(1);
                // Restore previous array index from stack
                if let Some(prev_index) = self.deserializer.array_index_stack.pop() {
                    self.deserializer.current_array_index = prev_index;
                }
                if self.deserializer.current_depth == 0 {
                    self.deserializer.transition_to_complete();
                    Ok(JsonProcessResult::Complete)
                } else {
                    Ok(JsonProcessResult::Continue)
                }
            }
            b'}' => {
                self.deserializer.object_nesting =
                    self.deserializer.object_nesting.saturating_sub(1);
                if self.deserializer.object_nesting == 0 {
                    self.deserializer.transition_to_complete();
                    Ok(JsonProcessResult::Complete)
                } else {
                    Ok(JsonProcessResult::Continue)
                }
            }
            b',' => Ok(JsonProcessResult::Continue), // Array/object separator
            _ => Ok(JsonProcessResult::Continue),
        }
    }

    /// Process byte when inside target array
    pub(super) fn process_array_byte(&mut self, byte: u8) -> JsonProcessResult {
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => JsonProcessResult::Continue, // Skip whitespace
            b'{' => {
                if self.deserializer.in_target_array && self.matches_current_path() {
                    self.deserializer.object_buffer.clear();
                    self.deserializer.object_buffer.push(byte);
                    self.deserializer.transition_to_processing_object();
                    self.deserializer.object_nesting = 1;
                    JsonProcessResult::Continue
                } else {
                    JsonProcessResult::Continue
                }
            }
            b'[' => {
                self.deserializer.current_depth = self.deserializer.current_depth.saturating_add(1);
                self.deserializer.transition_to_processing_array();
                // Push current array index to stack for nested arrays
                self.deserializer
                    .array_index_stack
                    .push(self.deserializer.current_array_index);
                self.deserializer.current_array_index = 0; // Reset for new array
                JsonProcessResult::Continue
            }
            b']' => {
                // Check if we have a remaining object to process before closing array
                if self.deserializer.in_target_array && !self.deserializer.object_buffer.is_empty()
                {
                    // Last object in array - process it before closing
                    let result = JsonProcessResult::ObjectFound;
                    self.deserializer.in_target_array = false;
                    self.deserializer.current_depth =
                        self.deserializer.current_depth.saturating_sub(1);
                    if let Some(prev_index) = self.deserializer.array_index_stack.pop() {
                        self.deserializer.current_array_index = prev_index;
                    }
                    return result;
                }

                if self.deserializer.in_target_array {
                    self.deserializer.in_target_array = false;
                }
                self.deserializer.current_depth = self.deserializer.current_depth.saturating_sub(1);
                // Restore previous array index from stack
                if let Some(prev_index) = self.deserializer.array_index_stack.pop() {
                    self.deserializer.current_array_index = prev_index;
                }
                if self.deserializer.current_depth == 0 {
                    self.deserializer.transition_to_complete();
                    JsonProcessResult::Complete
                } else {
                    JsonProcessResult::Continue
                }
            }
            b',' => {
                if self.deserializer.in_target_array && !self.deserializer.object_buffer.is_empty()
                {
                    // Complete object found - object_buffer contains a full JSON object
                    JsonProcessResult::ObjectFound
                } else {
                    // Increment array index for next element
                    self.deserializer.current_array_index =
                        self.deserializer.current_array_index.saturating_add(1);
                    JsonProcessResult::Continue
                }
            }
            _ => {
                if self.deserializer.in_target_array {
                    self.deserializer.object_buffer.push(byte);
                }
                JsonProcessResult::Continue
            }
        }
    }

    /// Process byte when inside JSON object
    pub(super) fn process_object_byte(&mut self, byte: u8) -> JsonPathResult<JsonProcessResult> {
        if self.deserializer.in_target_array {
            self.deserializer.object_buffer.push(byte);
        }

        match byte {
            b'"' => {
                self.skip_string_content()?;
                Ok(JsonProcessResult::Continue)
            }
            b'{' => {
                self.deserializer.object_nesting =
                    self.deserializer.object_nesting.saturating_add(1);
                Ok(JsonProcessResult::Continue)
            }
            b'}' => {
                self.deserializer.object_nesting =
                    self.deserializer.object_nesting.saturating_sub(1);
                if self.deserializer.object_nesting == 0 {
                    // Complete object found - check if we need to exit recursive descent
                    if self.deserializer.streaming_state.in_recursive_descent {
                        // Evaluate if we should continue or exit recursive descent
                        if !self.evaluate_recursive_descent_match() {
                            self.exit_recursive_descent_mode();
                        }
                    }
                    Ok(JsonProcessResult::ObjectFound)
                } else {
                    Ok(JsonProcessResult::Continue)
                }
            }
            _ => Ok(JsonProcessResult::Continue),
        }
    }
}
