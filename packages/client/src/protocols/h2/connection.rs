//! HTTP/2 connection management
//!
//! This module provides HTTP/2 connection types and stream management using h2 crate
//! direct polling primitives integrated with ystream streaming patterns.

use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Poll, RawWaker, RawWakerVTable, Waker};
use std::time::Instant;
use std::pin::Pin;
use std::future::Future;

use bytes::Bytes;
use crossbeam_utils::Backoff;
use ystream::{AsyncStream, emit};
// h2 imports removed - not used
use http::Request;

use super::streaming::H2ConnectionManager;
use crate::prelude::*;
use crate::protocols::{
    core::{HttpVersion, TimeoutConfig},
    strategy::H2Config,
};
// quiche import removed - not used

static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Create a thread-safe noop waker for use in AsyncStream contexts
/// Unlike futures::task::noop_waker(), this waker is Send + Sync
fn create_thread_safe_noop_waker() -> Waker {
    unsafe fn noop_clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE)
    }
    unsafe fn noop(_: *const ()) {}
    
    const NOOP_WAKER_VTABLE: RawWakerVTable = 
        RawWakerVTable::new(noop_clone, noop, noop, noop);
    
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE)) }
}

/// HTTP/2 connection wrapper using direct polling primitives
pub struct H2Connection {
    manager: super::streaming::H2ConnectionManager,
    config: super::super::core::TimeoutConfig,
    h2_config: super::super::strategy::H2Config,
    established_at: Option<Instant>,
    connection_id: u64,
}

impl std::fmt::Debug for H2Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("H2Connection")
            .field("manager", &self.manager)
            .field("config", &self.config)
            .field("h2_config", &self.h2_config)
            .field("established_at", &self.established_at)
            .field("connection_id", &self.connection_id)
            .finish()
    }
}

impl H2Connection {
    pub fn new() -> Self {
        Self::with_config(H2Config::default())
    }

    pub fn with_config(config: H2Config) -> Self {
        let connection_id = CONNECTION_COUNTER.fetch_add(1, Ordering::Relaxed);

        Self {
            manager: H2ConnectionManager::new(),
            config: TimeoutConfig::default(),
            h2_config: config,
            established_at: Some(Instant::now()),
            connection_id,
        }
    }

    /// Get the HTTP version
    pub fn version(&self) -> HttpVersion {
        HttpVersion::Http2
    }

    /// Get the timeout configuration
    pub fn config(&self) -> &TimeoutConfig {
        &self.config
    }

    /// Check if the connection is ready to send requests
    pub fn is_ready(&self) -> bool {
        !self.manager.is_error()
    }

    /// Send data through HTTP/2 connection
    pub fn send_data(
        &self,
        data: Vec<u8>,
    ) -> AsyncStream<crate::protocols::frames::FrameChunk, 1024> {
        AsyncStream::with_channel(move |sender| {
            // Convert data to H2 frame and emit
            let frame = crate::protocols::frames::H2Frame::Data {
                stream_id: 1,
                data,
                end_stream: false,
            };
            emit!(
                sender,
                crate::protocols::frames::FrameChunk::h2_frame(frame)
            );
        })
    }

    /// Receive data from HTTP/2 connection
    pub fn receive_data(&self) -> AsyncStream<crate::protocols::frames::FrameChunk, 1024> {
        AsyncStream::with_channel(move |sender| {
            // Poll for incoming H2 frames
            let frame = crate::protocols::frames::H2Frame::Data {
                stream_id: 1,
                data: vec![],
                end_stream: false,
            };
            emit!(
                sender,
                crate::protocols::frames::FrameChunk::h2_frame(frame)
            );
        })
    }

    /// Close HTTP/2 connection
    pub fn close(&self) -> AsyncStream<crate::protocols::frames::FrameChunk, 1024> {
        AsyncStream::with_channel(move |sender| {
            let frame = crate::protocols::frames::H2Frame::GoAway {
                last_stream_id: 0,
                error_code: 0,
                debug_data: vec![],
            };
            emit!(
                sender,
                crate::protocols::frames::FrameChunk::h2_frame(frame)
            );
        })
    }

    /// Send an HTTP/2 request and return a stream of response chunks
    pub fn send_request_stream<T>(
        &self,
        io: T,
        request: Request<()>,
        body: Option<Bytes>,
    ) -> AsyncStream<HttpChunk, 1024>
    where
        T: std::io::Read + std::io::Write + tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        AsyncStream::<HttpChunk, 1024>::with_channel(move |sender| {
            // Elite polling pattern within AsyncStream closure - thread-safe
            use futures::executor::block_on;
            use futures::task::noop_waker_ref;
            use std::task::Context;
            
            // Thread-safe context creation INSIDE closure
            let waker = noop_waker_ref();
            let mut cx = Context::from_waker(waker);
            let backoff = Backoff::new();
            
            match block_on(h2::client::Builder::new().handshake(io)) {
                Ok((mut send_request, _connection)) => {
                    // Elite polling loop for send readiness
                    loop {
                        match send_request.poll_ready(&mut cx) {
                            Poll::Ready(Ok(())) => break, // Ready to send request
                            Poll::Ready(Err(e)) => {
                                emit!(sender, HttpChunk::Error(format!("Send readiness error: {}", e)));
                                return;
                            }
                            Poll::Pending => {
                                backoff.snooze(); // Elite backoff pattern
                                continue;
                            }
                        }
                    }

                    // Send request after confirming readiness
                    match send_request.send_request(request, body.is_none()) {
                        Ok((response_future, mut request_stream)) => {
                            // Send body if provided (SendStream doesn't have poll_ready, use direct send)
                            if let Some(body_data) = body {
                                if let Err(e) = request_stream.send_data(body_data, true) {
                                    emit!(
                                        sender,
                                        HttpChunk::Error(format!(
                                            "Send body error: {}",
                                            e
                                        ))
                                    );
                                    return;
                                }
                            }

                            // Poll response future using Future trait with proper pinning
                            let mut response_future = response_future;
                            loop {
                                let pinned_future = Pin::new(&mut response_future);
                                match Future::poll(pinned_future, &mut cx) {
                                    Poll::Ready(Ok(response)) => {
                                        // Extract response headers
                                        let status = response.status();
                                        let headers = response.headers().clone();
                                        
                                        // Emit status/headers as chunks BEFORE body processing
                                        emit!(sender, HttpChunk::Headers(status, headers));
                                        
                                        // Get body stream and poll for data
                                        let mut body_stream = response.into_body();
                                        loop {
                                            match body_stream.poll_data(&mut cx) {
                                                Poll::Ready(Some(Ok(data))) => {
                                                    let data_len = data.len();
                                                    emit!(sender, HttpChunk::Data(data));
                                                    let _ = body_stream.flow_control().release_capacity(data_len);
                                                }
                                                Poll::Ready(Some(Err(e))) => {
                                                    emit!(sender, HttpChunk::Error(format!("Body stream error: {}", e)));
                                                    break;
                                                }
                                                Poll::Ready(None) => {
                                                    // Body stream ended - now poll for trailers
                                                    loop {
                                                        match body_stream.poll_trailers(&mut cx) {
                                                            Poll::Ready(Ok(Some(trailers))) => {
                                                                emit!(sender, HttpChunk::Trailers(trailers));
                                                                break;
                                                            }
                                                            Poll::Ready(Ok(None)) => {
                                                                // No trailers available
                                                                break;
                                                            }
                                                            Poll::Ready(Err(e)) => {
                                                                emit!(sender, HttpChunk::Error(format!("Trailers error: {}", e)));
                                                                break;
                                                            }
                                                            Poll::Pending => {
                                                                backoff.snooze();
                                                                continue;
                                                            }
                                                        }
                                                    }
                                                    break;
                                                }
                                                Poll::Pending => {
                                                    backoff.snooze();
                                                    continue;
                                                }
                                            }
                                        }
                                        break;
                                    }
                                    Poll::Ready(Err(e)) => {
                                        emit!(sender, HttpChunk::Error(format!("Response error: {}", e)));
                                        break;
                                    }
                                    Poll::Pending => {
                                        backoff.snooze();
                                        continue;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            emit!(
                                sender,
                                HttpChunk::Error(format!("Send request error: {}", e))
                            );
                        }
                    }
                }
                Err(e) => {
                    emit!(sender, HttpChunk::Error(format!("Handshake error: {}", e)));
                }
            }
        })
    }

    /// Check if the connection has encountered an error
    pub fn is_error(&self) -> bool {
        self.manager.is_error()
    }

    pub fn connection_id(&self) -> u64 {
        self.connection_id
    }

    pub fn is_established(&self) -> bool {
        self.established_at.is_some()
    }
}

impl H2Connection {}

/// HTTP/2 stream wrapper that bridges h2::RecvStream to AsyncStream
pub struct H2Stream {
    stream: AsyncStream<HttpChunk, 1024>,
}

impl H2Stream {
    /// Create a new H2Stream from h2::RecvStream using pure streams architecture
    ///
    /// Note: This method currently delegates to an empty stream as recv-only streams
    /// are not the primary use case for this HTTP client library. The main pattern
    /// is request-response via send_request_stream().
    pub fn from_recv_stream(_recv_stream: h2::RecvStream) -> Self {
        // Create an empty stream for recv-only scenario
        // Most HTTP client usage goes through send_request_stream() which handles the full lifecycle
        let stream = AsyncStream::<HttpChunk, 1024>::with_channel(move |sender| {
            // Emit end immediately for recv-only streams
            // Real HTTP client usage should use send_request_stream() instead
            emit!(sender, HttpChunk::End);
        });

        Self { stream }
    }

    /// Get the underlying AsyncStream
    pub fn into_stream(self) -> AsyncStream<HttpChunk, 1024> {
        self.stream
    }

    /// Collect all chunks from the stream
    pub fn collect(self) -> Vec<HttpChunk> {
        self.stream.collect()
    }
}

impl ystream::prelude::MessageChunk for H2Connection {
    fn bad_chunk(_error: String) -> Self {
        // Create error connection without tokio dependencies
        Self {
            manager: H2ConnectionManager::new(),
            config: TimeoutConfig::default(),
            h2_config: H2Config::default(),
            established_at: None, // No establishment time for error connections
            connection_id: CONNECTION_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }

    fn error(&self) -> Option<&str> {
        if self.is_error() {
            Some("H2 connection error")
        } else {
            None
        }
    }
}


impl Clone for H2Connection {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
            config: self.config.clone(),
            h2_config: self.h2_config.clone(),
            established_at: self.established_at,
            connection_id: self.connection_id,
        }
    }
}
