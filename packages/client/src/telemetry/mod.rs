//! Telemetry and metrics collection for HTTP3 package
//!
//! This module consolidates all telemetry, metrics, and statistics functionality
//! from across the HTTP3 package into a single, well-organized location.

pub mod cache_stats;
pub mod client_stats;
pub mod jsonpath;
pub mod metrics;
pub mod retry_stats;
pub mod types;

// Re-export key types for convenience
pub use cache_stats::*;
pub use client_stats::*;
pub use jsonpath::*;
pub use metrics::*;
pub use retry_stats::*;
