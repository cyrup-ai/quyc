//! JSON Path State Machine Tests
//!
//! Tests for the JSONPath streaming state machine, moved from src/json_path/state_machine.rs

use quyc::jsonpath::{
    JsonPathParser,
    error::stream_error,
    state_machine::{JsonStreamState, StreamStateMachine},
};

#[cfg(test)]
mod state_machine_tests {
    use super::*;

    #[test]
    fn test_state_machine_creation() {
        let sm = StreamStateMachine::new();
        assert!(matches!(sm.state(), JsonStreamState::Initial));
        assert_eq!(sm.stats().objects_yielded, 0);
    }

    #[test]
    fn test_initialization_with_expression() {
        let mut sm = StreamStateMachine::new();
        let expr = JsonPathParser::compile("$.data[*]").expect("Valid JSONPath expression");

        sm.initialize(expr);
        assert!(matches!(sm.state(), JsonStreamState::Navigating { .. }));
    }

    #[test]
    fn test_simple_json_processing() {
        let mut sm = StreamStateMachine::new();
        let expr = JsonPathParser::compile("$[*]").expect("Valid JSONPath expression");
        sm.initialize(expr);

        let json_data = b"[{\"id\":1}]";
        let boundaries = sm.process_bytes(json_data, 0);

        // Simplified test - full implementation would detect object boundaries
        assert!(sm.stats().state_transitions > 0);
    }

    #[test]
    fn test_depth_tracking() {
        let mut sm = StreamStateMachine::new();

        sm.enter_object();
        assert_eq!(sm.stats().current_depth, 1);

        sm.enter_array();
        assert_eq!(sm.stats().current_depth, 2);
        assert_eq!(sm.stats().max_depth, 2);

        sm.exit_object();
        sm.exit_object();
        assert_eq!(sm.stats().current_depth, 0);
        assert_eq!(sm.stats().max_depth, 2); // Max remains
    }

    #[test]
    fn test_state_transitions() {
        let mut sm = StreamStateMachine::new();
        assert_eq!(sm.stats().state_transitions, 0);

        sm.transition_to_complete();
        assert_eq!(sm.stats().state_transitions, 1);
        assert!(sm.is_complete());
    }

    #[test]
    fn test_error_handling() {
        let mut sm = StreamStateMachine::new();
        let error = stream_error("test error", "test", true);

        sm.transition_to_error(error, true);
        assert!(matches!(sm.state(), JsonStreamState::Error { .. }));
        assert_eq!(sm.stats().parse_errors, 1);
    }
}
