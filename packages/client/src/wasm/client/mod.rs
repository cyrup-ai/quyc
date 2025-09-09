//! WASM HTTP client module
//!
//! Provides HTTP client functionality for WebAssembly environments,
//! including request building, configuration, and fetch operations.

pub mod builder;
pub mod config;
pub mod core;
pub mod fetch;

// Re-export main types
pub use core::Client;

pub use builder::ClientBuilder;
