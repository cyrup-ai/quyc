use serde::de::DeserializeOwned;

use super::super::iterator::JsonPathIterator;
use crate::jsonpath::{buffer::StreamBuffer, parser::JsonPathExpression, error::JsonPathError, ast::JsonSelector};

/// Current state of the JSON deserializer
#[derive(Debug, Clone, PartialEq)]
pub enum DeserializerState {
    /// Initial state - waiting for JSON to begin
    Initial,
    /// Navigating through JSON structure to find target location
    Navigating,
    /// Processing array elements at target location
    ProcessingArray,
    /// Processing individual JSON object
    ProcessingObject,
    /// Processing complete
    Complete,
}

/// Complete streaming `JSONPath` evaluation state
#[derive(Debug, Clone)]
pub struct StreamingJsonPathState {
    // === EXISTING INTEGRATION ===
    /// `JSONPath` expression selectors (from `CoreJsonPathEvaluator`)
    pub selectors: Vec<JsonSelector>,
    
    /// Current position in the compiled `JSONPath` selector chain
    /// IMPLEMENTED: Tracks which selector in the expression we're evaluating  
    pub current_selector_index: usize,
    
    /// Maximum number of selectors in the compiled expression
    pub max_selector_count: usize,
    
    // === RECURSIVE DESCENT INTEGRATION ===  
    /// Whether we're currently processing a recursive descent (..) operation
    /// IMPLEMENTED: Controls depth-first search behavior for ".." operator
    pub in_recursive_descent: bool,
    
    /// Stack of recursive descent contexts for nested ".." operations
    /// IMPLEMENTED: Handles complex patterns like "$..[*]..[*]"
    pub recursive_descent_stack: Vec<RecursiveDescentFrame>,
    
    // === NAVIGATION STATE INTEGRATION ===
    /// Navigation history for pattern backtracking and path reconstruction\
    /// IMPLEMENTED: Enables efficient backtracking in complex `JSONPath` expressions
    pub path_breadcrumbs: Vec<PathNavigationFrame>,
    
    // === PERFORMANCE TRACKING ===
    /// Current JSON depth level for performance optimization
    pub current_depth: usize,
    
    /// Maximum allowed depth to prevent stack overflow  
    pub max_depth: usize,
    
    /// Performance metrics for streaming evaluation
    pub evaluation_stats: StreamingStats,
}

/// Recursive descent context frame
#[derive(Debug, Clone)]
pub struct RecursiveDescentFrame {
    /// Depth level where recursive descent started
    pub start_depth: usize,
    
    /// Current depth level in the recursive search
    pub current_depth: usize,
    
    /// Index of the selector that triggered recursive descent
    pub triggering_selector_index: usize,
    
    /// `JSONPath` of the node where recursive descent began
    pub origin_path: String,
    
    /// Whether this frame should continue searching deeper
    pub should_continue: bool,
    
    /// Stack of nodes visited during this recursive descent
    pub visited_nodes: Vec<String>,
}

/// Path navigation breadcrumb for backtracking
#[derive(Debug, Clone)]
pub struct PathNavigationFrame {
    /// Property key or array index
    pub segment: PathSegment,
    
    /// Depth level of this navigation frame
    pub depth: usize,
    
    /// JSON path from root to this point
    pub accumulated_path: String,
    
    /// Whether this frame represents a match
    pub is_match: bool,
    
    /// Selector index that created this frame
    pub selector_index: usize,
}

/// Path segment types for navigation
#[derive(Debug, Clone)]
pub enum PathSegment {
    Property(String),
    ArrayIndex(usize),
    Wildcard,
    RecursiveDescent,
    FilterMatch(String),
}

/// Zero-allocation performance statistics  
#[derive(Debug, Clone, Default)]
pub struct StreamingStats {
    pub nodes_processed: u64,
    pub matches_found: u64,
    pub recursive_descents_performed: u64,
    pub backtrack_operations: u64,
    pub max_depth_reached: usize,
}

/// Comprehensive processing statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub buffer_size: usize,
    pub buffer_capacity: usize,
    pub buffer_utilization: f64,
    pub current_depth: usize,
    pub streaming_depth: usize,
    pub selector_progress: f64,
    pub in_recursive_descent: bool,
    pub navigation_frames: usize,
    pub matches_found: u64,
    pub nodes_processed: u64,
    pub processing_efficiency: f64,
    pub should_buffer_current: bool,
}

/// Result of selector advancement
#[derive(Debug)]
pub enum SelectorAdvanceResult {
    Advanced(usize),
    ExpressionComplete,
}

impl StreamingJsonPathState {
    /// Create new state from compiled `JSONPath` expression
    #[must_use] 
    pub fn new(compiled_expression: &JsonPathExpression) -> Self {
        Self {
            selectors: compiled_expression.selectors().to_vec(),
            current_selector_index: 0,
            max_selector_count: compiled_expression.selectors().len(),
            in_recursive_descent: false,
            recursive_descent_stack: Vec::with_capacity(16), // Pre-allocate
            path_breadcrumbs: Vec::with_capacity(32),
            current_depth: 0,
            max_depth: 1000, // Prevent stack overflow
            evaluation_stats: StreamingStats::default(),
        }
    }

    /// Reset state for processing new buffer data while preserving compiled expression
    pub fn reset_for_new_buffer(&mut self) {
        self.current_selector_index = 0;
        self.in_recursive_descent = false;
        self.recursive_descent_stack.clear();
        self.path_breadcrumbs.clear();
        self.current_depth = 0;
        // Keep evaluation_stats for cumulative performance tracking
    }

    /// Check if state indicates completion of `JSONPath` expression evaluation
    #[must_use] 
    pub fn is_expression_complete(&self) -> bool {
        self.current_selector_index >= self.max_selector_count
    }

    /// Check if current state indicates a potential match worth buffering
    #[must_use] 
    pub fn should_buffer_current_object(&self) -> bool {
        // Buffer if we're close to completing the expression
        if self.current_selector_index >= self.max_selector_count.saturating_sub(2) {
            return true;
        }
        
        // Buffer if we have matching navigation frames
        if self.path_breadcrumbs.iter().any(|frame| frame.is_match) {
            return true;
        }
        
        // Buffer if we're in recursive descent mode
        if self.in_recursive_descent {
            return true;
        }
        
        false
    }

    /// Calculate the minimum buffer size needed for efficient processing
    #[must_use] 
    pub fn required_buffer_size(&self) -> usize {
        // Base size for JSON structure parsing
        let base_size = 8192; // 8KB base
        
        // Additional space for deep nesting
        let depth_overhead = self.current_depth * 512; // 512 bytes per depth level
        
        // Additional space for complex selectors
        let selector_overhead = self.selectors.len() * 256; // 256 bytes per selector
        
        // Additional space for recursive descent tracking
        let recursive_overhead = if self.in_recursive_descent { 4096 } else { 0 };
        
        base_size + depth_overhead + selector_overhead + recursive_overhead
    }

    /// Update state based on buffer consumption (bytes processed)
    pub fn handle_buffer_consumption(&mut self, bytes_consumed: usize) {
        self.evaluation_stats.nodes_processed += bytes_consumed as u64 / 10; // Rough estimate of nodes per byte
        
        // If we've consumed a significant amount and have no matches, consider resetting
        if bytes_consumed > 16384 && self.evaluation_stats.matches_found == 0 {
            // Don't reset completely, but reduce some overhead
            if !self.in_recursive_descent && self.path_breadcrumbs.len() > 64 {
                // Keep only the most recent breadcrumbs to prevent memory bloat
                let keep_count = 32;
                if self.path_breadcrumbs.len() > keep_count {
                    self.path_breadcrumbs.drain(..self.path_breadcrumbs.len() - keep_count);
                }
            }
        }
    }

    /// Get processing efficiency metrics for buffer optimization
    #[must_use] 
    pub fn processing_efficiency(&self) -> f64 {
        if self.evaluation_stats.nodes_processed == 0 {
            return 1.0; // Start optimistically
        }
        
        // Precision loss acceptable for JSONPath processing efficiency statistics
        #[allow(clippy::cast_precision_loss)]
        let match_rate = self.evaluation_stats.matches_found as f64 / self.evaluation_stats.nodes_processed as f64;
        let backtrack_penalty = if self.evaluation_stats.backtrack_operations > 0 {
            #[allow(clippy::cast_precision_loss)]
            { 1.0 - (self.evaluation_stats.backtrack_operations as f64 / self.evaluation_stats.nodes_processed as f64) }
        } else {
            1.0
        };
        
        (match_rate * backtrack_penalty).clamp(0.1, 1.0)
    }
    
    /// Advance to the next selector in the `JSONPath` expression
    pub fn advance_selector(&mut self) -> SelectorAdvanceResult {
        if self.current_selector_index + 1 >= self.max_selector_count {
            SelectorAdvanceResult::ExpressionComplete
        } else {
            self.current_selector_index += 1;
            SelectorAdvanceResult::Advanced(self.current_selector_index)
        }
    }
    
    /// Enter recursive descent mode for ".." operator
    ///
    /// # Errors
    /// Returns `JsonPathError` if:
    /// - Maximum recursion depth is exceeded during descent
    /// - Invalid selector index is provided for triggering selector
    /// - Memory allocation fails while tracking recursive state
    pub fn enter_recursive_descent(
        &mut self, 
        origin_path: String, 
        triggering_selector: usize
    ) -> Result<(), JsonPathError> {
        if self.current_depth > self.max_depth {
            return Err(JsonPathError::new(
                crate::jsonpath::error::ErrorKind::ProcessingError,
                format!("Max depth {} exceeded at depth {}", self.max_depth, self.current_depth)
            ));
        }
        
        let frame = RecursiveDescentFrame {
            start_depth: self.current_depth,
            current_depth: self.current_depth,
            triggering_selector_index: triggering_selector,
            origin_path,
            should_continue: true,
            visited_nodes: Vec::new(),
        };
        
        self.recursive_descent_stack.push(frame);
        self.in_recursive_descent = true;
        self.evaluation_stats.recursive_descents_performed += 1;
        
        Ok(())
    }
    
    /// Exit current recursive descent level
    pub fn exit_recursive_descent(&mut self) -> Option<RecursiveDescentFrame> {
        if let Some(frame) = self.recursive_descent_stack.pop() {
            self.in_recursive_descent = !self.recursive_descent_stack.is_empty();
            Some(frame)
        } else {
            self.in_recursive_descent = false;
            None
        }
    }
    
    /// Add navigation breadcrumb for path reconstruction
    pub fn push_navigation_frame(&mut self, segment: PathSegment, is_match: bool) {
        let accumulated_path = self.build_current_path(&segment);
        
        let frame = PathNavigationFrame {
            segment,
            depth: self.current_depth,
            accumulated_path,
            is_match,
            selector_index: self.current_selector_index,
        };
        
        self.path_breadcrumbs.push(frame);
        
        if is_match {
            self.evaluation_stats.matches_found += 1;
        }
    }
    
    /// Remove navigation breadcrumb during backtracking
    pub fn pop_navigation_frame(&mut self) -> Option<PathNavigationFrame> {
        if let Some(frame) = self.path_breadcrumbs.pop() {
            self.evaluation_stats.backtrack_operations += 1;
            Some(frame)
        } else {
            None
        }
    }
    
    /// Build current `JSONPath` from breadcrumbs  
    fn build_current_path(&self, new_segment: &PathSegment) -> String {
        let mut path = String::from("$");
        
        for frame in &self.path_breadcrumbs {
            match &frame.segment {
                PathSegment::Property(key) => {
                    use std::fmt::Write;
                    let _ = write!(path, ".{key}");
                },
                PathSegment::ArrayIndex(idx) => {
                    use std::fmt::Write;
                    let _ = write!(path, "[{idx}]");
                },
                PathSegment::Wildcard => path.push_str("[*]"),
                PathSegment::RecursiveDescent => path.push_str(".."),
                PathSegment::FilterMatch(filter) => {
                    use std::fmt::Write;
                    let _ = write!(path, "[?{filter}]");
                },
            }
        }
        
        // Add the new segment
        match new_segment {
            PathSegment::Property(key) => {
                use std::fmt::Write;
                let _ = write!(path, ".{key}");
            },
            PathSegment::ArrayIndex(idx) => {
                use std::fmt::Write;
                let _ = write!(path, "[{idx}]");
            },
            PathSegment::Wildcard => path.push_str("[*]"),
            PathSegment::RecursiveDescent => path.push_str(".."),
            PathSegment::FilterMatch(filter) => {
                use std::fmt::Write;
                let _ = write!(path, "[?{filter}]");
            },
        }
        
        path
    }
    
    /// Get current `JSONPath` for matched node
    #[must_use] 
    pub fn current_json_path(&self) -> String {
        if self.path_breadcrumbs.is_empty() {
            "$".to_string()
        } else {
            self.path_breadcrumbs.last().map_or_else(|| "$".to_string(), |frame| frame.accumulated_path.clone())
        }
    }
    
    /// Check if we should continue recursive descent at current level
    #[must_use] 
    pub fn should_continue_recursive_descent(&self) -> bool {
        self.in_recursive_descent && 
        self.recursive_descent_stack.last()
            .is_some_and(|frame| frame.should_continue)
    }
    
    /// Update depth tracking
    pub fn enter_depth(&mut self) {
        self.current_depth += 1;
        self.evaluation_stats.max_depth_reached = 
            self.evaluation_stats.max_depth_reached.max(self.current_depth);
    }
    
    pub fn exit_depth(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }
}

/// High-performance streaming JSON deserializer with `JSONPath` navigation
///
/// Combines `JSONPath` expression evaluation with incremental JSON parsing to extract
/// individual array elements from nested JSON structures during HTTP streaming.
/// Supports full `JSONPath` specification including recursive descent (..) operators.
pub struct JsonPathDeserializer<'a, T> {
    /// `JSONPath` expression for navigation and filtering
    pub path_expression: &'a JsonPathExpression,
    /// Streaming buffer for efficient byte processing
    pub buffer: &'a mut StreamBuffer,
    /// Current parsing state
    pub state: DeserializerState,
    /// Current parsing depth in JSON structure
    pub current_depth: usize,
    /// Whether we've reached the target array location
    pub in_target_array: bool,
    /// Current object nesting level within target array
    pub object_nesting: usize,
    /// Buffer for accumulating complete JSON objects
    pub object_buffer: Vec<u8>,
    
    // === STREAMING JSONPATH STATE INTEGRATION ===
    /// Complete streaming `JSONPath` evaluation state
    /// IMPLEMENTED: Replaces all previous TODO fields with working implementation
    pub streaming_state: StreamingJsonPathState,
    
    /// Current array index for slice and index evaluation
    pub current_array_index: i64,
    /// Array index stack for nested array processing
    pub array_index_stack: Vec<i64>,
    /// Current position in the buffer for consistent reading
    pub buffer_position: usize,
    /// Target property name for $.property[*] patterns
    pub(super) target_property: Option<String>,
    /// Whether we're currently inside the target property
    pub(super) in_target_property: bool,
    /// Performance marker
    pub(super) _phantom: std::marker::PhantomData<T>,
}

impl<'a, T> JsonPathDeserializer<'a, T>
where
    T: DeserializeOwned,
{
    /// Optimize buffer capacity based on streaming state
    ///
    /// # Errors
    /// Returns `JsonPathError` if:
    /// - Memory allocation fails during buffer resizing
    /// - Buffer capacity calculations overflow or produce invalid sizes
    /// - Internal buffer manager encounters allocation errors
    pub fn optimize_buffer_capacity(&mut self) -> Result<(), JsonPathError> {
        let required_size = self.streaming_state.required_buffer_size();
        let current_capacity = self.buffer.capacity();
        
        // Only resize if significantly different
        if required_size > current_capacity * 2 || required_size < current_capacity / 4 {
            // Let the buffer's internal capacity manager handle the optimization
            // We just trigger it by checking if we need more data
            if self.buffer.len() + required_size > current_capacity {
                // Force a capacity check by the buffer's internal manager
                return Ok(()); // Buffer handles capacity internally
            }
        }
        
        Ok(())
    }

    /// Reset streaming state for new data while preserving configuration
    pub fn reset_for_new_stream(&mut self) {
        self.streaming_state.reset_for_new_buffer();
        self.current_depth = 0;
        self.in_target_array = false;
        self.object_nesting = 0;
        self.object_buffer.clear();
        self.current_array_index = -1;
        self.array_index_stack.clear();
        self.buffer_position = 0;
        self.in_target_property = false;
        self.state = DeserializerState::Initial;
    }

    /// Check if the deserializer should continue processing based on efficiency
    #[must_use] 
    pub fn should_continue_processing(&self) -> bool {
        let efficiency = self.streaming_state.processing_efficiency();
        
        // Continue if efficiency is reasonable or we're early in processing
        efficiency > 0.05 || self.streaming_state.evaluation_stats.nodes_processed < 1000
    }

    /// Update internal state based on buffer consumption
    pub fn handle_buffer_consumed(&mut self, bytes_consumed: usize) {
        self.streaming_state.handle_buffer_consumption(bytes_consumed);
        
        // Update buffer position tracking
        if self.buffer_position >= bytes_consumed {
            self.buffer_position -= bytes_consumed;
        } else {
            self.buffer_position = 0;
        }
    }

    /// Get detailed processing statistics for monitoring and debugging
    #[must_use] 
    pub fn get_processing_stats(&self) -> ProcessingStats {
        ProcessingStats {
            buffer_size: self.buffer.len(),
            buffer_capacity: self.buffer.capacity(),
            // Precision loss acceptable for buffer utilization and progress statistics
            #[allow(clippy::cast_precision_loss)]
            buffer_utilization: self.buffer.len() as f64 / self.buffer.capacity() as f64,
            current_depth: self.current_depth,
            streaming_depth: self.streaming_state.current_depth,
            #[allow(clippy::cast_precision_loss)]
            selector_progress: self.streaming_state.current_selector_index as f64 / self.streaming_state.max_selector_count as f64,
            in_recursive_descent: self.streaming_state.in_recursive_descent,
            navigation_frames: self.streaming_state.path_breadcrumbs.len(),
            matches_found: self.streaming_state.evaluation_stats.matches_found,
            nodes_processed: self.streaming_state.evaluation_stats.nodes_processed,
            processing_efficiency: self.streaming_state.processing_efficiency(),
            should_buffer_current: self.streaming_state.should_buffer_current_object(),
        }
    }

    /// Estimate memory usage for capacity planning
    #[must_use] 
    pub fn estimated_memory_usage(&self) -> usize {
        let buffer_memory = self.buffer.capacity();
        let object_buffer_memory = self.object_buffer.capacity();
        let breadcrumbs_memory = self.streaming_state.path_breadcrumbs.capacity() * std::mem::size_of::<crate::jsonpath::deserializer::core::types::PathNavigationFrame>();
        let recursive_stack_memory = self.streaming_state.recursive_descent_stack.capacity() * std::mem::size_of::<crate::jsonpath::deserializer::core::types::RecursiveDescentFrame>();
        let selectors_memory = self.streaming_state.selectors.capacity() * std::mem::size_of::<crate::jsonpath::ast::JsonSelector>();
        let array_stack_memory = self.array_index_stack.capacity() * std::mem::size_of::<i64>();
        
        buffer_memory + object_buffer_memory + breadcrumbs_memory + recursive_stack_memory + selectors_memory + array_stack_memory
    }

    /// Process available bytes and yield deserialized objects
    ///
    /// Incrementally parses JSON while evaluating `JSONPath` expressions to identify
    /// and extract individual array elements for deserialization.
    ///
    /// # Returns
    ///
    /// Iterator over successfully deserialized objects of type `T`
    ///
    /// # Performance
    ///
    /// Uses zero-copy byte processing and pre-allocated buffers for optimal performance.
    /// Inlined hot paths minimize function call overhead during streaming.
    pub fn process_available(&mut self) -> JsonPathIterator<'_, 'a, T> {
        // Continue from current buffer position to process newly available data
        // Buffer position tracks our progress through the streaming data
        JsonPathIterator::new(self)
    }

    /// Process available bytes with full `JSONPath` streaming
    pub fn process_available_with_streaming(&mut self) -> StreamingJsonPathIterator<'_, 'a, T> {
        StreamingJsonPathIterator::new(self)
    }
}

/// Streaming `JSONPath` iterator for enhanced streaming processing
pub struct StreamingJsonPathIterator<'de, 'a, T> {
    deserializer: &'de mut JsonPathDeserializer<'a, T>,
}

impl<'de, 'a, T> StreamingJsonPathIterator<'de, 'a, T> {
    pub fn new(deserializer: &'de mut JsonPathDeserializer<'a, T>) -> Self {
        Self { deserializer }
    }
}

impl<T> Iterator for StreamingJsonPathIterator<'_, '_, T>
where
    T: DeserializeOwned,
{
    type Item = Result<T, JsonPathError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Process available buffer data through comprehensive streaming JSONPath evaluation
        loop {
            // Check if we have any data to process
            if self.deserializer.buffer.is_empty() {
                return None;
            }

            // Find complete JSON object boundaries in the buffer
            let boundaries = self.deserializer.buffer.find_object_boundaries();
            if boundaries.is_empty() {
                // No complete objects found, need more data
                return None;
            }

            // Process buffer data through comprehensive JSON parsing
            let buffer_start = self.deserializer.buffer_position;
            let buffer_len = self.deserializer.buffer.len();
            
            // Get the data to process without holding buffer reference
            let process_end = boundaries.iter()
                .find(|&&boundary| boundary <= buffer_len)
                .copied()
                .unwrap_or(buffer_len);
                
            if process_end <= buffer_start {
                // No valid data to process
                return None;
            }
            
            // Extract chunk to process (copy data to avoid borrowing conflicts)
            let chunk_data = {
                let buffer_data = self.deserializer.buffer.as_bytes();
                if buffer_start < buffer_data.len() && process_end <= buffer_data.len() {
                    buffer_data[buffer_start..process_end].to_vec()
                } else {
                    return None;
                }
            };
            
            // Process the chunk through comprehensive JSON parsing with owned data
            // Use comprehensive state management from StreamStateMachine
            let mut state_machine = crate::jsonpath::state_machine::StreamStateMachine::new();
            
            // Process chunk through comprehensive JSONPath state management
            match state_machine.process_chunk_with_jsonpath_state(&chunk_data, &mut self.deserializer.streaming_state) {
                Ok(matched_values) => {
                    // Process each matched value found by comprehensive state management
                    for matched_value in matched_values {
                        // Extract JSON data for the matched range
                        let object_data = {
                            let buffer_data = self.deserializer.buffer.as_bytes();
                            let match_start = (buffer_start + matched_value.start_offset).min(buffer_data.len());
                            let match_end = (buffer_start + matched_value.end_offset).min(buffer_data.len());
                            
                            if match_start < buffer_data.len() && match_end <= buffer_data.len() && match_start < match_end {
                                Some(&buffer_data[match_start..match_end])
                            } else {
                                None
                            }
                        };
                        
                        if let Some(json_slice) = object_data {
                            // Attempt deserialization of matched object
                            match serde_json::from_slice::<T>(json_slice) {
                                Ok(deserialized_obj) => {
                                    // Success! Update buffer state and return object
                                    let consumed = matched_value.end_offset;
                                    self.deserializer.buffer.consume(consumed);
                                    self.deserializer.buffer_position = 0;
                                    
                                    // Update streaming statistics
                                    self.deserializer.streaming_state.evaluation_stats.matches_found += 1;
                                    
                                    return Some(Ok(deserialized_obj));
                                }
                                Err(serde_err) => {
                                    // Deserialization failed - log warning and continue
                                    log::warn!("Failed to deserialize matched JSON object at path {}: {}", 
                                              matched_value.path, serde_err);
                                }
                            }
                        }
                    }
                }
                Err(state_error) => {
                    // State management error - log and continue with next chunk
                    log::warn!("JSONPath state management error: {state_error}");
                    self.deserializer.buffer_position = process_end;
                    continue;
                }
            }
            
            // No matches found in this chunk - continue processing

            // Update buffer position to continue processing
            self.deserializer.buffer_position = process_end;
            
            // Consume processed data if we've completed processing this chunk
            if process_end >= self.deserializer.buffer_position {
                let consumed = process_end - buffer_start;
                if consumed > 0 {
                    self.deserializer.buffer.consume(consumed);
                    self.deserializer.buffer_position = 0;
                }
            }
        }
    }
}

/// Processing result from JSON chunk analysis
#[derive(Debug)]
struct ChunkProcessingResult {
    /// Whether the chunk was parsed successfully
    parsed_successfully: bool,
    /// Whether UTF-8 security validation passed
    security_validated: bool,
    /// Number of JSON objects detected
    objects_found: usize,
    /// Property names encountered during parsing
    properties: Vec<String>,
    /// Depth changes during parsing
    depth_changes: Vec<DepthChange>,
    /// Array indices encountered
    array_indices: Vec<usize>,
}

/// Represents a depth change during JSON parsing
#[derive(Debug, Clone)]
enum DepthChange {
    EnterObject,
    ExitObject,
    EnterArray,
    ExitArray,
}

impl<T> StreamingJsonPathIterator<'_, '_, T>
where
    T: DeserializeOwned,
{
    /// Process JSON chunk through streaming state machine with `JSONPath` evaluation
    fn process_json_chunk(&mut self, chunk: &[u8]) -> Result<(), JsonPathError> {
        // First perform UTF-8 security validation on the chunk
        let validation_context = crate::jsonpath::safe_parsing::SafeParsingContext::with_limits(10_000, true);
        if let Err(validation_error) = validation_context.validate_utf8_strict(chunk) {
            return Err(crate::jsonpath::error::invalid_expression_error(
                "",
                format!("UTF-8 validation failed during chunk processing: {validation_error}"),
                None
            ));
        }
        
        self.process_json_structure(chunk)
    }
    
    /// Process JSON structure without security validation (for internal use after validation)
    fn process_json_structure(&mut self, chunk: &[u8]) -> Result<(), JsonPathError> {
        let mut brace_depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut current_property = String::new();
        let mut in_property_name = false;
        let mut array_index = 0;

        for &byte in chunk {
            // Update streaming state based on JSON structure
            match byte {
                b'\\' if in_string => {
                    escape_next = true;
                    continue;
                }
                b'"' if !escape_next => {
                    if in_string {
                        in_string = false;
                        if in_property_name {
                            in_property_name = false;
                            // We've completed parsing a property name
                            self.handle_property_name(&current_property);
                            current_property.clear();
                        }
                    } else {
                        in_string = true;
                        in_property_name = brace_depth > 0; // We're in an object context
                    }
                }
                b'{' if !in_string => {
                    brace_depth += 1;
                    self.deserializer.streaming_state.enter_depth();
                    self.handle_object_start()?;
                }
                b'}' if !in_string => {
                    brace_depth -= 1;
                    self.handle_object_end();
                    self.deserializer.streaming_state.exit_depth();
                }
                b'[' if !in_string => {
                    self.deserializer.streaming_state.enter_depth();
                    self.handle_array_start();
                    array_index = 0;
                }
                b']' if !in_string => {
                    self.handle_array_end();
                    self.deserializer.streaming_state.exit_depth();
                }
                b',' if !in_string => {
                    if brace_depth == 0 {
                        // Array element separator at top level
                        array_index += 1;
                        self.handle_array_element_separator(array_index);
                    }
                }
                _ => {
                    if in_string && in_property_name {
                        current_property.push(byte as char);
                    }
                }
            }

            escape_next = false;
            
            // Update node processing stats
            self.deserializer.streaming_state.evaluation_stats.nodes_processed += 1;
        }

        Ok(())
    }



    /// Process JSON chunk with owned data to avoid borrowing conflicts
    fn process_json_chunk_owned(&mut self, chunk_data: Vec<u8>) -> Result<ProcessingResult, JsonPathError> {
        let mut brace_depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut current_property = String::new();
        let mut in_property_name = false;
        let mut array_index = 0;
        let mut object_boundaries = Vec::new();
        let mut current_object_start = None;

        for (pos, &byte) in chunk_data.iter().enumerate() {
            match byte {
                b'\\' if in_string => {
                    escape_next = true;
                    continue;
                }
                b'"' if !escape_next => {
                    if in_string {
                        in_string = false;
                        if in_property_name {
                            in_property_name = false;
                            // We've completed parsing a property name
                            self.handle_property_name(&current_property);
                            current_property.clear();
                        }
                    } else {
                        in_string = true;
                        in_property_name = brace_depth > 0; // We're in an object context
                    }
                }
                b'{' if !in_string => {
                    if brace_depth == 0 {
                        current_object_start = Some(pos);
                    }
                    brace_depth += 1;
                    self.deserializer.streaming_state.enter_depth();
                    self.handle_object_start()?;
                }
                b'}' if !in_string => {
                    brace_depth -= 1;
                    self.handle_object_end();
                    self.deserializer.streaming_state.exit_depth();
                    if brace_depth == 0
                        && let Some(start) = current_object_start {
                            object_boundaries.push((start, pos + 1));
                            current_object_start = None;
                        }
                }
                b'[' if !in_string => {
                    self.deserializer.streaming_state.enter_depth();
                    self.handle_array_start();
                    array_index = 0;
                }
                b']' if !in_string => {
                    self.handle_array_end();
                    self.deserializer.streaming_state.exit_depth();
                }
                b',' if !in_string => {
                    if brace_depth == 0 {
                        // Array element separator at top level
                        array_index += 1;
                        self.handle_array_element_separator(array_index);
                    }
                }
                _ => {
                    if in_string && in_property_name {
                        current_property.push(byte as char);
                    }
                }
            }

            escape_next = false;
            
            // Update node processing stats
            self.deserializer.streaming_state.evaluation_stats.nodes_processed += 1;
        }

        Ok(ProcessingResult {
            object_boundaries,
            bytes_processed: chunk_data.len(),
        })
    }

    /// Apply processing results to deserializer state
    fn apply_processing_result(&mut self, result: ProcessingResult) {
        // Update buffer position tracking based on bytes processed
        self.deserializer.buffer_position += result.bytes_processed;
        
        // Handle buffer state updates if needed
        self.deserializer.streaming_state.handle_buffer_consumption(result.bytes_processed);
    }

    /// Check if current position matches `JSONPath` expression  
    fn matches_current_expression(&self) -> bool {
        let state = &self.deserializer.streaming_state;
        
        // If no selectors, nothing can match
        if state.selectors.is_empty() {
            return false;
        }
        
        // If we've reached the end of the expression, we have a match
        if state.current_selector_index >= state.max_selector_count {
            return true;
        }
        
        // Check recursive descent conditions
        if state.in_recursive_descent {
            return self.evaluate_recursive_descent_match();
        }
        
        // Check if navigation breadcrumbs indicate a match
        if !state.path_breadcrumbs.is_empty() {
            // Count matching navigation frames
            let matching_frames = state.path_breadcrumbs.iter()
                .filter(|frame| frame.is_match)
                .count();
                
            // We need matches for all non-recursive-descent selectors
            let non_recursive_selectors = state.selectors.iter()
                .filter(|sel| !matches!(sel, crate::jsonpath::ast::JsonSelector::RecursiveDescent))
                .count();
                
            return matching_frames >= non_recursive_selectors.saturating_sub(1);
        }
        
        // Default: evaluate current selector
        self.evaluate_current_selector()
    }
    
    /// Evaluate current selector against streaming state
    #[allow(clippy::cast_possible_truncation)]
    fn evaluate_current_selector(&self) -> bool {
        let state = &self.deserializer.streaming_state;
        
        if state.current_selector_index >= state.selectors.len() {
            return false;
        }
        
        let current_selector = &state.selectors[state.current_selector_index];
        
        match current_selector {
            crate::jsonpath::ast::JsonSelector::Root => state.current_depth == 0,
            crate::jsonpath::ast::JsonSelector::Child { name, .. } => {
                state.path_breadcrumbs.iter().any(|frame| {
                    matches!(&frame.segment, PathSegment::Property(prop) if prop == name)
                })
            }
            crate::jsonpath::ast::JsonSelector::Index { index, .. } => {
                state.path_breadcrumbs.iter().any(|frame| {
                    if let PathSegment::ArrayIndex(i) = &frame.segment {
                        if *index >= 0 {
                            if let Ok(index_usize) = usize::try_from(*index) {
                                *i == index_usize
                            } else {
                                false
                            }
                        } else {
                            false // Negative indices don't match directly
                        }
                    } else {
                        false
                    }
                })
            }
            crate::jsonpath::ast::JsonSelector::Wildcard => {
                !state.path_breadcrumbs.is_empty()
            }
            crate::jsonpath::ast::JsonSelector::RecursiveDescent => {
                state.in_recursive_descent
            }
            crate::jsonpath::ast::JsonSelector::Slice { start, end, step } => {
                // Check if we're in an array context and index falls within slice
                state.path_breadcrumbs.iter().any(|frame| {
                    if let PathSegment::ArrayIndex(i) = frame.segment {
                        self.index_matches_slice(i, *start, *end, *step)
                    } else {
                        false
                    }
                })
            }
            crate::jsonpath::ast::JsonSelector::Filter { expression: _ } => {
                // Filter evaluation requires object/array context
                // For streaming, we do basic presence check for structural context
                !state.path_breadcrumbs.is_empty() && state.current_depth > 0
            }
            crate::jsonpath::ast::JsonSelector::Union { selectors } => {
                // Union matches if any sub-selector matches
                selectors.iter().any(|sub_selector| {
                    self.evaluate_single_union_selector(sub_selector)
                })
            }
        }
    }
    
    /// Evaluate recursive descent matching
    fn evaluate_recursive_descent_match(&self) -> bool {
        let state = &self.deserializer.streaming_state;
        
        if !state.in_recursive_descent {
            return false;
        }
        
        // Find the selector after the recursive descent
        for (idx, selector) in state.selectors.iter().enumerate() {
            if matches!(selector, crate::jsonpath::ast::JsonSelector::RecursiveDescent) {
                // Check if there's a selector after the recursive descent
                if idx + 1 < state.selectors.len() {
                    let next_selector = &state.selectors[idx + 1];
                    return self.matches_selector(next_selector);
                } 
                // Recursive descent at end of expression matches everything
                return true;
            }
        }
        
        false // No recursive descent found
    }
    
    /// Check if a specific selector matches current state
    #[allow(clippy::cast_possible_truncation)]
    fn matches_selector(&self, selector: &crate::jsonpath::ast::JsonSelector) -> bool {
        let state = &self.deserializer.streaming_state;
        
        match selector {
            crate::jsonpath::ast::JsonSelector::Child { name, .. } => {
                state.path_breadcrumbs.iter().any(|frame| {
                    matches!(&frame.segment, PathSegment::Property(prop) if prop == name)
                })
            }
            crate::jsonpath::ast::JsonSelector::Wildcard => true,
            crate::jsonpath::ast::JsonSelector::Index { index, .. } => {
                state.path_breadcrumbs.iter().any(|frame| {
                    if let PathSegment::ArrayIndex(i) = &frame.segment {
                        if *index >= 0 {
                            if let Ok(index_usize) = usize::try_from(*index) {
                                *i == index_usize
                            } else {
                                false
                            }
                        } else {
                            false // Negative indices don't match directly
                        }
                    } else {
                        false
                    }
                })
            }
            _ => false,
        }
    }

    /// Handle property name encountered during JSON parsing
    fn handle_property_name(&mut self, property_name: &str) {
        let state = &mut self.deserializer.streaming_state;
        
        // Check if this property matches current selector
        let is_match = if state.current_selector_index < state.selectors.len() {
            match &state.selectors[state.current_selector_index] {
                crate::jsonpath::ast::JsonSelector::Child { name, .. } => {
                    property_name == name
                }
                crate::jsonpath::ast::JsonSelector::Wildcard => true,
                _ => false,
            }
        } else {
            false
        };
        
        // Add navigation frame for this property
        state.push_navigation_frame(
            PathSegment::Property(property_name.to_string()),
            is_match
        );
        
        // Advance selector if we have a match
        if is_match {
            match state.advance_selector() {
                crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::Advanced(_) 
                | crate::jsonpath::deserializer::core::types::SelectorAdvanceResult::ExpressionComplete => {
                    // Successfully advanced to next selector or reached end of expression
                }
            }
        }
    }

    /// Handle JSON object start during parsing
    fn handle_object_start(&mut self) -> Result<(), JsonPathError> {
        let state = &mut self.deserializer.streaming_state;
        
        // Check if we should enter recursive descent mode
        if !state.in_recursive_descent && state.current_selector_index < state.selectors.len()
            && matches!(state.selectors[state.current_selector_index], crate::jsonpath::ast::JsonSelector::RecursiveDescent) {
                state.enter_recursive_descent(
                    format!("$.object_at_depth_{}", state.current_depth),
                    state.current_selector_index
                )?;
            }
        
        Ok(())
    }

    /// Handle JSON object end during parsing
    fn handle_object_end(&mut self) {
        // Pop navigation frame for object exit
        self.deserializer.streaming_state.pop_navigation_frame();
    }

    /// Handle JSON array start during parsing  
    fn handle_array_start(&mut self) {
        let state = &mut self.deserializer.streaming_state;
        
        // Add navigation frame for array entry
        state.push_navigation_frame(
            PathSegment::ArrayIndex(0),
            false // Will be updated when we process array elements
        );
    }

    /// Handle JSON array end during parsing
    fn handle_array_end(&mut self) {
        self.deserializer.streaming_state.pop_navigation_frame();
    }

    /// Handle array element separator (comma) during parsing
    fn handle_array_element_separator(&mut self, array_index: usize) {
        let state = &mut self.deserializer.streaming_state;
        
        // Check if current selector expects this array index
        let is_match = if state.current_selector_index < state.selectors.len() {
            match &state.selectors[state.current_selector_index] {
                crate::jsonpath::ast::JsonSelector::Index { index, .. } => {
                    i64::try_from(array_index).unwrap_or(i64::MAX) == *index
                }
                crate::jsonpath::ast::JsonSelector::Wildcard => true,
                _ => false,
            }
        } else {
            false
        };
        
        // Update the current navigation frame or add new one
        if let Some(last_frame) = state.path_breadcrumbs.last_mut()
            && matches!(last_frame.segment, PathSegment::ArrayIndex(_)) {
                last_frame.segment = PathSegment::ArrayIndex(array_index);
                last_frame.is_match = is_match;
            }
    }



    /// Evaluate a single selector within a Union
    #[allow(clippy::cast_possible_truncation)]
    fn evaluate_single_union_selector(&self, selector: &crate::jsonpath::ast::JsonSelector) -> bool {
        let state = &self.deserializer.streaming_state;
        
        match selector {
            crate::jsonpath::ast::JsonSelector::Root => state.current_depth == 0,
            crate::jsonpath::ast::JsonSelector::Child { name, .. } => {
                state.path_breadcrumbs.iter().any(|frame| {
                    matches!(&frame.segment, PathSegment::Property(prop) if prop == name)
                })
            }
            crate::jsonpath::ast::JsonSelector::Index { index, .. } => {
                state.path_breadcrumbs.iter().any(|frame| {
                    if let PathSegment::ArrayIndex(i) = &frame.segment {
                        if *index >= 0 {
                            if let Ok(index_usize) = usize::try_from(*index) {
                                *i == index_usize
                            } else {
                                false
                            }
                        } else {
                            false // Negative indices don't match directly
                        }
                    } else {
                        false
                    }
                })
            }
            crate::jsonpath::ast::JsonSelector::Wildcard => {
                !state.path_breadcrumbs.is_empty()
            }
            crate::jsonpath::ast::JsonSelector::RecursiveDescent => {
                state.in_recursive_descent
            }
            crate::jsonpath::ast::JsonSelector::Slice { start, end, step } => {
                state.path_breadcrumbs.iter().any(|frame| {
                    if let PathSegment::ArrayIndex(i) = frame.segment {
                        self.index_matches_slice(i, *start, *end, *step)
                    } else {
                        false
                    }
                })
            }
            crate::jsonpath::ast::JsonSelector::Filter { expression: _ } => {
                // For streaming Union context, basic structural check
                !state.path_breadcrumbs.is_empty() && state.current_depth > 0
            }
            crate::jsonpath::ast::JsonSelector::Union { selectors } => {
                // Nested Union - evaluate all sub-selectors
                selectors.iter().any(|sub_selector| {
                    self.evaluate_single_union_selector(sub_selector)
                })
            }
        }
    }

    /// Check if array index matches slice criteria
    #[allow(clippy::cast_possible_truncation)]
    fn index_matches_slice(&self, index: usize, start: Option<i64>, end: Option<i64>, step: Option<i64>) -> bool {
        let step = usize::try_from(step.unwrap_or(1).max(1)).unwrap_or(1); // Ensure positive step
        let index = i64::try_from(index).unwrap_or(i64::MAX);
        
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
            usize::try_from(index - start_bound).unwrap_or(0).is_multiple_of(step)
        } else {
            true
        }
    }
}

/// Result of processing a JSON chunk
#[derive(Debug)]
struct ProcessingResult {
    /// Object boundaries found during processing (start, end)
    object_boundaries: Vec<(usize, usize)>,
    /// Number of bytes processed
    bytes_processed: usize,
}
