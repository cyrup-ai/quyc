//! Proxy builder modules
//!
//! Decomposed proxy builder implementation with logical separation of concerns:
//!
//! - `types`: Core Proxy struct and ProxyIntercept enum definitions
//! - `constructors`: Static methods for creating proxy configurations
//! - `configuration`: Instance methods for auth, headers, and no-proxy settings
//!
//! All modules maintain production-quality error handling and comprehensive tests.

pub mod types;
pub mod constructors;
pub mod configuration;

// Re-export the main type for backward compatibility
pub use types::{Proxy, ProxyIntercept};

