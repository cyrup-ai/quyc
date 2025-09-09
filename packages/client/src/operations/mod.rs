//! HTTP Operation Modules
//!
//! This module provides focused implementations for each HTTP verb (GET, POST, PUT, DELETE, PATCH, DOWNLOAD)
//! with proper separation of concerns and blazing-fast performance.

use http::Method;

// Re-exports for operation modules
pub mod delete;
pub mod download;
pub mod get;
pub mod patch;
pub mod post;
pub mod put;

/// Base trait for all HTTP operations providing common functionality.
pub trait HttpOperation {
    /// The type of stream this operation produces.
    type Output;

    /// Execute the HTTP operation and return a stream of results.
    fn execute(&self) -> Self::Output;

    /// Get the HTTP method for this operation.
    fn method(&self) -> Method;

    /// Get the target URL for this operation.
    fn url(&self) -> &str;
}
