//! Byte processing implementations for different states
//!
//! This module contains the detailed byte processing logic for each state
//! of the JSON streaming state machine.

use super::types::{ProcessResult, StreamStateMachine};
use crate::jsonpath::error::{JsonPathError, stream_error};

/// Process byte while navigating to JSONPath target
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
            // Start of new object in array
            Ok(ProcessResult::ObjectBoundary {
                start: absolute_offset,
                end: absolute_offset + 1, /* Placeholder - real implementation would track object end */
            })
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
            machine.enter_object();
            Ok(ProcessResult::Continue)
        }
        b'}' => {
            machine.exit_object();
            if machine.stats.current_depth == 0 {
                // Complete object processed
                Ok(ProcessResult::ObjectBoundary {
                    start: absolute_offset,
                    end: absolute_offset + 1,
                })
            } else {
                Ok(ProcessResult::Continue)
            }
        }
        b'"' => {
            // String handling would be implemented here
            // For now, we just continue processing
            Ok(ProcessResult::Continue)
        }
        b':' => {
            // Object key-value separator
            Ok(ProcessResult::Continue)
        }
        b',' => {
            // Object property separator
            Ok(ProcessResult::Continue)
        }
        b' ' | b'\t' | b'\n' | b'\r' => Ok(ProcessResult::Continue), // Skip whitespace
        _ => Ok(ProcessResult::Continue),
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
                &format!("unexpected byte 0x{:02x} in finishing state", byte),
                "finishing",
                false,
            );
            Ok(ProcessResult::Error(err))
        }
    }
}

/// Process string literal byte-by-byte
pub fn process_string_byte(
    _machine: &mut StreamStateMachine,
    byte: u8,
    _in_string: &mut bool,
    _escaped: &mut bool,
) -> Result<ProcessResult, JsonPathError> {
    match byte {
        b'"' => {
            // End of string (unless escaped)
            Ok(ProcessResult::Continue)
        }
        b'\\' => {
            // Escape character
            Ok(ProcessResult::Continue)
        }
        _ => Ok(ProcessResult::Continue),
    }
}

/// Process numeric literal byte-by-byte
pub fn process_numeric_byte(
    _machine: &mut StreamStateMachine,
    byte: u8,
) -> Result<ProcessResult, JsonPathError> {
    match byte {
        b'0'..=b'9' | b'.' | b'-' | b'+' | b'e' | b'E' => {
            // Valid numeric characters
            Ok(ProcessResult::Continue)
        }
        b' ' | b'\t' | b'\n' | b'\r' | b',' | b'}' | b']' => {
            // End of number
            Ok(ProcessResult::Continue)
        }
        _ => {
            let err = stream_error(
                &format!("invalid numeric character 0x{:02x}", byte),
                "numeric_processing",
                false,
            );
            Ok(ProcessResult::Error(err))
        }
    }
}

/// Process boolean or null literal
pub fn process_literal_byte(
    _machine: &mut StreamStateMachine,
    byte: u8,
    expected_literal: &str,
    position: usize,
) -> Result<ProcessResult, JsonPathError> {
    if position < expected_literal.len() {
        let expected_byte = expected_literal.as_bytes()[position];
        if byte == expected_byte {
            Ok(ProcessResult::Continue)
        } else {
            let err = stream_error(
                &format!(
                    "unexpected byte 0x{:02x}, expected 0x{:02x} for literal '{}'",
                    byte, expected_byte, expected_literal
                ),
                "literal_processing",
                false,
            );
            Ok(ProcessResult::Error(err))
        }
    } else {
        // Literal completed
        Ok(ProcessResult::Continue)
    }
}
