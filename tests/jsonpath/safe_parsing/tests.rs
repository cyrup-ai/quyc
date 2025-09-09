//! Tests for safe parsing utilities
//!
//! Comprehensive test suite covering all safe parsing functionality
//! including resource limits, UTF-8 handling, and buffer operations.

use quyc_client::jsonpath::safe_parsing::context::*;
use quyc_client::jsonpath::safe_parsing::utf8::*;
use quyc_client::jsonpath::safe_parsing::buffer::*;

#[test]
fn test_nesting_depth_limits() {
    let mut context = SafeParsingContext::new();

    // Should be able to nest up to limit
    for _ in 0..MAX_NESTING_DEPTH {
        assert!(context.enter_nesting().is_ok());
    }

    // Should fail on exceeding limit
    assert!(context.enter_nesting().is_err());

    // Should be able to exit and enter again
    context.exit_nesting();
    assert!(context.enter_nesting().is_ok());
}

#[test]
fn test_memory_allocation_limits() {
    let mut context = SafeParsingContext::new();

    // Should be able to allocate within limits
    assert!(context.allocate_memory(1000).is_ok());
    assert_eq!(context.allocated_memory(), 1000);

    // Should fail when exceeding limits
    assert!(context.allocate_memory(MAX_BUFFER_SIZE).is_err());
}

#[test]
fn test_utf8_recovery_strategies() {
    let valid_utf8 = b"hello world";
    let invalid_utf8 = b"hello \xFF world";

    // Strict mode should fail on invalid UTF-8
    assert!(
        Utf8Handler::validate_utf8_with_recovery(valid_utf8, Utf8RecoveryStrategy::Strict)
            .is_ok()
    );
    assert!(
        Utf8Handler::validate_utf8_with_recovery(invalid_utf8, Utf8RecoveryStrategy::Strict)
            .is_err()
    );

    // Replace mode should succeed with replacement character
    let replaced =
        Utf8Handler::validate_utf8_with_recovery(invalid_utf8, Utf8RecoveryStrategy::Replace)
            .expect("Failed to validate UTF-8 with replacement strategy");
    assert!(replaced.contains('\u{FFFD}')); // Unicode replacement character

    // Ignore mode should succeed by skipping invalid bytes
    let ignored =
        Utf8Handler::validate_utf8_with_recovery(invalid_utf8, Utf8RecoveryStrategy::Ignore)
            .expect("Failed to validate UTF-8 with ignore strategy");
    assert_eq!(ignored, "hello  world"); // Invalid byte skipped
}

#[test]
fn test_unicode_escape_handling() {
    // Valid Unicode escape
    let result = Utf8Handler::validate_jsonpath_string("hello\\u0041world", true)
        .expect("Failed to validate JSONPath string with Unicode escape");
    assert_eq!(result, "helloAworld");

    // Invalid Unicode escape
    assert!(Utf8Handler::validate_jsonpath_string("hello\\uXXXX", true).is_err());

    // Incomplete Unicode escape
    assert!(Utf8Handler::validate_jsonpath_string("hello\\u00", true).is_err());
}

#[test]
fn test_safe_string_buffer() {
    let mut buffer = SafeStringBuffer::with_capacity(10);

    // Should accept data within limits
    assert!(buffer.append(b"hello").is_ok());
    assert_eq!(buffer.len(), 5);

    // Should accept more data within limits
    assert!(buffer.append(b"world").is_ok());
    assert_eq!(buffer.len(), 10);

    // Should reject data exceeding limits
    assert!(buffer.append(b"!").is_err());

    // Should convert to string successfully
    let string = buffer
        .to_string(Utf8RecoveryStrategy::Strict)
        .expect("Failed to convert buffer to string with strict strategy");
    assert_eq!(string, "helloworld");
}

#[test]
fn test_bom_handling() {
    let with_bom = b"\xEF\xBB\xBFhello";
    let without_bom = b"hello";

    assert_eq!(Utf8Handler::handle_bom(with_bom), b"hello");
    assert_eq!(Utf8Handler::handle_bom(without_bom), b"hello");
}