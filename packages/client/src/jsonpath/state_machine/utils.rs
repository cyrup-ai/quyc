//! Utility functions and helper methods
//!
//! This module contains utility functions, helper methods, and public API
//! functions for the state machine.

use super::types::{JsonStreamState, StateStats, StreamStateMachine};
use crate::jsonpath::error::JsonPathError;

/// Check if current JSONPath target is an array for streaming
pub fn is_target_array(machine: &StreamStateMachine) -> bool {
    // Simplified check - full implementation would evaluate JSONPath expression
    machine
        .path_expression
        .as_ref()
        .map(|e| e.is_array_stream())
        .unwrap_or(false)
}

/// Get current error if state machine is in error state
pub fn current_error(machine: &StreamStateMachine) -> Option<JsonPathError> {
    if let JsonStreamState::Error { error, .. } = &machine.state {
        Some(error.clone())
    } else {
        None
    }
}

/// Check if stream processing is complete
#[inline]
pub fn is_complete(machine: &StreamStateMachine) -> bool {
    matches!(machine.state, JsonStreamState::Complete)
}

/// Check if state machine is in error state
#[inline]
pub fn is_error_state(machine: &StreamStateMachine) -> bool {
    matches!(machine.state, JsonStreamState::Error { .. })
}

/// Check if error is recoverable
pub fn is_recoverable_error(machine: &StreamStateMachine) -> bool {
    if let JsonStreamState::Error { recoverable, .. } = &machine.state {
        *recoverable
    } else {
        false
    }
}

/// Get current processing depth
#[inline]
pub fn current_depth(machine: &StreamStateMachine) -> usize {
    machine.stats.current_depth
}

/// Get maximum depth reached during processing
#[inline]
pub fn max_depth_reached(machine: &StreamStateMachine) -> usize {
    machine.stats.max_depth
}

/// Check if currently processing at target depth
pub fn at_target_depth(machine: &StreamStateMachine) -> bool {
    match &machine.state {
        JsonStreamState::StreamingArray { target_depth, .. } => {
            machine.stats.current_depth >= *target_depth
        }
        JsonStreamState::ProcessingObject { depth, .. } => machine.stats.current_depth >= *depth,
        _ => false,
    }
}

/// Estimate memory usage of state machine
pub fn estimate_memory_usage(machine: &StreamStateMachine) -> usize {
    let base_size = std::mem::size_of::<StreamStateMachine>();
    let stack_size =
        machine.depth_stack.len() * std::mem::size_of::<super::types::FrameIdentifier>();
    let expression_size = machine
        .path_expression
        .as_ref()
        .map(|_| 256) // Estimated size
        .unwrap_or(0);

    base_size + stack_size + expression_size
}

/// Implementation of public API methods for StreamStateMachine
impl StreamStateMachine {
    /// Get current state for debugging
    pub fn current_state(&self) -> &JsonStreamState {
        &self.state
    }

    /// Get processing statistics
    pub fn stats(&self) -> &StateStats {
        &self.stats
    }

    /// Get number of objects successfully yielded
    #[inline]
    pub fn objects_yielded(&self) -> u64 {
        self.stats.objects_yielded
    }

    /// Get number of parse errors encountered
    #[inline]
    pub fn parse_errors(&self) -> u64 {
        self.stats.parse_errors
    }

    /// Get number of state transitions performed
    #[inline]
    pub fn state_transitions(&self) -> u64 {
        self.stats.state_transitions
    }

    /// Check if stream processing is complete
    #[inline]
    pub fn is_complete(&self) -> bool {
        is_complete(self)
    }

    /// Check if state machine is in error state
    #[inline]
    pub fn is_error(&self) -> bool {
        is_error_state(self)
    }

    /// Check if current error is recoverable
    pub fn is_recoverable(&self) -> bool {
        is_recoverable_error(self)
    }

    /// Get current processing depth
    #[inline]
    pub fn depth(&self) -> usize {
        current_depth(self)
    }

    /// Get maximum depth reached
    #[inline]
    pub fn max_depth(&self) -> usize {
        max_depth_reached(self)
    }

    /// Reset state machine for new stream
    pub fn reset(&mut self) {
        super::transitions::reset_to_initial(self);
        self.stats = StateStats::default();
        self.path_expression = None;
    }

    /// Attempt to recover from error state
    pub fn recover(&mut self) -> bool {
        super::transitions::attempt_recovery(self)
    }

    /// Check if ready for processing
    pub fn is_ready(&self) -> bool {
        super::transitions::is_ready_for_processing(self)
    }

    /// Get estimated memory usage
    pub fn memory_usage(&self) -> usize {
        estimate_memory_usage(self)
    }

    /// Check if at target JSONPath depth
    pub fn at_target_depth(&self) -> bool {
        at_target_depth(self)
    }

    /// Get current error details
    pub fn current_error(&self) -> Option<JsonPathError> {
        current_error(self)
    }

    /// Force reset to specific state (for testing/recovery)
    #[cfg(test)]
    pub fn force_state(&mut self, state: JsonStreamState) {
        super::transitions::force_transition_to_state(self, state);
    }
}
