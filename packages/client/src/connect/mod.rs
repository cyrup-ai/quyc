//! HTTP/3 connection management and establishment
//!
//! This module provides zero-allocation, lock-free connection handling for HTTP/3 clients.

pub mod builder;
pub mod chunks;
pub mod proxy;
pub mod service;
pub mod tcp;
pub mod types;

// Re-export all public types for backward compatibility
pub use builder::ConnectorBuilder;
pub use chunks::TcpConnectionChunk;
pub use proxy::{
    HttpConnectConfig, Intercepted, ProxyBypass, ProxyConfig, SocksAuth, SocksConfig, SocksVersion,
};
pub use service::ConnectorService;
// native-TLS function removed - using rustls universally
#[cfg(feature = "__rustls")]
pub use tcp::establish_rustls_connection;
pub use tcp::{
    configure_tcp_socket, configure_tcp_socket_inline, connect_to_address_list,
    establish_connect_tunnel, establish_http_connection, happy_eyeballs_connect, resolve_host_sync,
    socks_handshake, socks4_handshake, socks5_handshake,
};
pub use types::{
    BoxedConnectorLayer, BoxedConnectorService, BrokenConnectionImpl, Conn, ConnectionTrait,
    Connector, ConnectorKind, TcpConnection, TcpStreamWrapper, TlsConnection, TlsInfo, Unnameable,
};

// Type aliases for compatibility
pub type Connect = Connector;
pub type HttpConnector = Connector;

// Direct connection method implementation for Connector
impl Connector {
    /// Direct connection method - replaces `Service::call` with `AsyncStream`
    /// RETAINS: All proxy handling, TLS, timeouts, connection pooling functionality
    /// Returns unwrapped `AsyncStream`<TcpConnectionChunk> per async-stream architecture
    pub fn connect(&mut self, dst: http::Uri) -> ystream::AsyncStream<TcpConnectionChunk> {
        match &mut self.inner {
            types::ConnectorKind::WithLayers(s) => s.connect(dst),
            #[cfg(feature = "__tls")]
            types::ConnectorKind::BuiltDefault(s) => s.connect(dst),
            #[cfg(not(feature = "__tls"))]
            types::ConnectorKind::BuiltHttp(s) => s.connect(dst),
        }
    }
}
