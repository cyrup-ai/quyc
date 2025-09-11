//! `JSONPath` telemetry and statistics collection
//!
//! This module provides telemetry and metrics collection for `JSONPath` operations,
//! including buffer management and streaming statistics.

pub mod buffer;
pub mod stream;

pub use buffer::*;
// stream re-export removed - not used
