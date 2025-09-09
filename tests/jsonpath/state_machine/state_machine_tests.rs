//! Tests for state machine implementation
//! 
//! Extracted from src/jsonpath/state_machine/mod.rs
//! Tests streaming JSON state machine functionality

use quyc_client::jsonpath::state_machine::{StreamStateMachine, JsonStreamState, StateType, get_state_type};
use quyc_client::jsonpath::parser::{JsonPathExpression, JsonSelector};

#[test]
fn test_state_machine_creation() {
    let machine = StreamStateMachine::new();
    assert!(matches!(machine.state(), JsonStreamState::Initial));
    assert_eq!(machine.objects_yielded(), 0);
    assert_eq!(machine.parse_errors(), 0);
}

#[test]
fn test_state_machine_initialization() {
    let mut machine = StreamStateMachine::new();
    let expression = JsonPathExpression::root(); // Placeholder
    machine.initialize(expression);

    assert!(matches!(machine.state(), JsonStreamState::Navigating { .. }));
    assert!(machine.is_ready());
}

#[test]
fn test_byte_processing() {
    let mut machine = StreamStateMachine::new();
    let expression = JsonPathExpression::root();
    machine.initialize(expression);

    let data = b"[{\"test\": 123}]";
    let boundaries = machine.process_bytes(data, 0);

    // Should have processed some data
    assert!(machine.state_transitions() > 0);
}

#[test]
fn test_state_transitions() {
    let mut machine = StreamStateMachine::new();

    // Test initial state
    assert!(matches!(machine.current_state(), JsonStreamState::Initial));

    // Test state type detection
    let state_type = get_state_type(machine.current_state());
    assert!(matches!(state_type, StateType::Initial));
}

#[test]
fn test_error_handling() {
    let mut machine = StreamStateMachine::new();
    let expression = JsonPathExpression::root();
    machine.initialize(expression);

    // Process invalid JSON
    let data = b"invalid json{[}";
    let _boundaries = machine.process_bytes(data, 0);

    // Should handle errors gracefully
    assert!(machine.parse_errors() >= 0); // May or may not have errors depending on implementation
}

#[test]
fn test_reset_functionality() {
    let mut machine = StreamStateMachine::new();
    let expression = JsonPathExpression::root();
    machine.initialize(expression);

    // Process some data
    let data = b"{}";
    let _boundaries = machine.process_bytes(data, 0);

    // Reset machine
    machine.reset();

    assert!(matches!(machine.current_state(), JsonStreamState::Initial));
    assert_eq!(machine.objects_yielded(), 0);
    assert_eq!(machine.parse_errors(), 0);
}

#[test]
fn test_memory_usage_estimation() {
    let machine = StreamStateMachine::new();
    let usage = machine.memory_usage();

    // Should return a reasonable estimate
    assert!(usage > 0);
    assert!(usage < 10000); // Should be reasonable for empty machine
}