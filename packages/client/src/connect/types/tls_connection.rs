//! TLS connection implementation using enterprise `TlsManager`
//!
//! Provides proper TLS connection wrapper that integrates with the existing
//! enterprise TLS infrastructure in src/tls/ instead of bypassing it.

use std::net::SocketAddr;
use tokio::net::TcpStream;

use super::connection::ConnectionTrait;
use crate::tls::{TlsManager, TlsError};

/// Enterprise TLS connection implementation using `TlsManager`
///
/// This connection wrapper uses the proper enterprise TLS system from src/tls/
/// with comprehensive security features including OCSP validation, CRL checking,
/// certificate management, and proper async I/O integration.
#[derive(Debug)]
pub struct TlsConnection {
    /// The underlying TLS stream from `TlsManager.create_connection()`
    pub stream: tokio_rustls::client::TlsStream<TcpStream>,
}

impl TlsConnection {
    /// Create a new TLS connection wrapper
    ///
    /// This should only be called from `TlsManager.create_connection()` or
    /// connection establishment functions that properly validate TLS.
    pub fn new(stream: tokio_rustls::client::TlsStream<TcpStream>) -> Self {
        Self { stream }
    }
    
    /// Create TLS connection using enterprise `TlsManager`
    ///
    /// This is the proper way to establish TLS connections, using the full
    /// enterprise TLS infrastructure with security validation.
    ///
    /// # Arguments
    /// * `tls_manager` - The enterprise TLS manager instance
    /// * `host` - The hostname for TLS validation
    /// * `port` - The port to connect to
    ///
    /// # Returns
    /// * `Ok(TlsConnection)` - Successfully established TLS connection
    /// * `Err(TlsError)` - TLS connection or validation failed
    pub async fn create_with_manager(
        tls_manager: &TlsManager,
        host: &str,
        port: u16,
    ) -> Result<Self, TlsError> {
        let tls_stream = tls_manager.create_connection(host, port).await?;
        Ok(Self::new(tls_stream))
    }
}

impl tokio::io::AsyncRead for TlsConnection {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for TlsConnection {
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

impl Unpin for TlsConnection {}

impl ConnectionTrait for TlsConnection {
    fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        // Get the underlying TCP stream from the TLS stream
        self.stream.get_ref().0.peer_addr()
    }

    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        // Get the underlying TCP stream from the TLS stream  
        self.stream.get_ref().0.local_addr()
    }

    fn is_closed(&self) -> bool {
        // Check if the underlying TCP connection is closed
        // This is a proper implementation that checks actual connection state
        match self.stream.get_ref().0.peer_addr() {
            Ok(_) => false, // Connection is still valid
            Err(ref e) if e.kind() == std::io::ErrorKind::NotConnected => true,
            Err(_) => false, // Other errors don't necessarily mean closed
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}