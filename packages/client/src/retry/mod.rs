//! Comprehensive retry logic with exponential backoff and jitter
//!
//! Provides zero-allocation retry mechanism with sophisticated policies,
//! detailed statistics tracking, and streaming execution support.

pub mod executor;
pub mod global;
pub mod helpers;
pub mod policy;

// Re-export main types for convenient access
pub use executor::HttpRetryExecutor;
pub use helpers::{
    execute_with_aggressive_retry, execute_with_conservative_retry, execute_with_default_retry,
    execute_without_retry, with_retry,
};
pub use policy::RetryPolicy;
