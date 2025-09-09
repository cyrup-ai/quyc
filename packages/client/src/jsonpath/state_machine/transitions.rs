//! State transition implementations
//!
//! This module handles all state transitions in the JSON streaming
//! state machine, ensuring proper state management and statistics tracking.

use super::types::{JsonStreamState, StreamStateMachine};
use crate::jsonpath::error::JsonPathError;

/// Transition from Initial to Navigating state
pub fn transition_to_navigating(machine: &mut StreamStateMachine) {
    machine.state = JsonStreamState::Navigating {
        depth: machine.stats.current_depth,
        remaining_selectors: machine
            .path_expression
            .as_ref()
            .map(|e| e.selectors().to_vec())
            .unwrap_or_default(),
        current_value: None,
    };
    machine.stats.state_transitions += 1;
}

/// Transition from Navigating to StreamingArray state
pub fn transition_to_streaming(machine: &mut StreamStateMachine) {
    machine.state = JsonStreamState::StreamingArray {
        target_depth: machine.stats.current_depth,
        current_index: 0,
        in_element: false,
        element_depth: 0,
    };
    machine.stats.state_transitions += 1;
}

/// Transition to ProcessingObject state
pub fn transition_to_processing_object(machine: &mut StreamStateMachine) {
    machine.state = JsonStreamState::ProcessingObject {
        depth: machine.stats.current_depth,
        brace_depth: 0,
        in_string: false,
        escaped: false,
    };
    machine.stats.state_transitions += 1;
}

/// Transition to Finishing state
pub fn transition_to_finishing(machine: &mut StreamStateMachine, expected_closes: usize) {
    machine.state = JsonStreamState::Finishing { expected_closes };
    machine.stats.state_transitions += 1;
}

/// Transition to Complete state
pub fn transition_to_complete(machine: &mut StreamStateMachine) {
    machine.state = JsonStreamState::Complete;
    machine.stats.state_transitions += 1;
}

/// Transition to Error state
pub fn transition_to_error(
    machine: &mut StreamStateMachine,
    error: JsonPathError,
    recoverable: bool,
) {
    machine.state = JsonStreamState::Error { error, recoverable };
    machine.stats.parse_errors += 1;
    machine.stats.state_transitions += 1;
}

/// Attempt to recover from error state
pub fn attempt_recovery(machine: &mut StreamStateMachine) -> bool {
    match &machine.state {
        JsonStreamState::Error { recoverable, .. } => {
            if *recoverable {
                // Reset to initial state for recovery
                machine.state = JsonStreamState::Initial;
                machine.stats.current_depth = 0;
                machine.depth_stack.clear();
                machine.stats.state_transitions += 1;
                true
            } else {
                false
            }
        }
        _ => false, // Not in error state
    }
}

/// Check if current state allows transition to target state
pub fn can_transition_to(current: &JsonStreamState, target_state: StateType) -> bool {
    match (current, target_state) {
        (JsonStreamState::Initial, StateType::Navigating) => true,
        (JsonStreamState::Navigating { .. }, StateType::Streaming) => true,
        (JsonStreamState::Navigating { .. }, StateType::ProcessingObject) => true,
        (JsonStreamState::StreamingArray { .. }, StateType::ProcessingObject) => true,
        (JsonStreamState::ProcessingObject { .. }, StateType::Streaming) => true,
        (_, StateType::Finishing) => true,
        (_, StateType::Complete) => true,
        (_, StateType::Error) => true,
        _ => false,
    }
}

/// State type enumeration for transition validation
#[derive(Debug, Clone, Copy)]
pub enum StateType {
    Initial,
    Navigating,
    Streaming,
    ProcessingObject,
    Finishing,
    Complete,
    Error,
}

/// Get current state type
pub fn get_state_type(state: &JsonStreamState) -> StateType {
    match state {
        JsonStreamState::Initial => StateType::Initial,
        JsonStreamState::Navigating { .. } => StateType::Navigating,
        JsonStreamState::StreamingArray { .. } => StateType::Streaming,
        JsonStreamState::ProcessingObject { .. } => StateType::ProcessingObject,
        JsonStreamState::Finishing { .. } => StateType::Finishing,
        JsonStreamState::Complete => StateType::Complete,
        JsonStreamState::Error { .. } => StateType::Error,
    }
}

/// Validate state transition
pub fn validate_transition(
    machine: &StreamStateMachine,
    target_state: StateType,
) -> Result<(), JsonPathError> {
    let current_type = get_state_type(&machine.state);

    if can_transition_to(&machine.state, target_state) {
        Ok(())
    } else {
        Err(crate::jsonpath::error::stream_error(
            &format!(
                "invalid state transition from {:?} to {:?}",
                current_type, target_state
            ),
            "state_validation",
            false,
        ))
    }
}

/// Force state transition (for recovery scenarios)
pub fn force_transition_to_state(machine: &mut StreamStateMachine, new_state: JsonStreamState) {
    machine.state = new_state;
    machine.stats.state_transitions += 1;
}

/// Reset state machine to initial state
pub fn reset_to_initial(machine: &mut StreamStateMachine) {
    machine.state = JsonStreamState::Initial;
    machine.stats.current_depth = 0;
    machine.depth_stack.clear();
    machine.stats.state_transitions += 1;
}

/// Check if state machine is in a terminal state
pub fn is_terminal_state(state: &JsonStreamState) -> bool {
    matches!(
        state,
        JsonStreamState::Complete
            | JsonStreamState::Error {
                recoverable: false,
                ..
            }
    )
}

/// Get state machine readiness for processing
pub fn is_ready_for_processing(machine: &StreamStateMachine) -> bool {
    match &machine.state {
        JsonStreamState::Initial => machine.path_expression.is_some(),
        JsonStreamState::Error { recoverable, .. } => *recoverable,
        _ => !is_terminal_state(&machine.state),
    }
}
