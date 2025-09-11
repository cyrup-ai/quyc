//! TCP connection implementations with `MessageChunk` support
//!
//! Provides TCP stream wrappers and connection implementations
//! with comprehensive error handling and `MessageChunk` compliance.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use ystream::prelude::MessageChunk;

use super::connection::ConnectionTrait;

/// Wrapper for `TcpStream` to implement `MessageChunk` safely
#[derive(Debug)]
pub enum TcpStreamWrapper {
    Connected(TcpStream),
    Error(BrokenStream),
}

/// Mock stream that always returns connection errors
#[derive(Debug)]
pub struct BrokenStream {
    error: String,
}

impl Clone for TcpStreamWrapper {
    fn clone(&self) -> Self {
        // Create a new error stream since TcpStream can't be cloned
        TcpStreamWrapper::bad_chunk("Stream cloning not supported".to_string())
    }
}

impl Default for TcpStreamWrapper {
    fn default() -> Self {
        Self::bad_chunk("Default TcpStreamWrapper".to_string())
    }
}

impl BrokenStream {
    #[must_use] 
    pub fn new(error: String) -> Self {
        Self { error }
    }
}

impl Read for BrokenStream {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            self.error.clone(),
        ))
    }
}

impl Write for BrokenStream {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            self.error.clone(),
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            self.error.clone(),
        ))
    }
}

impl TcpStreamWrapper {
    #[must_use] 
    pub fn new(stream: TcpStream) -> Self {
        Self::Connected(stream)
    }
}

impl MessageChunk for TcpStreamWrapper {
    fn bad_chunk(error: String) -> Self {
        // Clean solution: return Error variant with BrokenStream
        Self::Error(BrokenStream::new(error))
    }

    fn is_error(&self) -> bool {
        matches!(self, TcpStreamWrapper::Error(_))
    }

    fn error(&self) -> Option<&str> {
        match self {
            TcpStreamWrapper::Error(broken) => Some(&broken.error),
            TcpStreamWrapper::Connected(_) => None,
        }
    }
}

impl Read for TcpStreamWrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            TcpStreamWrapper::Connected(stream) => stream.read(buf),
            TcpStreamWrapper::Error(broken) => broken.read(buf),
        }
    }
}

impl Write for TcpStreamWrapper {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            TcpStreamWrapper::Connected(stream) => stream.write(buf),
            TcpStreamWrapper::Error(broken) => broken.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            TcpStreamWrapper::Connected(stream) => stream.flush(),
            TcpStreamWrapper::Error(broken) => broken.flush(),
        }
    }
}

/// TCP connection implementation for async I/O (`ConnectionTrait`)
#[derive(Debug)]
pub struct TcpConnection {
    pub stream: tokio::net::TcpStream,
}

impl TcpConnection {
    pub fn new(stream: tokio::net::TcpStream) -> Self {
        Self { stream }
    }
}

impl tokio::io::AsyncRead for TcpConnection {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for TcpConnection {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl Unpin for TcpConnection {}

impl ConnectionTrait for TcpConnection {
    fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        self.stream.peer_addr()
    }

    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.stream.local_addr()
    }

    fn is_closed(&self) -> bool {
        // Check if the TCP connection is actually closed by attempting to get peer address
        match self.stream.peer_addr() {
            Err(ref e) if e.kind() == std::io::ErrorKind::NotConnected => true,
            _ => false, // Connection valid or other errors don't necessarily mean closed
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TryFrom<TcpStream> for TcpConnection {
    type Error = std::io::Error;
    
    fn try_from(stream: TcpStream) -> Result<Self, Self::Error> {
        // Convert std::net::TcpStream to tokio::net::TcpStream
        let tokio_stream = tokio::net::TcpStream::from_std(stream)?;
        Ok(Self::new(tokio_stream))
    }
}


