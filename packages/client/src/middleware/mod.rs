//! HTTP middleware for request/response processing
//! Simplified, streaming-first processing aligned with `quyc`'s zero-allocation design

#![allow(dead_code)]

use std::sync::Arc;

use crate::prelude::*;

/// HTTP middleware trait for quyc
pub trait Middleware: Send + Sync {
    /// Process request before sending - returns Result directly
    fn process_request(&self, request: HttpRequest) -> crate::error::Result<HttpRequest> {
        Ok(request)
    }

    /// Process response after receiving - returns Result directly  
    fn process_response(&self, response: HttpResponse) -> crate::error::Result<HttpResponse> {
        Ok(response)
    }

    /// Handle errors - returns Result directly
    fn handle_error(&self, error: HttpError) -> crate::error::Result<HttpError> {
        Ok(error)
    }
}

/// Middleware chain for sequential processing
#[derive(Default)]
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn add<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }
}

/// Cache middleware module
pub mod cache;
pub use cache::CacheMiddleware;
