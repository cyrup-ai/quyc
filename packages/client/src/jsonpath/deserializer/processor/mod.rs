//! JSON byte-by-byte processing logic
//!
//! Contains the core logic for processing individual JSON bytes during streaming,
//! including state transitions and `JSONPath` evaluation integration.
//!
//! NOTE: Many methods in this module appear to duplicate functionality from core.rs.
//! These are preserved as they may represent different architectural approaches.
#![allow(dead_code)]

pub mod array_selectors;
pub mod core;
pub mod path_evaluation;
pub mod state_processors;

// Re-export main types for backward compatibility
pub use core::JsonProcessResult;
