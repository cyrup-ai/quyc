//! TCP connection utilities with zero-allocation networking
//!
//! Modular TCP connection implementation with DNS resolution, Happy Eyeballs,
//! socket configuration, TLS support, and proxy protocols (HTTP CONNECT, SOCKS).

pub mod basic_connection;
pub mod dns_resolution;
pub mod happy_eyeballs;
pub mod http_connect;
pub mod socket_config;
pub mod socks_protocol;
pub mod tls_connections;

// Re-export main functions for backward compatibility
pub use basic_connection::connect_to_address_list;
pub use dns_resolution::resolve_host_sync;
pub use happy_eyeballs::happy_eyeballs_connect;
pub use http_connect::establish_connect_tunnel;
pub use socket_config::{configure_tcp_socket, configure_tcp_socket_inline};
pub use socks_protocol::{socks_handshake, socks4_handshake, socks5_handshake};
pub use tls_connections::establish_http_connection;

// native-TLS function removed - using rustls universally

#[cfg(feature = "__rustls")]
pub use tls_connections::establish_rustls_connection;
