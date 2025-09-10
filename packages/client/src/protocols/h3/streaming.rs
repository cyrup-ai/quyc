use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

use ystream::{AsyncStream, emit};
use h3::client::Connection;
use h3::quic::{BufRecvStream, RecvStream, SendStream};

use crate::protocols::h3::h3_chunks::{H3BiStreamChunk, H3ConnectionChunk, H3DataChunk, H3SendResult};

static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct H3Connection<T> {
    connection: Connection<T>,
    connection_id: u64,
}

impl<T> H3Connection<T>
where
    T: h3::quic::Connection<BufRecvStream>,
{
    #[inline]
    pub fn new(connection: Connection<T>) -> Self {
        let connection_id = CONNECTION_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self {
            connection,
            connection_id,
        }
    }

    #[inline]
    pub fn connection_id(&self) -> u64 {
        self.connection_id
    }

    pub fn accept_recv_stream(
        mut self,
        mut context: Context<'_>,
    ) -> AsyncStream<H3ConnectionChunk, 1024> {
        AsyncStream::<H3ConnectionChunk, 1024>::with_channel(move |sender| {
            let mut connection = self.connection;

            // Use h3's direct poll_accept_recv primitive - NO Futures
            loop {
                match connection.poll_accept_recv(&mut context) {
                    Poll::Ready(Ok(recv_stream)) => {
                        emit!(
                            sender,
                            H3ConnectionChunk::new_recv_stream(recv_stream.stream_id())
                        );
                    }
                    Poll::Ready(Err(e)) => {
                        emit!(
                            sender,
                            H3ConnectionChunk::bad_chunk(format!("Accept error: {e}"))
                        );
                        break;
                    }
                    Poll::Pending => break, // AsyncStream elite polling loop handles this
                }
            }
        })
    }

    pub fn accept_bidi_stream(
        mut self,
        mut context: Context<'_>,
    ) -> AsyncStream<H3BiStreamChunk, 1024> {
        AsyncStream::<H3BiStreamChunk, 1024>::with_channel(move |sender| {
            let mut connection = self.connection;

            // Use h3's direct poll_accept_bidi primitive - NO Futures
            loop {
                match connection.poll_accept_bidi(&mut context) {
                    Poll::Ready(Ok((send_stream, recv_stream))) => {
                        emit!(
                            sender,
                            H3BiStreamChunk::new_bidi_stream(
                                send_stream.stream_id(),
                                recv_stream.stream_id()
                            )
                        );
                    }
                    Poll::Ready(Err(e)) => {
                        emit!(
                            sender,
                            H3BiStreamChunk::bad_chunk(format!("Accept bidi error: {e}"))
                        );
                        break;
                    }
                    Poll::Pending => break, // AsyncStream elite polling loop handles this
                }
            }
        })
    }
}

pub struct H3RecvStreamHandler {
    stream_id: u64,
}

impl H3RecvStreamHandler {
    #[inline]
    pub fn new(stream_id: u64) -> Self {
        Self { stream_id }
    }

    pub fn poll_data_stream<T>(
        &self,
        mut buf_recv_stream: BufRecvStream<T>,
        mut context: Context<'_>,
    ) -> AsyncStream<H3DataChunk, 1024>
    where
        T: h3::quic::RecvStream,
    {
        AsyncStream::<H3DataChunk, 1024>::with_channel(move |sender| {
            // Use h3's direct poll_data primitive - NO Futures
            loop {
                match buf_recv_stream.poll_data(&mut context) {
                    Poll::Ready(Ok(Some(data))) => {
                        emit!(sender, H3DataChunk::from_bytes(data));
                    }
                    Poll::Ready(Ok(None)) => {
                        emit!(sender, H3DataChunk::stream_complete());
                        break;
                    }
                    Poll::Ready(Err(e)) => {
                        emit!(sender, H3DataChunk::bad_chunk(format!("Data error: {e}")));
                        break;
                    }
                    Poll::Pending => break, // AsyncStream elite polling loop handles this
                }
            }
        })
    }
}

pub struct H3SendStreamHandler {
    stream_id: u64,
}

impl H3SendStreamHandler {
    #[inline]
    pub fn new(stream_id: u64) -> Self {
        Self { stream_id }
    }

    pub fn send_data_stream<T>(
        &self,
        mut send_stream: SendStream<T>,
        data_chunks: Vec<bytes::Bytes>,
        mut context: Context<'_>,
    ) -> AsyncStream<H3SendResult, 1024>
    where
        T: h3::quic::SendStream,
    {
        AsyncStream::<H3SendResult, 1024>::with_channel(move |sender| {
            // Use h3's direct poll_ready and send_data primitives - NO Futures
            for data_chunk in data_chunks {
                match send_stream.poll_ready(&mut context) {
                    Poll::Ready(Ok(())) => match send_stream.send_data(data_chunk) {
                        Ok(()) => emit!(sender, H3SendResult::data_sent()),
                        Err(e) => {
                            emit!(
                                sender,
                                H3SendResult::bad_chunk(format!("Send error: {e}"))
                            );
                            return;
                        }
                    },
                    Poll::Ready(Err(e)) => {
                        emit!(
                            sender,
                            H3SendResult::bad_chunk(format!("Ready error: {e}"))
                        );
                        return;
                    }
                    Poll::Pending => break, // AsyncStream elite polling loop handles this
                }
            }

            // Finish stream using direct poll_finish primitive
            match send_stream.poll_finish(&mut context) {
                Poll::Ready(Ok(())) => emit!(sender, H3SendResult::send_complete()),
                Poll::Ready(Err(e)) => emit!(
                    sender,
                    H3SendResult::bad_chunk(format!("Finish error: {e}"))
                ),
                Poll::Pending => {} // AsyncStream elite polling loop handles this
            }
        })
    }

    pub fn poll_ready_stream<T>(
        &self,
        mut send_stream: SendStream<T>,
        mut context: Context<'_>,
    ) -> AsyncStream<H3SendResult, 1024>
    where
        T: h3::quic::SendStream,
    {
        let stream_id = self.stream_id;

        AsyncStream::<H3SendResult, 1024>::with_channel(move |sender| {
            match send_stream.poll_ready(&mut context) {
                Poll::Ready(Ok(())) => {
                    emit!(sender, H3SendResult::send_ready(stream_id));
                }
                Poll::Ready(Err(e)) => {
                    emit!(
                        sender,
                        H3SendResult::bad_chunk(format!("Poll ready error: {e}"))
                    );
                }
                Poll::Pending => {} // AsyncStream elite polling loop handles this
            }
        })
    }
}

pub struct H3ConnectionPool<T> {
    connections: Vec<Arc<H3Connection<T>>>,
    next_connection: AtomicU64,
}

impl<T> H3ConnectionPool<T>
where
    T: h3::quic::Connection<BufRecvStream>,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            next_connection: AtomicU64::new(0),
        }
    }

    #[inline]
    pub fn add_connection(&mut self, connection: H3Connection<T>) {
        self.connections.push(Arc::new(connection));
    }

    #[inline]
    pub fn get_connection(&self) -> Option<Arc<H3Connection<T>>> {
        if self.connections.is_empty() {
            return None;
        }

        let index =
            self.next_connection.fetch_add(1, Ordering::Relaxed) as usize % self.connections.len();
        self.connections.get(index).cloned()
    }

    #[inline]
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

impl<T> Default for H3ConnectionPool<T>
where
    T: h3::quic::Connection<BufRecvStream>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
