use std::fmt;
use std::time::Duration;

use super::types::{Error, Kind};

/// A marker type to indicate that a connection timed out.
#[derive(Debug)]
pub struct TimedOut;

impl fmt::Display for TimedOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("timed out")
    }
}

impl std::error::Error for TimedOut {}

/// A marker type to indicate that a URI scheme was bad.
#[derive(Debug)]
pub struct BadScheme;

impl fmt::Display for BadScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bad scheme")
    }
}

impl std::error::Error for BadScheme {}

/// A marker type to indicate that a connection was closed.
#[derive(Debug)]
pub struct ConnectionClosed;

impl fmt::Display for ConnectionClosed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("connection closed")
    }
}

impl std::error::Error for ConnectionClosed {}

/// A marker type to indicate that an operation was canceled.
#[derive(Debug)]
pub struct OperationCanceled;

impl fmt::Display for OperationCanceled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("operation canceled")
    }
}

impl std::error::Error for OperationCanceled {}

/// A marker type to indicate that a message was incomplete.
#[derive(Debug)]
pub struct IncompleteMessage;

impl fmt::Display for IncompleteMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("incomplete message")
    }
}

impl std::error::Error for IncompleteMessage {}

/// A marker type to indicate that a message was unexpected.
#[derive(Debug)]
pub struct UnexpectedMessage;

impl fmt::Display for UnexpectedMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unexpected message")
    }
}

impl std::error::Error for UnexpectedMessage {}

/// Create a WASM-specific error
#[cfg(target_arch = "wasm32")]
pub fn wasm<E: std::fmt::Debug>(js_error: E) -> Error {
    Error::new(Kind::Request, Some(format!("WASM error: {:?}", js_error)))
}

/// Create a decode error
pub fn decode<E: std::fmt::Debug>(decode_error: E) -> Error {
    Error::new(Kind::Decode).with(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("Decode error: {:?}", decode_error),
    ))
}

/// Create a status code error
pub fn status_code(status: u16) -> Error {
    Error::new(Kind::Request).with(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("HTTP status code error: {status}"),
    ))
}

/// Helper function to create a timeout duration from milliseconds.
pub fn timeout_from_millis(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

/// Helper function to create a timeout duration from seconds.
pub fn timeout_from_secs(secs: u64) -> Duration {
    Duration::from_secs(secs)
}

/// Helper function to check if a duration has elapsed.
pub fn is_timeout_elapsed(start: std::time::Instant, timeout: Duration) -> bool {
    start.elapsed() >= timeout
}

/// Helper function to calculate remaining timeout duration.
pub fn remaining_timeout(start: std::time::Instant, timeout: Duration) -> Option<Duration> {
    let elapsed = start.elapsed();
    if elapsed >= timeout {
        None
    } else {
        Some(timeout - elapsed)
    }
}

/// Helper function to format error messages with context.
pub fn format_error_with_context(error: &str, context: &str) -> String {
    format!("{}: {}", context, error)
}

/// Helper function to chain error messages.
pub fn chain_error_message(original: &str, additional: &str) -> String {
    format!("{} ({})", original, additional)
}
