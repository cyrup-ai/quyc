//! State types for Http3Builder typestate pattern
//!
//! Provides marker types to track builder state at compile time, ensuring
//! proper usage patterns and preventing invalid method calls.

/// State marker indicating no body has been set
#[derive(Debug, Clone, Copy)]
pub struct BodyNotSet;

/// State marker indicating a body has been set
#[derive(Debug, Clone, Copy)]
pub struct BodySet;

/// JSONPath streaming configuration state
#[derive(Debug, Clone)]
pub struct JsonPathStreaming {
    /// JSONPath expression for filtering JSON array responses
    pub jsonpath_expr: String,
}
