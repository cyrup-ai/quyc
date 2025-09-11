use serde::de::DeserializeOwned;

use super::types::{DeserializerState, JsonPathDeserializer, StreamingJsonPathState};
use crate::jsonpath::{buffer::StreamBuffer, parser::JsonPathExpression};

impl<'a, T> JsonPathDeserializer<'a, T>
where
    T: DeserializeOwned,
{
    /// Create new streaming deserializer instance
    ///
    /// # Arguments
    ///
    /// * `path_expression` - Compiled `JSONPath` expression for navigation
    /// * `buffer` - Streaming buffer containing JSON bytes
    #[inline]
    pub fn new(path_expression: &'a JsonPathExpression, buffer: &'a mut StreamBuffer) -> Self {
        let has_recursive_descent = path_expression.has_recursive_descent();
        let initial_capacity = if has_recursive_descent { 256 } else { 32 };

        // Extract target property name for $.property[*] patterns
        let target_property = {
            let expr = path_expression.as_string();
            if expr.starts_with("$.") && expr.ends_with("[*]") {
                let property_part = &expr[2..expr.len() - 3]; // Remove "$." and "[*]"
                if !property_part.contains('.') && !property_part.contains('[') {
                    Some(property_part.to_string())
                } else {
                    None // Complex nested paths not supported yet
                }
            } else {
                None
            }
        };

        Self {
            path_expression,
            buffer,
            state: DeserializerState::Initial,
            current_depth: 0,
            in_target_array: false,
            object_nesting: 0,
            object_buffer: Vec::with_capacity(initial_capacity * 4), // Adaptive capacity based on complexity
            // Initialize streaming state with compiled expression
            streaming_state: StreamingJsonPathState::new(path_expression),
            current_array_index: 0,
            array_index_stack: Vec::with_capacity(16), // Support up to 16 nested arrays
            buffer_position: 0,
            target_property,
            in_target_property: false,
            _phantom: std::marker::PhantomData,
        }
    }
}
