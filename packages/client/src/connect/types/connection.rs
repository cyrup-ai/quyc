//! HTTP connection abstractions and trait definitions
//!
//! Provides the core connection types with `MessageChunk` implementations
//! for error handling and connection management.


use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite};

use ystream::prelude::MessageChunk;

/// HTTP connection wrapper that abstracts different connection types.
pub struct Conn {
    pub(super) inner: Box<dyn ConnectionTrait + Send + Sync>,
    pub(super) is_proxy: bool,
    pub(super) tls_info: Option<TlsInfo>,
}

impl std::fmt::Debug for Conn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Conn")
            .field("is_proxy", &self.is_proxy)
            .field("tls_info", &self.tls_info)
            .finish()
    }
}

impl MessageChunk for Conn {
    fn bad_chunk(error: String) -> Self {
        // Create a broken connection that will fail on any operation
        let broken_conn = Box::new(BrokenConnectionImpl::new(error));

        Conn {
            inner: broken_conn,
            is_proxy: false,
            tls_info: None,
        }
    }

    fn is_error(&self) -> bool {
        // Check if this is a broken connection by checking if it's closed
        self.inner.is_closed()
    }

    fn error(&self) -> Option<&str> {
        if let Some(broken) = self.inner.as_any().downcast_ref::<BrokenConnectionImpl>() {
            Some(&broken.error_message)
        } else {
            None
        }
    }
}

impl Default for Conn {
    fn default() -> Self {
        #[derive(Debug)]
        struct NullConnection;

        impl Unpin for NullConnection {}

        impl AsyncRead for NullConnection {
            fn poll_read(
                self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
                _buf: &mut tokio::io::ReadBuf<'_>,
            ) -> std::task::Poll<std::io::Result<()>> {
                std::task::Poll::Ready(Ok(())) // EOF
            }
        }

        impl AsyncWrite for NullConnection {
            fn poll_write(
                self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
                buf: &[u8],
            ) -> std::task::Poll<Result<usize, std::io::Error>> {
                std::task::Poll::Ready(Ok(buf.len())) // Pretend to write everything
            }

            fn poll_flush(
                self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), std::io::Error>> {
                std::task::Poll::Ready(Ok(()))
            }

            fn poll_shutdown(
                self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), std::io::Error>> {
                std::task::Poll::Ready(Ok(()))
            }
        }

        impl ConnectionTrait for NullConnection {
            fn peer_addr(&self) -> std::io::Result<SocketAddr> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "Null connection has no address",
                ))
            }

            fn local_addr(&self) -> std::io::Result<SocketAddr> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "Null connection has no address",
                ))
            }

            fn is_closed(&self) -> bool {
                true
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        Conn {
            inner: Box::new(NullConnection),
            is_proxy: false,
            tls_info: None,
        }
    }
}

impl Conn {
    /// Returns whether this connection is through a proxy.
    #[must_use] 
    pub fn is_proxy(&self) -> bool {
        self.is_proxy
    }

    /// Returns TLS information for this connection if available.
    #[must_use] 
    pub fn tls_info(&self) -> Option<&TlsInfo> {
        self.tls_info.as_ref()
    }
}

impl AsyncRead for Conn {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut *self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for Conn {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut *self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut *self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut *self.inner).poll_shutdown(cx)
    }
}

impl Unpin for Conn {}

/// Connection trait for different connection types
pub trait ConnectionTrait: AsyncRead + AsyncWrite + std::fmt::Debug + Send + Sync + Unpin {
    /// Get the peer address of this connection.
    ///
    /// # Errors
    ///
    /// Returns an IO error if the peer address cannot be determined.
    fn peer_addr(&self) -> std::io::Result<SocketAddr>;
    
    /// Get the local address of this connection.
    ///
    /// # Errors
    ///
    /// Returns an IO error if the local address cannot be determined.
    fn local_addr(&self) -> std::io::Result<SocketAddr>;
    fn is_closed(&self) -> bool;
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Broken connection implementation for error handling
#[derive(Debug)]
pub struct BrokenConnectionImpl {
    pub error_message: String,
}

impl BrokenConnectionImpl {
    #[must_use] 
    pub fn new(error_message: String) -> Self {
        Self { error_message }
    }
}

impl AsyncRead for BrokenConnectionImpl {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        _buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            self.error_message.clone(),
        )))
    }
}

impl AsyncWrite for BrokenConnectionImpl {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        _buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::task::Poll::Ready(Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            self.error_message.clone(),
        )))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            self.error_message.clone(),
        )))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            self.error_message.clone(),
        )))
    }
}

impl Unpin for BrokenConnectionImpl {}

impl ConnectionTrait for BrokenConnectionImpl {
    fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            self.error_message.clone(),
        ))
    }

    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            self.error_message.clone(),
        ))
    }

    fn is_closed(&self) -> bool {
        true // Broken connections are always closed
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// TLS connection information
#[derive(Debug, Default, Clone)]
pub struct TlsInfo {
    /// Peer certificate data
    pub peer_certificate: Option<Vec<u8>>,
}
