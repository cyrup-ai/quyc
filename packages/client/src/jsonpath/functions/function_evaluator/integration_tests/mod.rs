//! Function evaluator utilities
//!
//! This module provides utilities for function evaluation testing and mocking.
//! The integration tests have been moved to the tests/ directory.

pub mod mock_evaluator;

// Re-export test utilities for convenience
pub use mock_evaluator::mock_evaluator;
