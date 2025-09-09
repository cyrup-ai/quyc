//! Safe parsing utilities with UTF-8 decode error handling and memory protection
//!
//! Provides robust parsing capabilities that handle malformed input gracefully
//! and protect against memory exhaustion attacks through deep nesting or
//! extremely large expressions.

pub mod buffer;
pub mod context;
pub mod utf8;

// Re-export all public types and functions
pub use buffer::SafeStringBuffer;
pub use context::{
    MAX_BUFFER_SIZE, MAX_COMPLEXITY_SCORE, MAX_NESTING_DEPTH, MAX_PARSE_TIME, SafeParsingContext,
    global_memory_usage, reset_global_memory_tracking,
};
pub use utf8::{Utf8Handler, Utf8RecoveryStrategy};