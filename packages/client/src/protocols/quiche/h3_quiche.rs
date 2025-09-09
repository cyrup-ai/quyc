use std::net::SocketAddr;

use ystream::prelude::*;
use http::{HeaderMap, Method};
use quiche;

use super::h3_adapter::{Http3Chunk, Http3Connection};

/// Compatibility wrapper for the new AsyncStream-based HTTP/3 implementation
pub struct Connection {
    inner: Http3Connection,
}

impl Connection {
    pub fn new(
        conn: quiche::Connection,
        socket: std::net::UdpSocket,
        peer_addr: SocketAddr,
    ) -> Self {
        Self {
            inner: Http3Connection::new(conn, socket, peer_addr),
        }
    }

    /// Send HTTP/3 request and return streaming response
    pub fn send_request(
        &mut self,
        method: &Method,
        path: &str,
        headers: &HeaderMap,
        body: Option<&[u8]>,
    ) -> AsyncStream<Http3Chunk> {
        self.inner.send_request(method, path, headers, body)
    }
}
