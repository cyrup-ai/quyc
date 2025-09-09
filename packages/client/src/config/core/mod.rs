//! Core HTTP configuration types and default implementations
//!
//! This module provides the main HttpConfig struct and related enums for HTTP client configuration.
//! The configuration is organized into logical modules:
//!
//! - `types`: Core HttpConfig struct definition with all configuration fields
//! - `enums`: Connection reuse strategies, retry policies, and error classifications
//! - `defaults`: Sensible default values optimized for HTTP/3 usage patterns
//! - `builders`: Fluent builder methods for common configuration scenarios
//!
//! All modules maintain production-quality code standards and comprehensive documentation.

pub mod builders;
pub mod defaults;
pub mod enums;
pub mod retry;
pub mod types;

// Re-export all main types for backward compatibility
pub use enums::{ConnectionReuse, RetryPolicy, RetryableError};
pub use types::HttpConfig;
