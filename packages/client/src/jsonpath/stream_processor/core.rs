use bytes::Bytes;
use ystream::prelude::MessageChunk;
use ystream::{AsyncStream, AsyncStreamSender, handle_error};
use serde::de::DeserializeOwned;
// http_body::Body import removed - not used

use super::super::{JsonArrayStream, JsonPathError};
use super::types::{ErrorRecoveryState, JsonStreamProcessor, ProcessorStats};
use crate::prelude::*;
use crate::error::constructors::generic;


impl<T> JsonStreamProcessor<T>
where
    T: DeserializeOwned + ystream::prelude::MessageChunk + Default + Send + 'static,
{
    /// Create new `JsonStreamProcessor` with `JSONPath` expression
    #[must_use]
    pub fn new(jsonpath_expr: &str) -> Self {
        Self {
            json_array_stream: JsonArrayStream::new_typed(jsonpath_expr),
            chunk_handlers: Vec::new(),
            stats: ProcessorStats::new(),
            error_recovery: ErrorRecoveryState::new(),
        }
    }

    /// Get current processing statistics
    #[must_use] 
    pub fn stats(&self) -> super::types::ProcessorStatsSnapshot {
        self.stats.snapshot()
    }

    /// Add chunk processing handler for custom transformations
    #[must_use = "Stream processor builder methods return a new processor and should be used"]
    pub fn with_chunk_handler<F>(mut self, handler: F) -> Self
    where
        F: FnMut(Result<T, JsonPathError>) -> Result<T, JsonPathError> + Send + 'static,
    {
        self.chunk_handlers.push(Box::new(handler));
        self
    }

    /// Process HTTP chunks into deserialized objects stream
    pub fn process_chunks<I>(mut self, chunks: I) -> AsyncStream<T, 1024>
    where
        I: Iterator<Item = HttpChunk> + Send + 'static,
        T: MessageChunk + MessageChunk + Default + Send + 'static,
    {
        AsyncStream::with_channel(move |sender: AsyncStreamSender<T>| {
            for chunk in chunks {
                self.stats.update_last_process_time();

                match chunk {
                    HttpChunk::Body(bytes) => {
                        self.stats.record_chunk_processed(bytes.len());

                        match self.process_body_chunk(&sender, bytes) {
                            Ok(()) => {
                                self.record_success();
                            }
                            Err(e) => {
                                self.stats.record_processing_error();
                                let json_error = JsonPathError::new(
                                    crate::jsonpath::error::ErrorKind::Deserialization,
                                    format!("Body chunk processing failed: {e}"),
                                );
                                let http_error =
                                    generic(format!("JSONPath processing error: {json_error}"));

                                if let Err(recovery_error) =
                                    self.handle_error_with_recovery(http_error)
                                {
                                    handle_error!(
                                        recovery_error,
                                        "Failed to process body chunk with recovery"
                                    );
                                }
                            }
                        }
                    }
                    HttpChunk::Error(e) => {
                        self.stats.record_processing_error();
                        if let Err(recovery_error) = self.handle_error_with_recovery(
                            crate::error::Error::new(crate::error::Kind::Request)
                                .with(std::io::Error::other(e)),
                        ) {
                            handle_error!(recovery_error, "HTTP chunk error with recovery");
                        }
                    }
                    _ => {}
                }
            }
        })
    }

    /// Process HTTP response body into streaming objects
    pub fn process_body<B>(mut self, mut body: B) -> AsyncStream<T, 1024>
    where
        B: http_body::Body<Data = Bytes> + Send + Sync + 'static + Unpin,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
        T: MessageChunk + MessageChunk + Default + Send + 'static,
    {
        use std::pin::Pin;
        use std::task::{Context, Poll};
        
        AsyncStream::with_channel(move |sender: AsyncStreamSender<T>| {
            // Process body by polling for data frames
            let mut total_bytes_processed = 0usize;
            
            // Create a simple executor context for polling the body
            let waker = futures::task::noop_waker();
            let mut cx = Context::from_waker(&waker);
            
            // Poll the body for data chunks
            loop {
                // Pin the body for polling
                let pinned_body = Pin::new(&mut body);
                
                match pinned_body.poll_frame(&mut cx) {
                    Poll::Ready(Some(Ok(frame))) => {
                        // Check if this is a data frame
                        if let Some(data) = frame.data_ref() {
                            let chunk_size = data.len();
                            total_bytes_processed += chunk_size;
                            
                            // Record chunk processing
                            self.stats.record_chunk_processed(chunk_size);
                            
                            // Process the chunk
                            if let Err(e) = self.process_body_chunk(&sender, data.clone()) {
                                self.stats.record_processing_error();
                                handle_error!(e, "Body chunk processing failed");
                            } 
                            self.record_success();
                        }
                        // Handle trailers and other frame types as needed
                        // Check if this is the last frame by checking if data is empty and this is a data frame
                        if let Some(data_ref) = frame.data_ref()
                            && data_ref.is_empty() {
                                break;
                            }
                    }
                    Poll::Ready(Some(Err(e))) => {
                        // Body error occurred
                        self.stats.record_processing_error();
                        let error_msg = format!("HTTP body error: {}", e.into());
                        handle_error!(
                            JsonPathError::new(
                                crate::jsonpath::error::ErrorKind::IoError, 
                                error_msg
                            ), 
                            "HTTP body stream error"
                        );
                    }
                    Poll::Ready(None) => {
                        // End of body stream
                        break;
                    }
                    Poll::Pending => {
                        // No data available right now, yield and try again
                        std::thread::yield_now();
                    }
                }
            }
            
            tracing::debug!(
                target: "quyc::jsonpath::stream_processor",
                total_bytes = total_bytes_processed,
                "Completed processing HTTP body stream"
            );
        })
    }

    /// Record successful operation for circuit breaker
    pub(super) fn record_success(&self) {
        self.error_recovery.record_success();
    }
}
