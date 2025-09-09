//! State machine core processing engine
//!
//! This module contains the main processing logic for the JSON streaming
//! state machine, including byte processing and state management.

use std::collections::VecDeque;

use super::types::{
    FrameIdentifier, JsonStreamState, ObjectBoundary, ProcessResult, StateStats, StreamStateMachine,
};
use crate::jsonpath::{
    error::{JsonPathError, stream_error},
    parser::JsonPathExpression,
};

impl StreamStateMachine {
    /// Create new state machine for JSON streaming
    pub fn new() -> Self {
        Self {
            state: JsonStreamState::Initial,
            stats: StateStats::default(),
            path_expression: None,
            depth_stack: VecDeque::new(),
        }
    }

    /// Initialize state machine with JSONPath expression
    ///
    /// # Arguments
    ///
    /// * `expression` - Compiled JSONPath expression to evaluate
    ///
    /// # Performance
    ///
    /// JSONPath expression is analyzed once during initialization to optimize
    /// runtime state transitions and minimize allocation during streaming.
    pub fn initialize(&mut self, expression: JsonPathExpression) {
        self.path_expression = Some(expression);
        self.state = JsonStreamState::Navigating {
            depth: 0,
            remaining_selectors: self
                .path_expression
                .as_ref()
                .map(|e| e.selectors().to_vec())
                .unwrap_or_default(),
            current_value: None,
        };
        self.stats.state_transitions += 1;
    }

    /// Get current state (for testing and debugging)
    #[inline]
    pub fn state(&self) -> &JsonStreamState {
        &self.state
    }

    /// Process incoming JSON bytes and update state
    ///
    /// # Arguments
    ///
    /// * `data` - JSON bytes to process
    /// * `offset` - Byte offset in overall stream
    ///
    /// # Returns
    ///
    /// Vector of byte ranges where complete JSON objects were found.
    ///
    /// # Performance
    ///
    /// Uses single-pass parsing with minimal allocations. State transitions
    /// are inlined for maximum performance in hot paths.
    pub fn process_bytes(&mut self, data: &[u8], offset: usize) -> Vec<ObjectBoundary> {
        let mut boundaries = Vec::new();
        let mut pos = 0;

        while pos < data.len() {
            match self.process_byte(data[pos], offset + pos) {
                Ok(ProcessResult::Continue) => pos += 1,
                Ok(ProcessResult::ObjectBoundary { start, end }) => {
                    boundaries.push(ObjectBoundary { start, end });
                    self.stats.objects_yielded += 1;
                    pos += 1;
                }
                Ok(ProcessResult::NeedMoreData) => break,
                Ok(ProcessResult::Complete) => {
                    super::transitions::transition_to_complete(self);
                    break;
                }
                Ok(ProcessResult::Error(err)) => {
                    super::transitions::transition_to_error(self, err.clone(), true);
                    log::error!("JSON parsing error at offset {}: {}", offset + pos, err);
                    // Continue processing to handle partial data gracefully
                    pos += 1;
                }
                Err(err) => {
                    super::transitions::transition_to_error(self, err.clone(), true);
                    log::error!("State machine error at offset {}: {}", offset + pos, err);
                    // Continue processing to handle partial data gracefully
                    pos += 1;
                }
            }
        }

        boundaries
    }

    /// Process single byte and update state machine
    ///
    /// # Performance
    ///
    /// This is the hot path - optimized for maximum performance with inlined
    /// state transitions and minimal branching.
    #[inline]
    fn process_byte(
        &mut self,
        byte: u8,
        absolute_offset: usize,
    ) -> Result<ProcessResult, JsonPathError> {
        match &mut self.state {
            JsonStreamState::Initial => self.process_initial_byte(byte),
            JsonStreamState::Navigating { .. } => {
                self.process_navigating_byte(byte, absolute_offset)
            }
            JsonStreamState::StreamingArray { .. } => {
                self.process_streaming_byte(byte, absolute_offset)
            }
            JsonStreamState::ProcessingObject { .. } => {
                self.process_object_byte(byte, absolute_offset)
            }
            JsonStreamState::Finishing { .. } => self.process_finishing_byte(byte),
            JsonStreamState::Complete => Ok(ProcessResult::Complete),
            JsonStreamState::Error { .. } => {
                if let Some(error) = super::utils::current_error(self) {
                    Err(error)
                } else {
                    Err(stream_error(
                        "State machine in error state without error details",
                        "process_byte",
                        false,
                    ))
                }
            }
        }
    }

    /// Process byte in initial state
    #[inline]
    fn process_initial_byte(&mut self, byte: u8) -> Result<ProcessResult, JsonPathError> {
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => Ok(ProcessResult::Continue), // Skip whitespace
            b'{' => {
                super::transitions::transition_to_navigating(self);
                self.enter_object();
                Ok(ProcessResult::Continue)
            }
            b'[' => {
                super::transitions::transition_to_navigating(self);
                self.enter_array();
                Ok(ProcessResult::Continue)
            }
            _ => {
                let err = stream_error(
                    &format!("unexpected byte 0x{:02x} in initial state", byte),
                    "initial",
                    false,
                );
                Ok(ProcessResult::Error(err))
            }
        }
    }

    /// Process byte while navigating to JSONPath target
    fn process_navigating_byte(
        &mut self,
        byte: u8,
        _absolute_offset: usize,
    ) -> Result<ProcessResult, JsonPathError> {
        super::processors::process_navigating_byte(self, byte)
    }

    /// Process byte while streaming array elements
    fn process_streaming_byte(
        &mut self,
        byte: u8,
        absolute_offset: usize,
    ) -> Result<ProcessResult, JsonPathError> {
        super::processors::process_streaming_byte(self, byte, absolute_offset)
    }

    /// Process byte while processing JSON object
    fn process_object_byte(
        &mut self,
        byte: u8,
        absolute_offset: usize,
    ) -> Result<ProcessResult, JsonPathError> {
        super::processors::process_object_byte(self, byte, absolute_offset)
    }

    /// Process byte in finishing state
    fn process_finishing_byte(&mut self, byte: u8) -> Result<ProcessResult, JsonPathError> {
        super::processors::process_finishing_byte(self, byte)
    }

    /// Enter an object context, incrementing the current depth
    pub fn enter_object(&mut self) {
        self.stats.current_depth += 1;
        self.stats.max_depth = self.stats.max_depth.max(self.stats.current_depth);
        self.depth_stack.push_back(FrameIdentifier::Root);
    }

    /// Exit an object context, decrementing the current depth
    pub fn exit_object(&mut self) {
        self.stats.current_depth = self.stats.current_depth.saturating_sub(1);
        self.depth_stack.pop_back();
    }

    /// Enter an array context, incrementing the current depth
    pub fn enter_array(&mut self) {
        self.stats.current_depth += 1;
        self.stats.max_depth = self.stats.max_depth.max(self.stats.current_depth);
        self.depth_stack.push_back(FrameIdentifier::Index(0));
    }

    /// Exit an array context, decrementing the current depth
    pub fn exit_array(&mut self) {
        self.stats.current_depth = self.stats.current_depth.saturating_sub(1);
        self.depth_stack.pop_back();
    }
}

impl Default for StreamStateMachine {
    fn default() -> Self {
        Self::new()
    }
}
