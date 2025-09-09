use ystream::prelude::*;

use super::RetryPolicy;

/// Retry executor for HTTP operations using pure AsyncStreams
pub struct HttpRetryExecutor<F, T>
where
    F: Fn() -> AsyncStream<T, 1024> + Send + Sync + 'static,
    T: MessageChunk + Send + Default + 'static,
{
    operation: std::sync::Arc<F>,
    policy: RetryPolicy,
}

impl<F, T> HttpRetryExecutor<F, T>
where
    F: Fn() -> AsyncStream<T, 1024> + Send + Sync + 'static,
    T: MessageChunk + Send + Default + 'static,
{
    /// Create new retry executor for HTTP operation
    pub fn new(operation: F, policy: RetryPolicy) -> Self {
        Self {
            operation: std::sync::Arc::new(operation),
            policy,
        }
    }

    /// Execute operation with retry logic using pure streaming patterns
    pub fn execute_with_retry(&self) -> AsyncStream<T, 1024> {
        let operation = std::sync::Arc::clone(&self.operation);
        let policy = self.policy.clone();

        AsyncStream::<T>::with_channel(move |sender| {
            let mut attempt = 1;

            loop {
                // Execute the operation and collect results
                let stream = (operation)();
                let results: Vec<T> = stream.collect();

                // Check if we got any successful results
                let successful_results: Vec<T> = results
                    .into_iter()
                    .filter(|chunk| !chunk.is_error())
                    .collect();

                if !successful_results.is_empty() {
                    // Success - emit all good results
                    for result in successful_results {
                        emit!(sender, result);
                    }
                    break;
                }

                // No successful results - check if we should retry
                if attempt >= policy.max_attempts {
                    // Exhausted retries - emit error chunk
                    emit!(
                        sender,
                        T::bad_chunk(format!(
                            "Max retry attempts ({}) exceeded",
                            policy.max_attempts
                        ))
                    );
                    break;
                }

                // Calculate delay and wait before retry
                let delay = policy.calculate_delay(attempt);
                std::thread::sleep(delay);
                attempt += 1;
            }
        })
    }
}
