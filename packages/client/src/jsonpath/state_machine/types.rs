//! State machine types and data structures
//!
//! This module contains all the type definitions, enums, and data structures
//! used by the JSON streaming state machine.

use std::collections::VecDeque;

use serde_json::Value;

use crate::jsonpath::{
    error::JsonPathError,
    parser::{JsonPathExpression, JsonSelector},
};

/// Current state of JSON streaming and `JSONPath` evaluation
#[derive(Debug, Clone)]
pub enum JsonStreamState {
    /// Initial state - waiting for JSON to begin
    Initial,

    /// Navigating to the target `JSONPath` location
    Navigating {
        /// Current depth in JSON structure
        depth: usize,
        /// `JSONPath` selectors remaining to process
        remaining_selectors: Vec<JsonSelector>,
        /// Current JSON value being processed
        current_value: Option<Value>,
    },

    /// Streaming array elements at target location
    StreamingArray {
        /// Array depth where streaming occurs
        target_depth: usize,
        /// Current array index being processed
        current_index: usize,
        /// Whether we're inside an array element
        in_element: bool,
        /// Brace/bracket nesting depth within current element
        element_depth: usize,
    },

    /// Processing individual JSON object at target location
    ProcessingObject {
        /// Object depth in JSON structure
        depth: usize,
        /// Current brace nesting depth
        brace_depth: usize,
        /// Whether we're inside a string literal
        in_string: bool,
        /// Whether the previous character was an escape character
        escaped: bool,
    },

    /// Finishing stream processing
    Finishing {
        /// Number of closing braces/brackets expected
        expected_closes: usize,
    },

    /// Stream processing completed successfully
    Complete,

    /// Error state - unrecoverable error occurred
    Error {
        /// The error that occurred
        error: JsonPathError,
        /// Whether recovery is possible
        recoverable: bool,
    },
}

/// Type of JSON structure
#[derive(Debug, Clone, Copy)]
pub enum JsonStructureType {
    /// JSON object structure (enclosed in {})
    Object,
    /// JSON array structure (enclosed in [])
    Array,
    /// JSON primitive value (string, number, boolean, null)
    Value,
}

/// Identifier for current frame
#[derive(Debug, Clone)]
pub enum FrameIdentifier {
    /// Object property name
    Property(String),
    /// Array index
    Index(usize),
    /// Root element
    Root,
}

/// State machine performance statistics
#[derive(Debug, Clone, Default)]
pub struct StateStats {
    /// Total objects yielded to application
    pub objects_yielded: u64,
    /// Parse errors encountered (recoverable)
    pub parse_errors: u64,
    /// State transitions performed
    pub state_transitions: u64,
    /// Maximum depth reached
    pub max_depth: usize,
    /// Current processing depth
    pub current_depth: usize,
    /// Start offset of current object being processed
    pub object_start_offset: Option<usize>,
}

/// Main streaming state machine
#[derive(Debug)]
pub struct StreamStateMachine {
    /// Current state of the state machine
    pub(super) state: JsonStreamState,
    /// Performance and debugging statistics
    pub(super) stats: StateStats,
    /// Optional `JSONPath` expression being evaluated
    pub(super) path_expression: Option<JsonPathExpression>,
    /// Stack tracking nested JSON structures
    pub(super) depth_stack: VecDeque<FrameIdentifier>,
}

/// Result of processing a single byte
#[derive(Debug)]
pub enum ProcessResult {
    /// Continue processing next byte
    Continue,
    /// Complete JSON object found at boundary
    ObjectBoundary { start: usize, end: usize },
    /// Need more data to continue processing
    NeedMoreData,
    /// Stream processing complete
    Complete,
    /// Error occurred during processing
    Error(JsonPathError),
}

/// Boundary of complete JSON object in stream
#[derive(Debug, Clone, Copy)]
pub struct ObjectBoundary {
    /// Start byte offset of object
    pub start: usize,
    /// End byte offset of object (exclusive)
    pub end: usize,
}
