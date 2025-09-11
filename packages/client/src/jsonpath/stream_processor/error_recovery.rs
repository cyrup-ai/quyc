//! Error recovery mechanisms for `JSONPath` stream processing
//!
//! Provides strategies for handling and recovering from errors during
//! streaming JSON processing with `JSONPath` evaluation.

use crate::jsonpath::error::JsonPathError;

/// Error recovery strategy
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Skip the current object and continue
    SkipObject,
    /// Reset to beginning of current array
    ResetArray,
    /// Abort processing
    Abort,
    /// Continue with partial data
    ContinuePartial,
}

/// Error recovery context
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    pub current_depth: usize,
    pub in_array: bool,
    pub object_count: usize,
    pub bytes_processed: usize,
}

/// Determine recovery strategy based on error type and context
#[must_use] 
pub fn determine_recovery_strategy(
    error: &JsonPathError,
    context: &RecoveryContext,
) -> RecoveryStrategy {
    match error.kind {
        crate::jsonpath::error::ErrorKind::InvalidJson => {
            if context.in_array && context.object_count > 0 {
                RecoveryStrategy::SkipObject
            } else {
                RecoveryStrategy::Abort
            }
        }
        crate::jsonpath::error::ErrorKind::InvalidPath => RecoveryStrategy::ContinuePartial,
        crate::jsonpath::error::ErrorKind::ProcessingError => RecoveryStrategy::ResetArray,
        crate::jsonpath::error::ErrorKind::IoError => RecoveryStrategy::ContinuePartial,
        _ => RecoveryStrategy::Abort,
    }
}

/// Attempt to recover from error and continue processing
pub fn attempt_recovery(
    strategy: RecoveryStrategy,
    context: &mut RecoveryContext,
) -> bool {
    match strategy {
        RecoveryStrategy::SkipObject => {
            // Skip to next object boundary
            true
        }
        RecoveryStrategy::ResetArray => {
            // Reset array processing state
            context.object_count = 0;
            true
        }
        RecoveryStrategy::ContinuePartial => {
            // Continue with what we have
            true
        }
        RecoveryStrategy::Abort => {
            // Cannot recover
            false
        }
    }
}
