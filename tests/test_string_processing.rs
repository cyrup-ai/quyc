//! Test the JSON string processing implementation
//!
//! Verifies that the string processing correctly handles quotes, escape sequences,
//! and structural elements inside strings vs outside strings.

use quyc_client::jsonpath::state_machine::{
    engine::StreamStateMachine,
    processors::process_object_byte,
    types::{JsonStreamState, ProcessResult},
};

#[test]
fn test_string_boundary_detection() {
    let mut machine = StreamStateMachine::new();
    
    // Set up ProcessingObject state
    machine.state = JsonStreamState::ProcessingObject {
        depth: 1,
        brace_depth: 0,
        in_string: false,
        escaped: false,
    };
    
    // Test quote handling - should toggle string state
    let result = process_object_byte(&mut machine, b'"', 0).expect("Processing quote should succeed");
    assert!(matches!(result, ProcessResult::Continue));
    
    // Verify string state was toggled
    if let JsonStreamState::ProcessingObject { in_string, .. } = &machine.state {
        assert!(*in_string, "Should be inside string after first quote");
    }
    
    // Test another quote - should exit string
    let result = process_object_byte(&mut machine, b'"', 1).expect("Processing quote should succeed");
    assert!(matches!(result, ProcessResult::Continue));
    
    if let JsonStreamState::ProcessingObject { in_string, .. } = &machine.state {
        assert!(!*in_string, "Should be outside string after second quote");
    }
}

#[test]
fn test_escape_sequence_handling() {
    let mut machine = StreamStateMachine::new();
    
    // Set up ProcessingObject state inside string
    machine.state = JsonStreamState::ProcessingObject {
        depth: 1,
        brace_depth: 0,
        in_string: true,
        escaped: false,
    };
    
    // Test backslash - should set escaped state
    let result = process_object_byte(&mut machine, b'\\', 0).expect("Processing backslash should succeed");
    assert!(matches!(result, ProcessResult::Continue));
    
    // Verify escaped state was set
    if let JsonStreamState::ProcessingObject { escaped, .. } = &machine.state {
        assert!(*escaped, "Should be in escaped state after backslash");
    }
    
    // Test escaped quote - should NOT toggle string state
    let result = process_object_byte(&mut machine, b'"', 1).expect("Processing escaped quote should succeed");
    assert!(matches!(result, ProcessResult::Continue));
    
    // Verify still in string and escaped state is reset
    if let JsonStreamState::ProcessingObject { in_string, escaped, .. } = &machine.state {
        assert!(*in_string, "Should still be in string after escaped quote");
        assert!(!*escaped, "Escaped state should be reset after processing quote");
    }
}

#[test]
fn test_braces_inside_strings_ignored() {
    let mut machine = StreamStateMachine::new();
    let initial_depth = machine.stats.current_depth;
    
    // Set up ProcessingObject state inside string
    machine.state = JsonStreamState::ProcessingObject {
        depth: 1,
        brace_depth: 0,
        in_string: true,
        escaped: false,
    };
    
    // Test opening brace inside string - should be ignored
    let result = process_object_byte(&mut machine, b'{', 0).expect("Processing brace in string should succeed");
    assert!(matches!(result, ProcessResult::Continue));
    
    // Verify depth didn't change and brace_depth didn't change
    assert_eq!(machine.stats.current_depth, initial_depth, "Depth should not change for brace inside string");
    
    if let JsonStreamState::ProcessingObject { brace_depth, .. } = &machine.state {
        assert_eq!(*brace_depth, 0, "Brace depth should not change for brace inside string");
    }
    
    // Test closing brace inside string - should be ignored  
    let result = process_object_byte(&mut machine, b'}', 1).expect("Processing brace in string should succeed");
    assert!(matches!(result, ProcessResult::Continue));
    
    // Verify depth didn't change
    assert_eq!(machine.stats.current_depth, initial_depth, "Depth should not change for closing brace inside string");
}

#[test]
fn test_backslash_outside_string_error() {
    let mut machine = StreamStateMachine::new();
    
    // Set up ProcessingObject state outside string
    machine.state = JsonStreamState::ProcessingObject {
        depth: 1,
        brace_depth: 0,
        in_string: false,
        escaped: false,
    };
    
    // Test backslash outside string - should return error
    let result = process_object_byte(&mut machine, b'\\', 0).expect("Should return ProcessResult");
    
    if let ProcessResult::Error(err) = result {
        assert!(err.to_string().contains("unexpected escape character outside string"));
    } else {
        panic!("Expected error for backslash outside string");
    }
}