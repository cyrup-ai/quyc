//! JSON streaming state machine with zero-allocation parsing
//!
//! This module implements a high-performance state machine for streaming JSON parsing
//! and JSONPath evaluation. Optimized for scenarios where JSON arrives in chunks
//! and needs incremental processing with minimal memory allocation.
//!
//! # Architecture
//!
//! - `types`: Core data structures, enums, and type definitions
//! - `engine`: Main processing engine and byte-level parsing
//! - `processors`: Specialized byte processors for different parsing states
//! - `transitions`: State transition logic and validation
//! - `utils`: Utility functions and public API methods
//!
//! # Usage
//!
//! ```rust
//! use crate::json_path::state_machine::StreamStateMachine;
//! use crate::json_path::parser::JsonPathExpression;
//!
//! let mut machine = StreamStateMachine::new();
//! machine.initialize(expression);
//!
//! let boundaries = machine.process_bytes(data, offset);
//! for boundary in boundaries {
//!     println!("Found object at bytes {}..{}", boundary.start, boundary.end);
//! }
//! ```
//!
//! # Performance
//!
//! This state machine is optimized for:
//! - Zero-allocation streaming parsing
//! - Single-pass byte processing
//! - Minimal branching in hot paths
//! - Incremental JSONPath evaluation
//! - Memory-efficient state tracking

#![allow(dead_code)]

mod engine;
mod processors;
mod transitions;
mod types;
mod utils;

// Re-export the main types and functionality
// Re-export transition functions for advanced usage
pub use transitions::{StateType, get_state_type, is_ready_for_processing, is_terminal_state};
pub use types::{
    FrameIdentifier, JsonStreamState, JsonStructureType, ObjectBoundary, ProcessResult, StateStats,
    StreamStateMachine,
};
// Re-export utility functions that might be needed externally
pub use utils::{
    current_depth, is_complete, is_error_state, is_recoverable_error, max_depth_reached,
};




