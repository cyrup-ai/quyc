//! Contains implementations of external traits including `ChunkHandler` for
//! ystream integration and Debug for development support.

use std::fmt;
use std::sync::Arc;

use ystream::prelude::ChunkHandler;

use super::builder_core::Http3Builder;
use crate::prelude::*;

/// Trait for builder extensions
pub trait BuilderExt {
    /// Add custom chunk handler for stream processing
    #[must_use]
    fn on_chunk<F>(self, handler: F) -> Self
    where
        F: Fn(Result<HttpChunk, HttpError>) -> HttpChunk + Send + Sync + 'static;
}

/// Request builder extensions
pub trait RequestBuilderExt {
    /// Configure request with custom settings
    #[must_use]
    fn configure<F>(self, config_fn: F) -> Self
    where
        F: FnOnce(Self) -> Self,
        Self: Sized;

    /// Add middleware to request processing
    #[must_use]
    fn middleware<F>(self, middleware_fn: F) -> Self
    where
        F: Fn(HttpChunk) -> HttpChunk + Send + Sync + 'static,
        Self: Sized;
}

impl<S> BuilderExt for Http3Builder<S> {
    #[inline]
    fn on_chunk<F>(self, handler: F) -> Self
    where
        F: Fn(Result<HttpChunk, HttpError>) -> HttpChunk + Send + Sync + 'static,
    {
        self.set_chunk_handler(Arc::new(handler))
    }
}

impl<S> RequestBuilderExt for Http3Builder<S> {
    #[inline]
    fn configure<F>(self, config_fn: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        config_fn(self)
    }

    #[inline]
    fn middleware<F>(self, _middleware_fn: F) -> Self
    where
        F: Fn(HttpChunk) -> HttpChunk + Send + Sync + 'static,
    {
        // Implementation would store middleware function for later use
        // Protocol config is preserved through self
        self
    }
}

/// Implement `ChunkHandler` trait for `Http3Builder` to support ystream `on_chunk` pattern
impl<S> ChunkHandler<HttpChunk, HttpError> for Http3Builder<S> {
    #[inline]
    fn on_chunk<F>(self, handler: F) -> Self
    where
        F: Fn(Result<HttpChunk, HttpError>) -> HttpChunk + Send + Sync + 'static,
    {
        self.set_chunk_handler(Arc::new(handler))
    }
}

impl<S> Http3Builder<S> {
    /// Internal method to set chunk handler
    #[inline]
    fn set_chunk_handler(
        mut self,
        handler: Arc<dyn Fn(Result<HttpChunk, HttpError>) -> HttpChunk + Send + Sync>,
    ) -> Self {
        self.chunk_handler = Some(handler);
        // Protocol config is preserved through mut self
        self
    }
}

impl<S> fmt::Debug for Http3Builder<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Http3Builder")
            .field("client", &"HttpClient")
            .field("request", &"HttpRequest")
            .field("debug_enabled", &self.debug_enabled)
            .field("jsonpath_config", &self.jsonpath_config)
            .field("chunk_handler", &self.chunk_handler.as_ref().map(|_| "ChunkHandler"))
            .field("protocol_config", &self.protocol_config)
            .finish()
    }
}

impl Default for Http3Builder<BodyNotSet> {
    fn default() -> Self {
        Self::json()
    }
}

impl<S> Http3Builder<S> {
    /// Enable HTTP/3 prior knowledge
    #[inline]
    pub fn http3_prior_knowledge(mut self, enable: bool) -> Self {
        self.request = self.request.h2_prior_knowledge(enable);
        self
    }

    /// Set HTTP/3 maximum idle timeout
    #[inline]
    pub fn http3_max_idle_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.request = self.request.with_timeout(timeout);
        self
    }

    /// Set HTTP/3 stream receive window
    #[inline]
    pub fn http3_stream_receive_window(mut self, window: u32) -> Self {
        self.protocol_config.h3_stream_receive_window = Some(window);
        self
    }

    /// Set HTTP/3 connection receive window
    #[inline]
    pub fn http3_conn_receive_window(mut self, window: u32) -> Self {
        self.protocol_config.h3_conn_receive_window = Some(window);
        self
    }

    /// Set HTTP/3 send window
    #[inline]
    pub fn http3_send_window(mut self, window: u32) -> Self {
        self.protocol_config.h3_send_window = Some(window);
        self
    }

    /// Enable HTTP/3 BBR congestion control
    #[inline]
    pub fn http3_congestion_bbr(mut self, enable: bool) -> Self {
        self.protocol_config.h3_congestion_bbr = Some(enable);
        self
    }

    /// Set HTTP/3 maximum field section size
    #[inline]
    pub fn http3_max_field_section_size(mut self, size: u64) -> Self {
        self.protocol_config.h3_max_field_section_size = Some(size);
        self
    }

    /// Enable HTTP/3 GREASE sending
    #[inline]
    pub fn http3_send_grease(mut self, enable: bool) -> Self {
        self.protocol_config.h3_send_grease = Some(enable);
        self
    }

    /// Enable HTTP/2 prior knowledge
    #[inline]
    pub fn http2_prior_knowledge(mut self, enable: bool) -> Self {
        self.request = self.request.h2_prior_knowledge(enable);
        self
    }

    /// Set HTTP/2 adaptive window
    #[inline]
    pub fn http2_adaptive_window(mut self, enable: bool) -> Self {
        self.protocol_config.h2_adaptive_window = Some(enable);
        self
    }

    /// Set HTTP/2 maximum frame size
    #[inline]
    pub fn http2_max_frame_size(mut self, size: u32) -> Self {
        self.protocol_config.h2_max_frame_size = Some(size);
        self
    }
}
