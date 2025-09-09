//! HTTP/3 connector builder with zero-allocation configuration
//!
//! Provides ergonomic builder pattern for connector configuration with production-quality defaults.
//!
//! # Modules
//!
//! - [`types`] - Core ConnectorBuilder struct and basic configuration methods
//! - [`tls`] - TLS-specific constructors and configuration (native-tls and rustls)
//! - [`build`] - Final build logic and connector creation

pub mod build;
pub mod tls;
pub mod types;

// Re-export the main type for backward compatibility
pub use types::ConnectorBuilder;
