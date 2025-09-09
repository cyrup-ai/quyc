//! Convenience functions for common retry scenarios
//!
//! Provides simple helper functions that encapsulate common retry patterns
//! for ease of use while maintaining zero-allocation performance.

use ystream::{AsyncStream, prelude::*};

use super::{HttpRetryExecutor, RetryPolicy};

/// Helper function to create retry executor for HTTP operations
///
/// Creates a new HttpRetryExecutor for the given operation closure.
/// The operation will be called once per retry attempt.
#[inline]
pub fn with_retry<F, T>(operation: F) -> HttpRetryExecutor<F, T>
where
    F: Fn() -> AsyncStream<T, 1024> + Send + Sync + 'static,
    T: MessageChunk + Send + Default + 'static,
{
    HttpRetryExecutor::new(operation, RetryPolicy::default())
}

/// Helper function to execute operation with default retry policy
///
/// Convenience wrapper that applies the default retry policy (3 attempts,
/// exponential backoff) to the given operation. Suitable for most HTTP operations.
pub fn execute_with_default_retry<F, T>(operation: F) -> AsyncStream<T, 1024>
where
    F: Fn() -> AsyncStream<T, 1024> + Send + Sync + 'static,
    T: MessageChunk + Send + Default + 'static,
{
    let executor = HttpRetryExecutor::new(operation, RetryPolicy::default());
    executor.execute_with_retry()
}

/// Helper function to execute operation with aggressive retry policy
///
/// Uses the aggressive retry policy (5 attempts, faster backoff) for
/// critical operations that must succeed and can tolerate retry overhead.
pub fn execute_with_aggressive_retry<F, T>(operation: F) -> AsyncStream<T, 1024>
where
    F: Fn() -> AsyncStream<T, 1024> + Send + Sync + 'static,
    T: MessageChunk + Send + Default + 'static,
{
    let executor = HttpRetryExecutor::new(operation, RetryPolicy::aggressive());
    executor.execute_with_retry()
}

/// Helper function to execute operation with conservative retry policy
///
/// Uses the conservative retry policy (2 attempts, longer delays) for
/// non-critical operations that should minimize resource consumption.
pub fn execute_with_conservative_retry<F, T>(operation: F) -> AsyncStream<T, 1024>
where
    F: Fn() -> AsyncStream<T, 1024> + Send + Sync + 'static,
    T: MessageChunk + Send + Default + 'static,
{
    let executor = HttpRetryExecutor::new(operation, RetryPolicy::conservative());
    executor.execute_with_retry()
}

/// Helper function to execute operation without retries
///
/// Uses the no-retry policy (single attempt) for operations that should
/// fail fast without consuming additional resources.
pub fn execute_without_retry<F, T>(operation: F) -> AsyncStream<T, 1024>
where
    F: Fn() -> AsyncStream<T, 1024> + Send + Sync + 'static,
    T: MessageChunk + Send + Default + 'static,
{
    let executor = HttpRetryExecutor::new(operation, RetryPolicy::no_retry());
    executor.execute_with_retry()
}
