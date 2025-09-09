//! HTTP3 Builder API modules
//!
//! Provides the complete fluent API for building and executing HTTP requests
//! with zero allocation and elegant method chaining.

pub mod auth;
pub mod body;
pub mod core;
pub mod headers;
pub mod methods;

// Re-export all public types for convenience
// auth::* re-export removed - not used
// body::* re-export removed - not used
pub use core::*;
pub use headers::*;
pub use methods::*;