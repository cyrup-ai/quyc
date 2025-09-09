//! Pure streams-first execution - NO Futures, NO Result wrapping
//! All operations return unwrapped AsyncStreams per fluent-ai architecture

use ystream::{AsyncStream, prelude::MessageChunk};
use serde::de::DeserializeOwned;

/// Execution context for HTTP requests
pub struct ExecutionContext {
    pub timeout_ms: Option<u64>,
    pub retry_attempts: Option<u32>,
    pub debug_enabled: bool,
}

/// Request execution configuration
pub struct RequestExecution {
    pub context: ExecutionContext,
    pub stream_buffer_size: usize,
    pub chunk_size: usize,
}

impl Default for ExecutionContext {
    #[inline]
    fn default() -> Self {
        Self {
            timeout_ms: None,
            retry_attempts: None,
            debug_enabled: false,
        }
    }
}

impl Default for RequestExecution {
    #[inline]
    fn default() -> Self {
        Self {
            context: ExecutionContext::default(),
            stream_buffer_size: 8192,
            chunk_size: 4096,
        }
    }
}

/// Pure streams extension trait - NO Result wrapping, streams-only
///
/// Provides streaming methods that return AsyncStreams of deserialized types
/// following the streams-first architecture mandate
pub trait HttpStreamExt<T> {
    /// Stream deserialized objects as they arrive - pure streaming
    ///
    /// Returns unwrapped AsyncStream of deserialized objects, not Result-wrapped
    /// Errors are emitted as stream items, not exceptions
    fn stream_objects(self) -> AsyncStream<T, 1024>;

    /// Collect all items into Vec - blocks until complete (streams-first bridge)
    fn collect_all(self) -> Vec<T>;

    /// Get first item only - blocks until available
    fn first_item(self) -> Option<T>;
}

/// HTTP stream type alias for ystream streaming
pub type HttpStream<T> = AsyncStream<T, 1024>;

impl<T> HttpStreamExt<T> for HttpStream<T>
where
    T: DeserializeOwned + MessageChunk + Send + Default + Clone + 'static,
{
    #[inline]
    fn stream_objects(self) -> AsyncStream<T, 1024> {
        self
    }

    #[inline]
    fn collect_all(self) -> Vec<T> {
        self.collect()
    }

    #[inline]
    fn first_item(self) -> Option<T> {
        self.try_next()
    }
}
