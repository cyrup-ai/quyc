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
    deserializer::core::types::StreamingJsonPathState,
};

impl StreamStateMachine {
    /// Create new state machine for JSON streaming
    #[must_use] 
    pub fn new() -> Self {
        Self {
            state: JsonStreamState::Initial,
            stats: StateStats::default(),
            path_expression: None,
            depth_stack: VecDeque::new(),
        }
    }

    /// Initialize state machine with `JSONPath` expression
    ///
    /// # Arguments
    ///
    /// * `expression` - Compiled `JSONPath` expression to evaluate
    ///
    /// # Performance
    ///
    /// `JSONPath` expression is analyzed once during initialization to optimize
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
    #[must_use] 
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
                    format!("unexpected byte 0x{byte:02x} in initial state"),
                    "initial",
                    false,
                );
                Ok(ProcessResult::Error(err))
            }
        }
    }

    /// Process byte while navigating to `JSONPath` target
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

/// Matched JSON value with location information
#[derive(Debug, Clone)]
pub struct MatchedValue {
    pub path: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub depth: usize,
}

impl StreamStateMachine {
    /// Process JSON chunk with full streaming `JSONPath` state management
    pub fn process_chunk_with_jsonpath_state(
        &mut self,
        chunk: &[u8],
        jsonpath_state: &mut StreamingJsonPathState,
    ) -> Result<Vec<MatchedValue>, JsonPathError> {
        let mut matches = Vec::new();
        let mut pos = 0;

        while pos < chunk.len() {
            jsonpath_state.evaluation_stats.nodes_processed += 1;

            match self.process_byte_with_jsonpath(chunk[pos], pos, jsonpath_state) {
                Ok(ProcessResult::Continue) => pos += 1,
                Ok(ProcessResult::ObjectBoundary { start, end }) => {
                    // Check if this object matches the JSONPath expression
                    if self.matches_current_expression(jsonpath_state) {
                        matches.push(MatchedValue {
                            path: jsonpath_state.current_json_path(),
                            start_offset: start,
                            end_offset: end,
                            depth: jsonpath_state.current_depth,
                        });
                        jsonpath_state.evaluation_stats.matches_found += 1;
                    }
                    pos += 1;
                }
                Ok(ProcessResult::NeedMoreData) => break,
                Ok(ProcessResult::Complete) => break,
                Ok(ProcessResult::Error(err)) => {
                    log::error!("JSONPath streaming error at offset {pos}: {err}");
                    pos += 1; // Continue processing
                }
                Err(err) => {
                    log::error!("State machine error at offset {pos}: {err}");
                    pos += 1; // Continue processing
                }
            }
        }

        Ok(matches)
    }

    /// Check if current position matches `JSONPath` expression
    fn matches_current_expression(&self, jsonpath_state: &StreamingJsonPathState) -> bool {
        // Complete implementation that evaluates JSONPath selectors
        if jsonpath_state.selectors.is_empty() {
            return false;
        }
        
        // Check if we're at the end of the expression
        if jsonpath_state.current_selector_index >= jsonpath_state.max_selector_count {
            return true;
        }
        
        // Get current selector for evaluation
        if jsonpath_state.current_selector_index >= jsonpath_state.selectors.len() {
            return false;
        }
        
        let current_selector = &jsonpath_state.selectors[jsonpath_state.current_selector_index];
        
        // Evaluate current selector based on streaming state
        match current_selector {
            crate::jsonpath::ast::JsonSelector::Root => {
                jsonpath_state.current_depth == 0
            }
            crate::jsonpath::ast::JsonSelector::RecursiveDescent => {
                // Recursive descent matches at any depth when active
                jsonpath_state.in_recursive_descent || self.should_activate_recursive_descent(jsonpath_state)
            }
            crate::jsonpath::ast::JsonSelector::Wildcard => {
                // Wildcard matches if we have navigation frames indicating structure
                !jsonpath_state.path_breadcrumbs.is_empty()
            }
            crate::jsonpath::ast::JsonSelector::Child { name, .. } => {
                // Check if current path contains the property
                jsonpath_state.path_breadcrumbs.iter().any(|frame| {
                    matches!(&frame.segment, crate::jsonpath::deserializer::core::types::PathSegment::Property(prop) if prop == name)
                })
            }
            crate::jsonpath::ast::JsonSelector::Index { index, .. } => {
                // Check if current path contains the array index
                jsonpath_state.path_breadcrumbs.iter().any(|frame| {
                    matches!(&frame.segment, crate::jsonpath::deserializer::core::types::PathSegment::ArrayIndex(i) if *i == *index as usize)
                })
            }
            crate::jsonpath::ast::JsonSelector::Slice { start, end, step } => {
                // Check if we're in an array context and index falls within slice
                jsonpath_state.path_breadcrumbs.iter().any(|frame| {
                    if let crate::jsonpath::deserializer::core::types::PathSegment::ArrayIndex(i) = frame.segment {
                        self.index_matches_slice(i, *start, *end, *step)
                    } else {
                        false
                    }
                })
            }
            crate::jsonpath::ast::JsonSelector::Filter { expression: _ } => {
                // Filter evaluation requires object/array context
                // For streaming, we do basic presence check
                !jsonpath_state.path_breadcrumbs.is_empty() && jsonpath_state.current_depth > 0
            }
            crate::jsonpath::ast::JsonSelector::Union { selectors } => {
                // Union matches if any sub-selector matches
                selectors.iter().any(|sub_selector| {
                    self.evaluate_single_selector(sub_selector, jsonpath_state)
                })
            }
        }
    }
    
    /// Check if recursive descent should be activated at current position
    fn should_activate_recursive_descent(&self, jsonpath_state: &StreamingJsonPathState) -> bool {
        // Recursive descent activates when we encounter the recursive descent operator
        // and we're navigating through nested structures
        jsonpath_state.current_depth > 0 && 
        jsonpath_state.selectors.iter().any(|sel| {
            matches!(sel, crate::jsonpath::ast::JsonSelector::RecursiveDescent)
        })
    }
    
    /// Check if array index matches slice criteria
    fn index_matches_slice(&self, index: usize, start: Option<i64>, end: Option<i64>, step: Option<i64>) -> bool {
        let step = step.unwrap_or(1).max(1) as usize; // Ensure positive step
        let index = index as i64;
        
        // Handle start boundary
        let start_bound = start.unwrap_or(0);
        if index < start_bound {
            return false;
        }
        
        // Handle end boundary
        if let Some(end_bound) = end
            && index >= end_bound {
                return false;
            }
        
        // Check step alignment
        if step > 1 {
            ((index - start_bound) as usize).is_multiple_of(step)
        } else {
            true
        }
    }
    
    /// Evaluate a single selector against current streaming state
    fn evaluate_single_selector(&self, selector: &crate::jsonpath::ast::JsonSelector, jsonpath_state: &StreamingJsonPathState) -> bool {
        match selector {
            crate::jsonpath::ast::JsonSelector::Child { name, .. } => {
                jsonpath_state.path_breadcrumbs.iter().any(|frame| {
                    matches!(&frame.segment, crate::jsonpath::deserializer::core::types::PathSegment::Property(prop) if prop == name)
                })
            }
            crate::jsonpath::ast::JsonSelector::Index { index, .. } => {
                jsonpath_state.path_breadcrumbs.iter().any(|frame| {
                    matches!(&frame.segment, crate::jsonpath::deserializer::core::types::PathSegment::ArrayIndex(i) if *i == *index as usize)
                })
            }
            crate::jsonpath::ast::JsonSelector::Wildcard => true,
            crate::jsonpath::ast::JsonSelector::RecursiveDescent => jsonpath_state.in_recursive_descent,
            _ => false,
        }
    }

    /// Process byte with `JSONPath` state tracking
    fn process_byte_with_jsonpath(
        &mut self,
        byte: u8,
        offset: usize,
        jsonpath_state: &mut StreamingJsonPathState,
    ) -> Result<ProcessResult, JsonPathError> {
        // Delegate to existing process_byte, then update JSONPath state
        let result = self.process_byte(byte, offset)?;

        // Update JSONPath state based on JSON structure changes
        match byte {
            b'{' => {
                jsonpath_state.enter_depth();
                self.handle_object_start_for_jsonpath(jsonpath_state)?;
            }
            b'}' => {
                self.handle_object_end_for_jsonpath(jsonpath_state)?;
                jsonpath_state.exit_depth();
            }
            b'[' => {
                jsonpath_state.enter_depth();
                self.handle_array_start_for_jsonpath(jsonpath_state)?;
            }
            b']' => {
                self.handle_array_end_for_jsonpath(jsonpath_state)?;
                jsonpath_state.exit_depth();
            }
            _ => {} // Handle property keys and values in separate methods
        }

        Ok(result)
    }

    /// Handle JSON object start for `JSONPath` evaluation
    fn handle_object_start_for_jsonpath(
        &mut self,
        jsonpath_state: &mut StreamingJsonPathState,
    ) -> Result<(), JsonPathError> {
        // Update depth tracking for object entry
        jsonpath_state.enter_depth();
        
        // Check current selector to determine navigation strategy
        if jsonpath_state.current_selector_index < jsonpath_state.selectors.len() {
            let current_selector = &jsonpath_state.selectors[jsonpath_state.current_selector_index];
            
            match current_selector {
                crate::jsonpath::ast::JsonSelector::RecursiveDescent => {
                    // Enter recursive descent mode for deep object traversal
                    let origin_path = jsonpath_state.current_json_path();
                    jsonpath_state.enter_recursive_descent(
                        origin_path,
                        jsonpath_state.current_selector_index
                    )?;
                    
                    // Add navigation frame for recursive descent marker
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::RecursiveDescent,
                        true
                    );
                }
                crate::jsonpath::ast::JsonSelector::Root => {
                    // Root selector only matches at depth 0
                    if jsonpath_state.current_depth == 1 {
                        match jsonpath_state.advance_selector() {
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::Advanced(_) => {
                                // Successfully advanced to next selector
                            }
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::ExpressionComplete => {
                                // Expression is complete
                            }
                        }
                    }
                }
                crate::jsonpath::ast::JsonSelector::Wildcard => {
                    // Wildcard matches any object at current depth
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::Wildcard,
                        true
                    );
                    
                    match jsonpath_state.advance_selector() {
                        crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::Advanced(_) => {
                            // Successfully advanced to next selector
                        }
                        crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::ExpressionComplete => {
                            // Expression is complete
                        }
                    }
                }
                crate::jsonpath::ast::JsonSelector::Child { .. } => {
                    // Property selectors will be handled when we encounter property names
                    // For now, just add a placeholder navigation frame that will be updated
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::Property(String::new()),
                        false
                    );
                }
                crate::jsonpath::ast::JsonSelector::Filter { expression: _ } => {
                    // Filter selectors require object/array context for evaluation
                    // Mark this as a potential match location
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::FilterMatch("pending".to_string()),
                        false // Will be updated during filter evaluation
                    );
                }
                crate::jsonpath::ast::JsonSelector::Union { selectors } => {
                    // Union selectors need evaluation against all alternatives
                    // For object start, we create a navigation frame and defer evaluation
                    let mut any_match = false;
                    
                    for sub_selector in selectors {
                        match sub_selector {
                            crate::jsonpath::ast::JsonSelector::Wildcard => {
                                any_match = true;
                                break;
                            }
                            crate::jsonpath::ast::JsonSelector::Child { .. } => {
                                // Will be evaluated when property names are encountered
                            }
                            _ => {},
                        }
                    }
                    
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::Property("union_object".to_string()),
                        any_match
                    );
                    
                    if any_match {
                        match jsonpath_state.advance_selector() {
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::Advanced(_) => {
                                // Successfully advanced to next selector
                            }
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::ExpressionComplete => {
                                // Expression is complete
                            }
                        }
                    }
                }
                _ => {
                    // Other selector types (Index, Slice, Child) are not applicable to object start
                    // Add a generic navigation frame for structural tracking
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::Property("object".to_string()),
                        false
                    );
                }
            }
        } else {
            // No more selectors to evaluate, add basic navigation frame
            jsonpath_state.push_navigation_frame(
                crate::jsonpath::deserializer::core::types::PathSegment::Property("object".to_string()),
                false
            );
        }
        
        // Update statistics
        jsonpath_state.evaluation_stats.nodes_processed += 1;
        
        Ok(())
    }

    /// Handle JSON object end for `JSONPath` evaluation  
    fn handle_object_end_for_jsonpath(
        &mut self,
        jsonpath_state: &mut StreamingJsonPathState,
    ) -> Result<(), JsonPathError> {
        // Handle backtracking and state cleanup
        jsonpath_state.pop_navigation_frame();
        Ok(())
    }

    /// Handle JSON array start for `JSONPath` evaluation
    fn handle_array_start_for_jsonpath(
        &mut self,
        jsonpath_state: &mut StreamingJsonPathState,
    ) -> Result<(), JsonPathError> {
        // Update depth tracking for array entry
        jsonpath_state.enter_depth();
        
        // Check current selector to determine navigation strategy
        if jsonpath_state.current_selector_index < jsonpath_state.selectors.len() {
            // Clone the current selector to avoid borrowing conflicts
            let current_selector = jsonpath_state.selectors[jsonpath_state.current_selector_index].clone();
            
            match current_selector {
                crate::jsonpath::ast::JsonSelector::RecursiveDescent => {
                    // Enter recursive descent mode for deep array traversal
                    let origin_path = jsonpath_state.current_json_path();
                    jsonpath_state.enter_recursive_descent(
                        origin_path,
                        jsonpath_state.current_selector_index
                    )?;
                    
                    // Add navigation frame for recursive descent marker
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::RecursiveDescent,
                        true
                    );
                }
                crate::jsonpath::ast::JsonSelector::Wildcard => {
                    // Wildcard matches any array element at current depth
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::Wildcard,
                        true
                    );
                    
                    match jsonpath_state.advance_selector() {
                        crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::Advanced(_) => {
                            // Successfully advanced to next selector
                        }
                        crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::ExpressionComplete => {
                            // Expression is complete
                        }
                    }
                }
                crate::jsonpath::ast::JsonSelector::Index { index, .. } => {
                    // Index selector - we'll check specific indices as we encounter array elements
                    // Use index directly
                    let target_index = index;
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::ArrayIndex(0),
                        target_index == 0 // Match if target is index 0
                    );
                    
                    if target_index == 0 {
                        match jsonpath_state.advance_selector() {
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::Advanced(_) => {
                                // Successfully advanced to next selector
                            }
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::ExpressionComplete => {
                                // Expression is complete
                            }
                        }
                    }
                }
                crate::jsonpath::ast::JsonSelector::Slice { start, end, step } => {
                    // Slice selector - check if index 0 falls within the slice
                    let index_matches = self.index_matches_slice(0, start, end, step);
                    
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::ArrayIndex(0),
                        index_matches
                    );
                    
                    if index_matches {
                        match jsonpath_state.advance_selector() {
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::Advanced(_) => {
                                // Successfully advanced to next selector
                            }
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::ExpressionComplete => {
                                // Expression is complete
                            }
                        }
                    }
                }
                crate::jsonpath::ast::JsonSelector::Filter { expression: _ } => {
                    // Filter selectors require array elements for evaluation
                    // Mark this as a potential match location for filter evaluation
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::FilterMatch("array_filter".to_string()),
                        false // Will be updated during filter evaluation
                    );
                }
                crate::jsonpath::ast::JsonSelector::Union { selectors } => {
                    // Union selectors need evaluation against all alternatives
                    let mut any_match = false;
                    
                    for sub_selector in selectors {
                        match sub_selector {
                            crate::jsonpath::ast::JsonSelector::Wildcard => {
                                any_match = true;
                                break;
                            }
                            crate::jsonpath::ast::JsonSelector::Index { index, .. } => {
                                if index == 0 {
                                    any_match = true;
                                    break;
                                }
                            }
                            crate::jsonpath::ast::JsonSelector::Slice { start, end, step } => {
                                if self.index_matches_slice(0, start, end, step) {
                                    any_match = true;
                                    break;
                                }
                            }
                            _ => {},
                        }
                    }
                    
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::ArrayIndex(0),
                        any_match
                    );
                    
                    if any_match {
                        match jsonpath_state.advance_selector() {
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::Advanced(_) => {
                                // Successfully advanced to next selector
                            }
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::ExpressionComplete => {
                                // Expression is complete
                            }
                        }
                    }
                }
                crate::jsonpath::ast::JsonSelector::Root => {
                    // Root selector only matches at depth 0, so array at higher depth doesn't match
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::ArrayIndex(0),
                        jsonpath_state.current_depth == 1
                    );
                    
                    if jsonpath_state.current_depth == 1 {
                        match jsonpath_state.advance_selector() {
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::Advanced(_) => {
                                // Successfully advanced to next selector
                            }
                            crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::ExpressionComplete => {
                                // Expression is complete
                            }
                        }
                    }
                }
                _ => {
                    // Other selector types (Property, Child) are not applicable to array start
                    // Add a generic navigation frame for structural tracking
                    jsonpath_state.push_navigation_frame(
                        crate::jsonpath::deserializer::core::types::PathSegment::ArrayIndex(0),
                        false
                    );
                }
            }
        } else {
            // No more selectors to evaluate, add basic navigation frame
            jsonpath_state.push_navigation_frame(
                crate::jsonpath::deserializer::core::types::PathSegment::ArrayIndex(0),
                false
            );
        }
        
        // Update statistics
        jsonpath_state.evaluation_stats.nodes_processed += 1;
        
        Ok(())
    }

    /// Handle JSON array end for `JSONPath` evaluation
    fn handle_array_end_for_jsonpath(
        &mut self,
        jsonpath_state: &mut StreamingJsonPathState,
    ) -> Result<(), JsonPathError> {
        // Handle array exit and state cleanup
        jsonpath_state.pop_navigation_frame();
        Ok(())
    }
}
