//! JSON byte processing and state machine logic
//!
//! Contains the core byte processing methods that handle JSON parsing
//! and state transitions during streaming deserialization.

use serde::de::DeserializeOwned;

use super::types::{DeserializerState, JsonPathDeserializer};

impl<T> JsonPathDeserializer<'_, T>
where
    T: DeserializeOwned,
{
    /// Process single JSON byte and update parsing state
    #[inline]
    pub fn process_json_byte(
        &mut self,
        byte: u8,
    ) -> crate::jsonpath::error::JsonPathResult<super::super::processor::JsonProcessResult> {
        match &self.state {
            DeserializerState::Initial => self.process_initial_byte(byte),
            DeserializerState::Navigating => self.process_navigating_byte(byte),
            DeserializerState::ProcessingArray => self.process_array_byte(byte),
            DeserializerState::ProcessingObject => self.process_object_byte(byte),
            DeserializerState::Complete => Ok(super::super::processor::JsonProcessResult::Complete),
        }
    }

    /// Process byte when parser is in initial state
    #[inline]
    pub(super) fn process_initial_byte(
        &mut self,
        byte: u8,
    ) -> crate::jsonpath::error::JsonPathResult<super::super::processor::JsonProcessResult> {
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => {
                Ok(super::super::processor::JsonProcessResult::Continue)
            } // Skip whitespace
            b'{' => {
                let expression = self.path_expression.as_string();
                if expression.starts_with("$.") && expression.ends_with("[*]") {
                    // For $.property[*] patterns, we need to navigate to the property first
                    self.transition_to_navigating();
                    self.object_nesting = self.object_nesting.saturating_add(1);
                    // Reprocess this byte in the new navigating state
                    self.process_navigating_byte(byte)
                } else {
                    self.transition_to_processing_object();
                    self.object_nesting = self.object_nesting.saturating_add(1);
                    Ok(super::super::processor::JsonProcessResult::Continue)
                }
            }
            b'[' => {
                self.transition_to_processing_array();
                self.current_depth = self.current_depth.saturating_add(1);
                self.array_index_stack.push(self.current_array_index);
                self.current_array_index = 0;

                // For $[*] expressions, we immediately enter target array at root level
                let expression = self.path_expression.as_string();
                if expression == "$[*]" {
                    self.in_target_array = true;
                }

                Ok(super::super::processor::JsonProcessResult::Continue)
            }
            _ => Ok(super::super::processor::JsonProcessResult::Continue),
        }
    }

    /// Process byte when navigating through JSON structure
    pub(super) fn process_navigating_byte(
        &mut self,
        byte: u8,
    ) -> crate::jsonpath::error::JsonPathResult<super::super::processor::JsonProcessResult> {
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => {
                Ok(super::super::processor::JsonProcessResult::Continue)
            }
            b'"' => {
                // Potential property name - need to check if it matches our target
                if self.target_property.is_some() {
                    let property_name = self.read_property_name()?;
                    if let Some(ref target_prop) = self.target_property
                        && property_name == *target_prop {
                            self.in_target_property = true;
                        }
                } else {
                    // Skip string content normally
                    let mut escaped = false;
                    while let Some(string_byte) = self.read_next_byte()? {
                        if escaped {
                            escaped = false;
                        } else {
                            match string_byte {
                                b'"' => break,
                                b'\\' => escaped = true,
                                _ => {}
                            }
                        }
                    }
                }
                Ok(super::super::processor::JsonProcessResult::Continue)
            }
            b'{' => {
                if self.in_target_array && self.matches_current_path() {
                    // We're inside the target array - start collecting this object
                    self.object_buffer.clear();
                    self.object_buffer.push(byte);
                    self.transition_to_processing_object();
                    self.object_nesting = 1;
                    Ok(super::super::processor::JsonProcessResult::Continue)
                } else {
                    // We're still navigating - this is just a nested object, continue navigating
                    self.object_nesting = self.object_nesting.saturating_add(1);
                    Ok(super::super::processor::JsonProcessResult::Continue)
                }
            }
            b'[' => {
                if self.in_target_property {
                    // Found array in target property - this is our target array!
                    self.in_target_array = true;
                    self.in_target_property = false;
                }
                self.current_depth = self.current_depth.saturating_add(1);
                self.transition_to_processing_array();
                self.array_index_stack.push(self.current_array_index);
                self.current_array_index = 0;
                Ok(super::super::processor::JsonProcessResult::Continue)
            }
            b']' => {
                if self.in_target_array {
                    self.in_target_array = false;
                }
                self.current_depth = self.current_depth.saturating_sub(1);
                if let Some(prev_index) = self.array_index_stack.pop() {
                    self.current_array_index = prev_index;
                }
                if self.current_depth == 0 {
                    self.transition_to_complete();
                    Ok(super::super::processor::JsonProcessResult::Complete)
                } else {
                    Ok(super::super::processor::JsonProcessResult::Continue)
                }
            }
            b'}' => {
                self.object_nesting = self.object_nesting.saturating_sub(1);
                if self.object_nesting == 0 {
                    self.transition_to_complete();
                    Ok(super::super::processor::JsonProcessResult::Complete)
                } else {
                    Ok(super::super::processor::JsonProcessResult::Continue)
                }
            }
            b':' => {
                // Property separator - next value could be our target array
                Ok(super::super::processor::JsonProcessResult::Continue)
            }
            _ => Ok(super::super::processor::JsonProcessResult::Continue),
        }
    }

    /// Process byte when inside target array
    pub(super) fn process_array_byte(
        &mut self,
        byte: u8,
    ) -> crate::jsonpath::error::JsonPathResult<super::super::processor::JsonProcessResult> {
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => {
                Ok(super::super::processor::JsonProcessResult::Continue)
            }
            b'{' => {
                if self.in_target_array && self.matches_current_path() {
                    self.object_buffer.clear();
                    self.object_buffer.push(byte);
                    self.transition_to_processing_object();
                    self.object_nesting = 1;
                    Ok(super::super::processor::JsonProcessResult::Continue)
                } else {
                    Ok(super::super::processor::JsonProcessResult::Continue)
                }
            }
            b'[' => {
                self.current_depth = self.current_depth.saturating_add(1);
                self.transition_to_processing_array();
                self.array_index_stack.push(self.current_array_index);
                self.current_array_index = 0;
                Ok(super::super::processor::JsonProcessResult::Continue)
            }
            b']' => {
                // Check if we have a remaining object to process before closing array
                if self.in_target_array && !self.object_buffer.is_empty() {
                    // Last object in array - process it before closing
                    let result = super::super::processor::JsonProcessResult::ObjectFound;
                    self.in_target_array = false;
                    self.current_depth = self.current_depth.saturating_sub(1);
                    if let Some(prev_index) = self.array_index_stack.pop() {
                        self.current_array_index = prev_index;
                    }
                    return Ok(result);
                }

                if self.in_target_array {
                    self.in_target_array = false;
                }
                self.current_depth = self.current_depth.saturating_sub(1);
                if let Some(prev_index) = self.array_index_stack.pop() {
                    self.current_array_index = prev_index;
                }
                if self.current_depth == 0 {
                    self.transition_to_complete();
                    Ok(super::super::processor::JsonProcessResult::Complete)
                } else {
                    Ok(super::super::processor::JsonProcessResult::Continue)
                }
            }
            b',' => {
                if self.in_target_array && !self.object_buffer.is_empty() {
                    // Don't add the comma to the object buffer - it's a separator, not part of the object
                    Ok(super::super::processor::JsonProcessResult::ObjectFound)
                } else {
                    self.current_array_index = self.current_array_index.saturating_add(1);
                    Ok(super::super::processor::JsonProcessResult::Continue)
                }
            }
            _ => {
                if self.in_target_array {
                    self.object_buffer.push(byte);
                }
                Ok(super::super::processor::JsonProcessResult::Continue)
            }
        }
    }

    /// Process byte when inside JSON object
    pub(super) fn process_object_byte(
        &mut self,
        byte: u8,
    ) -> crate::jsonpath::error::JsonPathResult<super::super::processor::JsonProcessResult> {
        // Always add bytes to object buffer if we're in target array BEFORE processing special characters
        if self.in_target_array {
            self.object_buffer.push(byte);
        }

        match byte {
            b'"' => {
                // Skip string content but don't add the bytes again since we already added the opening quote
                let mut escaped = false;
                while let Some(string_byte) = self.read_next_byte()? {
                    if self.in_target_array {
                        self.object_buffer.push(string_byte);
                    }
                    if escaped {
                        escaped = false;
                    } else {
                        match string_byte {
                            b'"' => break, // End of string
                            b'\\' => escaped = true,
                            _ => {}
                        }
                    }
                }
                Ok(super::super::processor::JsonProcessResult::Continue)
            }
            b'{' => {
                self.object_nesting = self.object_nesting.saturating_add(1);
                Ok(super::super::processor::JsonProcessResult::Continue)
            }
            b'[' => {
                // Transition to streaming array state when encountering array in object
                self.current_depth = self.current_depth.saturating_add(1);
                self.transition_to_processing_array();
                self.array_index_stack.push(self.current_array_index);
                self.current_array_index = 0;
                Ok(super::super::processor::JsonProcessResult::Continue)
            }
            b'}' => {
                self.object_nesting = self.object_nesting.saturating_sub(1);
                if self.object_nesting == 0 {
                    // Complete object found - transition back to streaming array state
                    self.transition_to_processing_array();
                    Ok(super::super::processor::JsonProcessResult::ObjectFound)
                } else {
                    Ok(super::super::processor::JsonProcessResult::Continue)
                }
            }
            _ => Ok(super::super::processor::JsonProcessResult::Continue),
        }
    }
}
