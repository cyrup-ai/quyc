//! HTTP configuration module
//!
//! Provides comprehensive HTTP client configuration including core types,
//! preset configurations, timeout settings, and security options.

// Module declarations
pub mod client;
pub mod core;
pub mod security;
pub mod timeouts;

// Re-export all public types for backward compatibility
pub use core::{ConnectionReuse, HttpConfig, RetryPolicy, RetryableError};

// Note: Preset configuration methods are available as HttpConfig::ai_optimized(), etc.
// Security and timeout methods are available as HttpConfig methods.
// No re-exports needed as all functionality is accessed via HttpConfig.
