//! Core proxy types and configuration module
//!
//! Modular organization of proxy functionality including types, constructors,
//! configuration methods, matcher integration, and no-proxy handling.

pub mod configuration;
pub mod constructors;
pub mod debug_impls;
pub mod matcher_integration;
pub mod no_proxy;
pub mod types;

// Re-export main types for backward compatibility
pub use types::{Extra, Intercept, NoProxy, Proxy};
