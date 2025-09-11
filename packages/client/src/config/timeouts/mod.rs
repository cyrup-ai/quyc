//! Timeout and connection-related configuration methods
//!
//! Provides builder methods for configuring timeouts, connection pooling,
//! and QUIC window sizes for optimal performance tuning.

pub mod basic_timeouts;
pub mod connection_pool;
pub mod keepalive;
pub mod quic_config;
pub mod timeout_config;

// Re-export main timeout configuration types
pub use timeout_config::{TimeoutConfig, TimeoutConfigProvider, StaticTimeoutConfig};
