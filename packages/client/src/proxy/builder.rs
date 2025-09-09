//! Proxy builder API and public interface
//!
//! Re-exports from decomposed proxy builder modules for backward compatibility.
//! The implementation has been split into logical modules for better maintainability.

pub mod types;
pub mod constructors;
pub mod configuration;

// Re-export the main types for backward compatibility
pub use types::{Proxy, ProxyIntercept};