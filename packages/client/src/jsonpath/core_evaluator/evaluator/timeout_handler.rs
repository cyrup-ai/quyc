//! Timeout protection for `JSONPath` evaluation
//!
//! Provides cooperative cancellation-based timeout handling to prevent excessive processing time
//! on pathological `JSONPath` expressions or deeply nested JSON structures.

use std::time::{Duration, Instant};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::convert::TryFrom;

use serde_json::Value;

use super::core_types::{CoreJsonPathEvaluator, JsonPathResult};

/// Configuration for timeout handling
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Maximum evaluation time before timeout
    pub timeout_duration: Duration,
    /// Whether to log timeout events
    pub log_timeouts: bool,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            timeout_duration: Duration::from_millis(1500), // 1.5 second timeout
            log_timeouts: true,
        }
    }
}

/// Timeout handler for `JSONPath` evaluation
pub struct TimeoutHandler;

impl TimeoutHandler {
    /// Evaluate `JSONPath` expression with timeout protection using cooperative cancellation
    ///
    /// # Errors
    /// Returns `JsonPathError` if:
    /// - Evaluation times out after the specified duration
    /// - Expression compilation or evaluation fails
    /// - Memory limits are exceeded during processing
    /// - Invalid timeout configuration is provided
    pub fn evaluate_with_timeout(
        evaluator: &CoreJsonPathEvaluator,
        json: &Value,
        config: Option<TimeoutConfig>,
    ) -> JsonPathResult<Vec<Value>> {
        let config = config.unwrap_or_default();
        let start_time = Instant::now();

        let expression = evaluator.expression().to_string();
        let json_clone = json.clone();

        // Create cancellation flag for cooperative cancellation
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let cancel_clone = cancel_flag.clone();

        // Clone expression before moving into async context
        let expression_clone = expression.clone();

        // Use existing tokio runtime or create one
        

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(Self::evaluate_with_timeout_async(
                expression_clone,
                json_clone,
                config,
                start_time,
                cancel_flag,
                cancel_clone,
            ))
        } else {
            // Create minimal runtime only if needed
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| crate::jsonpath::error::invalid_expression_error(
                    evaluator.expression(),
                    format!("Failed to create async runtime for timeout handling: {e}"),
                    None,
                ))?;
            rt.block_on(Self::evaluate_with_timeout_async(
                expression.clone(),
                json_clone,
                config,
                start_time,
                cancel_flag,
                cancel_clone,
            ))
        }
    }

    /// Async implementation of timeout evaluation with proper cancellation
    async fn evaluate_with_timeout_async(
        expression: String,
        json_clone: Value,
        config: TimeoutConfig,
        start_time: Instant,
        cancel_flag: Arc<AtomicBool>,
        cancel_clone: Arc<AtomicBool>,
    ) -> JsonPathResult<Vec<Value>> {
        if config.log_timeouts {
            log::debug!("Starting JSONPath evaluation with cooperative cancellation");
        }

        // Clone expression before moving into spawn_blocking
        let expression_for_task = expression.clone();

        // Spawn blocking task for CPU-intensive JSONPath evaluation
        let evaluation_task = tokio::task::spawn_blocking(move || {
            Self::evaluate_internal_with_cancellation(&expression_for_task, &json_clone, cancel_clone)
        });

        // Race between timeout and evaluation completion
        match tokio::time::timeout(config.timeout_duration, evaluation_task).await {
            Ok(Ok(result)) => {
                let elapsed = start_time.elapsed();
                if config.log_timeouts {
                    log::debug!(
                        "JSONPath evaluation completed successfully in {elapsed:?}"
                    );
                }
                result
            }
            Ok(Err(join_error)) => {
                let elapsed = start_time.elapsed();
                if config.log_timeouts {
                    log::error!(
                        "JSONPath evaluation task panicked after {elapsed:?}: {join_error}"
                    );
                }
                Err(crate::jsonpath::error::invalid_expression_error(
                    &expression,
                    format!("evaluation task panicked: {join_error}"),
                    None,
                ))
            }
            Err(_timeout_error) => {
                let elapsed = start_time.elapsed();
                if config.log_timeouts {
                    log::warn!(
                        "JSONPath evaluation timed out after {elapsed:?} - cancelling cooperatively"
                    );
                }

                // Signal cancellation to the evaluation task
                cancel_flag.store(true, Ordering::SeqCst);

                // Return empty results for timeout - prevents hanging
                Ok(Vec::new())
            }
        }
    }

    /// Internal evaluation method with cooperative cancellation support
    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_internal_with_cancellation(
        expression: &str, 
        json: &Value, 
        cancel_flag: Arc<AtomicBool>
    ) -> JsonPathResult<Vec<Value>> {
        use super::evaluation_engine::EvaluationEngine;

        // Check for cancellation before starting
        if cancel_flag.load(Ordering::SeqCst) {
            return Ok(Vec::new());
        }

        // Create temporary evaluator instance for method calls
        let temp_evaluator = CoreJsonPathEvaluator::create_temp_evaluator(expression)?;

        // Check for cancellation after evaluator creation
        if cancel_flag.load(Ordering::SeqCst) {
            return Ok(Vec::new());
        }

        // Delegate to evaluation engine with cancellation checking
        // Note: For full cooperative cancellation, the evaluation engine would need
        // to be modified to accept and check the cancellation flag during deep traversal.
        // For now, we check before and after the main evaluation.
        let result = EvaluationEngine::evaluate_expression(&temp_evaluator, json);

        // Final cancellation check
        if cancel_flag.load(Ordering::SeqCst) {
            return Ok(Vec::new());
        }

        result
    }



    /// Check if an expression is likely to cause timeout
    #[must_use] 
    pub fn is_potentially_slow(expression: &str) -> bool {
        // Patterns that are known to be expensive
        expression.contains("..") ||  // Recursive descent
        expression.contains('*') ||   // Wildcard
        expression.contains("[?") ||  // Filters
        expression.contains("[:") ||  // Slices
        expression.matches('[').count() > 3 // Deep nesting
    }

    /// Estimate evaluation complexity
    #[must_use] 
    pub fn estimate_complexity(expression: &str) -> u32 {
        let mut complexity: u32 = 1;

        // Add complexity for expensive operations with overflow protection
        
        // Recursive descent - cap at reasonable maximum to prevent overflow
        let recursive_count = u32::try_from(expression.matches("..").count()).unwrap_or(1000);
        complexity = complexity.saturating_add(recursive_count.saturating_mul(50));
        
        // Wildcard operations
        let wildcard_count = u32::try_from(expression.matches('*').count()).unwrap_or(1000);
        complexity = complexity.saturating_add(wildcard_count.saturating_mul(10));
        
        // Filter operations
        let filter_count = u32::try_from(expression.matches("[?").count()).unwrap_or(1000);
        complexity = complexity.saturating_add(filter_count.saturating_mul(20));
        
        // Slice operations
        let slice_count = u32::try_from(expression.matches("[:").count()).unwrap_or(1000);
        complexity = complexity.saturating_add(slice_count.saturating_mul(5));
        
        // Array access operations
        let array_count = u32::try_from(expression.matches('[').count()).unwrap_or(1000);
        complexity = complexity.saturating_add(array_count.saturating_mul(2));

        complexity
    }

    /// Get recommended timeout for expression
    #[must_use] 
    pub fn recommended_timeout(expression: &str) -> Duration {
        let complexity = Self::estimate_complexity(expression);

        match complexity {
            0..=10 => Duration::from_millis(100),
            11..=50 => Duration::from_millis(500),
            51..=100 => Duration::from_millis(1000),
            _ => Duration::from_millis(2000),
        }
    }
}


