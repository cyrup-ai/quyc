//! HTTP client modules with focused separation of concerns
//!
//! Provides a modular HTTP client implementation with excellent separation
//! of concerns across core functionality, execution, statistics, and configuration.

pub mod configuration;
pub mod core;
pub mod stats;

// Re-export main types for convenient access
pub use core::HttpClient;

pub use stats::{ClientStats, ClientStatsSnapshot};
