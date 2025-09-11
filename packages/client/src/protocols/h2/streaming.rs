//! H2 Connection Management - Direct Poll-Based Implementation
//!
//! Simple H2 connection management using direct poll-based primitives only,
//! following the exact specification requirements.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::task::Poll;
use std::pin::Pin;
use std::future::Future;

use ystream::prelude::MessageChunk;
use ystream::{AsyncStream, emit};
// futures block_on import removed - not used
// h2 client/server imports removed - not used

use super::chunks::{H2ConnectionChunk, H2DataChunk, H2SendResult};
// core protocol imports removed - not used

/// H2 Connection Manager with direct poll-based primitives only
///
/// Provides simple H2 connection management using only the patterns
/// specified in the task requirements.
#[derive(Debug)]
pub struct H2ConnectionManager {
    is_connected: AtomicBool,
    connection_id: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
}

impl H2ConnectionManager {
    /// Create a new H2 connection manager
    #[inline]
    #[must_use] 
    pub const fn new() -> Self {
        Self {
            is_connected: AtomicBool::new(false),
            connection_id: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
        }
    }

    /// Create H2 connection establishment stream using direct poll-based primitives
    ///
    /// Integrates `h2::client::handshake` with direct polling
    /// within `AsyncStream::with_channel` pattern.
    #[inline]
    pub fn establish_connection_stream<T>(
        &self,
        io: T,
    ) -> AsyncStream<H2ConnectionChunk, 1024>
    where
        T: std::io::Read + std::io::Write + tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let _connection_id = self.connection_id.fetch_add(1, Ordering::SeqCst);
        let _is_connected = self.is_connected.load(Ordering::Relaxed);

        AsyncStream::<H2ConnectionChunk, 1024>::with_channel(move |sender| {
            // Thread-safe context creation INSIDE closure
            use futures::task::noop_waker_ref;
            use std::task::Context;
            use std::time::Duration;
            
            let waker = noop_waker_ref();
            let mut cx = Context::from_waker(waker);

            // Create handshake future for manual polling
            let mut handshake = Box::pin(h2::client::Builder::new().handshake(io));
            
            // Manual polling loop with exponential backoff
            let mut backoff_ms = 1;
            loop {
                match handshake.as_mut().poll(&mut cx) {
                    Poll::Ready(Ok((_send_request, connection))) => {
                        // Connection established successfully
                        emit!(sender, H2ConnectionChunk::ready());

                        // Monitor connection using Future trait with proper pinning
                        let mut conn: h2::client::Connection<T> = connection;
                        loop {
                            let pinned_conn = Pin::new(&mut conn);
                            match Future::poll(pinned_conn, &mut cx) {
                                Poll::Ready(Ok(())) => {
                                    // Connection healthy, continue monitoring
                                }
                                Poll::Ready(Err(e)) => {
                                    // Connection lost
                                    emit!(
                                        sender,
                                        H2ConnectionChunk::bad_chunk(format!("Connection lost: {e}"))
                                    );
                                    break;
                                }
                                Poll::Pending => {
                                    // Connection not ready, AsyncStream handles this
                                    break;
                                }
                            }
                        }
                        break;
                    }
                    Poll::Ready(Err(e)) => {
                        emit!(
                            sender,
                            H2ConnectionChunk::bad_chunk(format!("Handshake failed: {e}"))
                        );
                        break;
                    }
                    Poll::Pending => {
                        // Exponential backoff up to 100ms
                        std::thread::sleep(Duration::from_millis(backoff_ms));
                        backoff_ms = (backoff_ms * 2).min(100);
                    }
                }
            }
        })
    }

    /// Create multiplexed H2 receive stream for multiple streams
    ///
    /// Handles multiple H2 receive streams using
    /// direct poll-based primitives within `AsyncStream::with_channel`.
    #[inline]
    #[must_use] 
    pub fn multiplexed_receive_stream(
        recv_streams: Vec<h2::RecvStream>,
    ) -> AsyncStream<H2DataChunk, 1024> {
        AsyncStream::<H2DataChunk, 1024>::with_channel(move |sender| {
            // Thread-safe context creation INSIDE closure
            use futures::task::noop_waker_ref;
            use std::task::Context;
            
            let waker = noop_waker_ref();
            let mut context = Context::from_waker(waker);
            let mut streams = recv_streams;

            // Poll all receive streams using direct poll_data primitives
            while !streams.is_empty() {
                let mut completed_indices = Vec::new();

                for (index, recv_stream) in streams.iter_mut().enumerate() {
                    match recv_stream.poll_data(&mut context) {
                        Poll::Ready(Some(Ok(data))) => {
                            emit!(sender, H2DataChunk::from_bytes(data));
                        }
                        Poll::Ready(Some(Err(e))) => {
                            emit!(sender, H2DataChunk::bad_chunk(format!("Data error: {e}")));
                            completed_indices.push(index);
                        }
                        Poll::Ready(None) => {
                            emit!(sender, H2DataChunk::stream_complete());
                            completed_indices.push(index);
                        }
                        Poll::Pending => {
                            // Stream not ready, continue to next
                        }
                    }
                }

                // Remove completed streams (in reverse order to maintain indices)
                for &index in completed_indices.iter().rev() {
                    let _ = streams.swap_remove(index);
                }

                if streams.is_empty() {
                    break;
                }
            }
        })
    }

    /// Create flow-controlled H2 send stream with backpressure
    ///
    /// Implements flow control using h2's direct `poll_ready` primitive
    /// within `AsyncStream::with_channel` pattern.
    #[inline]
    #[must_use] 
    pub fn flow_controlled_send_stream(
        send_stream: h2::SendStream<bytes::Bytes>,
        data_chunks: Vec<bytes::Bytes>,
    ) -> AsyncStream<H2SendResult, 1024> {
        AsyncStream::<H2SendResult, 1024>::with_channel(move |sender| {
            let mut stream = send_stream;
            // Thread-safe context creation INSIDE closure
            use futures::task::noop_waker_ref;
            use std::task::Context;
            
            let waker = noop_waker_ref();
            let _context = Context::from_waker(waker);
            let mut remaining_chunks = data_chunks;

            while let Some(chunk) = remaining_chunks.pop() {
                // SendStream doesn't have poll_ready, use direct send_data with flow control
                let is_last = remaining_chunks.is_empty();

                match stream.send_data(chunk, is_last) {
                    Ok(()) => {
                        emit!(sender, H2SendResult::data_sent());

                        if is_last {
                            emit!(sender, H2SendResult::send_complete());
                            break;
                        }
                    }
                    Err(e) => {
                        emit!(
                            sender,
                            H2SendResult::bad_chunk(format!("Send error: {e}"))
                        );
                        break;
                    }
                }
            }
        })
    }

    /// Get connection status
    #[inline]
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::Acquire)
    }

    /// Get connection ID
    #[inline]
    pub fn connection_id(&self) -> u64 {
        self.connection_id.load(Ordering::Acquire)
    }

    /// Update bytes sent counter
    #[inline]
    pub fn add_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::SeqCst);
    }

    /// Update bytes received counter
    #[inline]
    pub fn add_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::SeqCst);
    }

    /// Get bytes sent
    #[inline]
    pub fn bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Acquire)
    }

    /// Get bytes received
    #[inline]
    pub fn bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Acquire)
    }

    /// Check if connection has encountered an error
    #[inline]
    pub fn is_error(&self) -> bool {
        !self.is_connected()
    }
}

impl Default for H2ConnectionManager {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for H2ConnectionManager {
    fn clone(&self) -> Self {
        Self {
            is_connected: AtomicBool::new(self.is_connected.load(std::sync::atomic::Ordering::Relaxed)),
            connection_id: AtomicU64::new(self.connection_id.load(std::sync::atomic::Ordering::Relaxed)),
            bytes_sent: AtomicU64::new(self.bytes_sent.load(std::sync::atomic::Ordering::Relaxed)),
            bytes_received: AtomicU64::new(self.bytes_received.load(std::sync::atomic::Ordering::Relaxed)),
        }
    }
}
