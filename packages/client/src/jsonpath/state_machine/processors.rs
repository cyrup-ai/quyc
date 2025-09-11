//! Byte processing implementations for different states
//!
//! This module contains the detailed byte processing logic for each state
//! of the JSON streaming state machine.

use super::types::{JsonStreamState, ProcessResult, StreamStateMachine};
use crate::jsonpath::error::{JsonPathError, stream_error};

/// Process byte while navigating to `JSONPath` target
pub fn process_navigating_byte(
    machine: &mut StreamStateMachine,
    byte: u8,
) -> Result<ProcessResult, JsonPathError> {
    match byte {
        b' ' | b'\t' | b'\n' | b'\r' => Ok(ProcessResult::Continue), // Skip whitespace
        b'{' => {
            machine.enter_object();
            Ok(ProcessResult::Continue)
        }
        b'[' => {
            machine.enter_array();
            if super::utils::is_target_array(machine) {
                super::transitions::transition_to_streaming(machine);
            }
            Ok(ProcessResult::Continue)
        }
        b'}' | b']' => {
            if byte == b'}' {
                machine.exit_object();
            } else {
                machine.exit_array();
            }
            Ok(ProcessResult::Continue)
        }
        _ => Ok(ProcessResult::Continue), // Continue navigating
    }
}

/// Process byte while streaming array elements
pub fn process_streaming_byte(
    machine: &mut StreamStateMachine,
    byte: u8,
    absolute_offset: usize,
) -> Result<ProcessResult, JsonPathError> {
    match byte {
        b'{' => {
            // Start of new object in array - track start position and transition to ProcessingObject state
            machine.stats.object_start_offset = Some(absolute_offset);
            machine.enter_object();
            super::transitions::transition_to_processing_object(machine);
            Ok(ProcessResult::Continue)
        }
        b'[' => {
            machine.enter_array();
            Ok(ProcessResult::Continue)
        }
        b']' => {
            machine.exit_array();
            if machine.stats.current_depth == 0 {
                super::transitions::transition_to_complete(machine);
            }
            Ok(ProcessResult::Continue)
        }
        b',' => {
            // Array element separator
            Ok(ProcessResult::Continue)
        }
        b' ' | b'\t' | b'\n' | b'\r' => Ok(ProcessResult::Continue), // Skip whitespace
        _ => Ok(ProcessResult::Continue),
    }
}

/// Process byte while processing JSON object
pub fn process_object_byte(
    machine: &mut StreamStateMachine,
    byte: u8,
    absolute_offset: usize,
) -> Result<ProcessResult, JsonPathError> {
    match byte {
        b'{' => {
            // Only process structural braces when not inside strings
            let mut should_process_brace = false;
            if let JsonStreamState::ProcessingObject { in_string, escaped, brace_depth, .. } = &mut machine.state {
                should_process_brace = !*in_string || *escaped;
                if should_process_brace {
                    *brace_depth += 1;
                }
                *escaped = false; // Reset escape state
            }
            if should_process_brace {
                machine.enter_object();
            }
            Ok(ProcessResult::Continue)
        }
        b'}' => {
            // Only process structural braces when not inside strings
            let mut should_process_brace = false;
            let mut target_depth = 0;
            if let JsonStreamState::ProcessingObject { in_string, escaped, brace_depth, depth, .. } = &mut machine.state {
                should_process_brace = !*in_string || *escaped;
                target_depth = *depth;
                if should_process_brace && *brace_depth > 0 {
                    *brace_depth -= 1;
                }
                *escaped = false; // Reset escape state
            }
            
            if should_process_brace {
                machine.exit_object();
                // Object is complete when we exit back to the target depth
                if machine.stats.current_depth == target_depth {
                    // Return to streaming state and emit ObjectBoundary
                    super::transitions::transition_to_streaming(machine);
                    // Use tracked start position for accurate boundaries
                    let start_offset = machine.stats.object_start_offset.unwrap_or(absolute_offset);
                    machine.stats.object_start_offset = None; // Reset for next object
                    return Ok(ProcessResult::ObjectBoundary {
                        start: start_offset,
                        end: absolute_offset + 1,
                    });
                }
            }
            Ok(ProcessResult::Continue)
        }
        b'"' => {
            // Handle string boundary detection for accurate object parsing
            if let JsonStreamState::ProcessingObject { in_string, escaped, .. } = &mut machine.state {
                if !*escaped {
                    // Toggle string state on unescaped quote
                    *in_string = !*in_string;
                }
                // Reset escape state after processing quote
                *escaped = false;
            }
            Ok(ProcessResult::Continue)
        }
        b':' => {
            // Object key-value separator - reset escape state
            if let JsonStreamState::ProcessingObject { escaped, .. } = &mut machine.state {
                *escaped = false;
            }
            Ok(ProcessResult::Continue)
        }
        b',' => {
            // Object property separator - reset escape state
            if let JsonStreamState::ProcessingObject { escaped, .. } = &mut machine.state {
                *escaped = false;
            }
            Ok(ProcessResult::Continue)
        }
        b' ' | b'\t' | b'\n' | b'\r' => Ok(ProcessResult::Continue), // Skip whitespace
        b'\\' => {
            // Handle escape sequences in strings
            if let JsonStreamState::ProcessingObject { in_string, escaped, .. } = &mut machine.state {
                if *in_string {
                    // Set escape state only when inside a string
                    *escaped = true;
                } else {
                    // Backslash outside string - invalid JSON
                    return Ok(ProcessResult::Error(stream_error(
                        "unexpected escape character outside string",
                        "object_processing",
                        false,
                    )));
                }
            }
            Ok(ProcessResult::Continue)
        }
        _ => {
            // Validate escape sequences and reset escape state
            if let JsonStreamState::ProcessingObject { escaped, in_string, .. } = &mut machine.state {
                if *escaped && *in_string {
                    // Validate escape sequence character
                    match byte {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                            // Valid JSON escape sequences
                            *escaped = false;
                            Ok(ProcessResult::Continue)
                        }
                        b'u' => {
                            // Unicode escape sequence - for now, accept and reset (full \uXXXX validation would require state tracking)
                            // Production implementation would track the following 4 hex digits
                            *escaped = false;
                            Ok(ProcessResult::Continue)
                        }
                        _ => {
                            // Invalid escape sequence
                            *escaped = false;
                            Ok(ProcessResult::Error(stream_error(
                                format!("invalid escape sequence '\\{}'", byte as char),
                                "object_processing",
                                false,
                            )))
                        }
                    }
                } else {
                    // Normal character processing - reset escape state
                    *escaped = false;
                    Ok(ProcessResult::Continue)
                }
            } else {
                Ok(ProcessResult::Continue)
            }
        }
    }
}

/// Process byte in finishing state
pub fn process_finishing_byte(
    _machine: &mut StreamStateMachine,
    byte: u8,
) -> Result<ProcessResult, JsonPathError> {
    match byte {
        b' ' | b'\t' | b'\n' | b'\r' => Ok(ProcessResult::Continue), // Skip whitespace
        b'}' | b']' => {
            // Handle closing brackets/braces
            Ok(ProcessResult::Continue)
        }
        _ => {
            let err = stream_error(
                format!("unexpected byte 0x{byte:02x} in finishing state"),
                "finishing",
                false,
            );
            Ok(ProcessResult::Error(err))
        }
    }
}


