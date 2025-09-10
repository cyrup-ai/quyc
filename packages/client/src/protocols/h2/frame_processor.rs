use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};

use bytes::Bytes;
use crossbeam::queue::ArrayQueue;
use h2::RecvStream;

/// Synchronous frame processor for h2::RecvStream
/// Provides non-blocking access to HTTP/2 data frames without Future-based APIs
pub struct H2FrameProcessor {
    recv_stream: Option<RecvStream>,
    data_chunks: Arc<ArrayQueue<Bytes>>,
    is_closed: AtomicBool,
}

impl H2FrameProcessor {
    pub fn new(recv_stream: RecvStream) -> Self {
        Self {
            recv_stream: Some(recv_stream),
            data_chunks: Arc::new(ArrayQueue::new(1024)),
            is_closed: AtomicBool::new(false),
        }
    }

    /// Try to receive a data chunk without blocking
    /// Returns None if no chunks are available or stream is closed
    pub fn try_recv_data_chunk(&self) -> Option<Bytes> {
        if self.is_closed.load(Ordering::Acquire) {
            return None;
        }

        self.data_chunks.pop()
    }

    /// Start background processing of the h2 recv stream
    /// This spawns a task that polls the stream and captures data chunks
    pub fn start_processing(&mut self) {
        if let Some(recv_stream) = self.recv_stream.take() {
            let data_chunks = self.data_chunks.clone();
            let is_closed = Arc::new(self.is_closed.clone());

            // Use ystream spawn_task pattern
            ystream::spawn_task(move || {
                Self::process_stream_sync(recv_stream, data_chunks, is_closed)
            });
        }
    }

    /// Synchronous stream processing using no-op waker
    fn process_stream_sync(
        mut recv_stream: RecvStream,
        data_chunks: Arc<ArrayQueue<Bytes>>,
        is_closed: Arc<AtomicBool>,
    ) -> Result<(), h2::Error> {
        // Create a no-op waker for sync polling
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        loop {
            match Pin::new(&mut recv_stream).poll_data(&mut cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    // Got data chunk, store it in queue
                    if data_chunks.push(chunk.clone()).is_err() {
                        // Queue is full, drop oldest chunk
                        let _ = data_chunks.pop();
                        let _ = data_chunks.push(chunk);
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    // Stream error
                    is_closed.store(true, Ordering::Release);
                    return Err(e);
                }
                Poll::Ready(None) => {
                    // Stream ended
                    is_closed.store(true, Ordering::Release);
                    break;
                }
                Poll::Pending => {
                    // No data available right now, yield thread
                    std::thread::yield_now();
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Check if the connection is closed
    pub fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Acquire)
    }
}

/// Create a no-op waker for synchronous polling using safe futures API
fn noop_waker() -> &'static Waker {
    use futures::task::noop_waker;
    
    static WAKER: std::sync::OnceLock<Waker> = std::sync::OnceLock::new();
    WAKER.get_or_init(|| noop_waker())
}
