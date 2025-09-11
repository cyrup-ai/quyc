//! Quiche QUIC Streaming Implementation
//!
//! Provides streaming primitives for Quiche QUIC connections using ystream patterns.
//! All operations follow zero-allocation streaming architecture with error-as-data patterns.

use std::net::{SocketAddr, UdpSocket};
// Arc import removed - not used

use bytes::Bytes;
use crossbeam_utils::Backoff;
use ystream::prelude::*;
use quiche::{Config, Connection, ConnectionId};

use crate::protocols::quiche::chunks::{QuicheStreamChunk, QuichePacketChunk, QuicheWriteResult};

/// Quiche connection state chunk for connection lifecycle events
#[derive(Debug)]
pub struct QuicheConnectionChunk {
    pub local_addr: Option<SocketAddr>,
    pub peer_addr: Option<SocketAddr>,
    pub is_established: bool,
    pub is_closed: bool,
    pub timeout_ms: Option<u64>,
    pub error: Option<String>,
    pub connection: Option<QuicheConnection>,
}

impl QuicheConnectionChunk {
    #[inline]
    #[must_use] 
    pub fn established(local: SocketAddr, peer: SocketAddr) -> Self {
        Self {
            local_addr: Some(local),
            peer_addr: Some(peer),
            is_established: true,
            is_closed: false,
            timeout_ms: None,
            error: None,
            connection: None,
        }
    }

    #[inline]
    #[must_use] 
    pub fn connection_ready(connection: QuicheConnection) -> Self {
        let local = connection.local_addr();
        let peer = connection.peer_addr();
        Self {
            local_addr: Some(local),
            peer_addr: Some(peer),
            is_established: true,
            is_closed: false,
            timeout_ms: None,
            error: None,
            connection: Some(connection),
        }
    }

    #[inline]
    #[must_use] 
    pub fn connection_closed(local: SocketAddr, peer: SocketAddr) -> Self {
        Self {
            local_addr: Some(local),
            peer_addr: Some(peer),
            is_established: false,
            is_closed: true,
            timeout_ms: None,
            error: None,
            connection: None,
        }
    }

    #[inline]
    #[must_use] 
    pub fn timeout_event(timeout_ms: u64) -> Self {
        Self {
            local_addr: None,
            peer_addr: None,
            is_established: false,
            is_closed: false,
            timeout_ms: Some(timeout_ms),
            error: None,
            connection: None,
        }
    }

    #[inline]
    #[must_use] 
    pub fn streams_available(_readable_streams: Vec<u64>, _writable_streams: Vec<u64>) -> Self {
        Self {
            local_addr: None,
            peer_addr: None,
            is_established: true,
            is_closed: false,
            timeout_ms: None,
            error: None,
            connection: None,
        }
    }
}

impl MessageChunk for QuicheConnectionChunk {
    fn bad_chunk(error: String) -> Self {
        Self {
            local_addr: None,
            peer_addr: None,
            is_established: false,
            is_closed: false,
            timeout_ms: None,
            error: Some(error),
            connection: None,
        }
    }

    fn is_error(&self) -> bool {
        self.error.is_some()
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl Default for QuicheConnectionChunk {
    fn default() -> Self {
        Self::bad_chunk("Default QuicheConnectionChunk".to_string())
    }
}

impl Clone for QuicheConnectionChunk {
    fn clone(&self) -> Self {
        Self {
            local_addr: self.local_addr,
            peer_addr: self.peer_addr,
            is_established: self.is_established,
            is_closed: self.is_closed,
            timeout_ms: self.timeout_ms,
            error: self.error.clone(),
            connection: None, // Don't clone connections - they represent unique network state
        }
    }
}

/// Quiche QUIC connection wrapper providing streaming primitives
#[derive(Debug)]
pub struct QuicheConnection {
    connection: QuicheConnectionWrapper,
    socket: UdpSocket,
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
    is_server: bool,
}

impl QuicheConnection {
    /// Create new connection from established Quiche connection
    #[inline]
    #[must_use] 
    pub fn new(
        connection: Connection,
        socket: UdpSocket,
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        is_server: bool,
    ) -> Self {
        Self {
            connection: QuicheConnectionWrapper::new(connection),
            socket,
            local_addr,
            peer_addr,
            is_server,
        }
    }

    /// Get connection local address
    #[inline]
    #[must_use] 
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Get connection peer address
    #[inline]
    #[must_use] 
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    /// Check if connection is established
    #[inline]
    #[must_use] 
    pub fn is_established(&self) -> bool {
        self.connection.inner.is_established()
    }

    /// Check if connection is closed
    #[inline]
    #[must_use] 
    pub fn is_closed(&self) -> bool {
        self.connection.inner.is_closed()
    }

    /// Stream readable data from connection
    #[must_use] 
    pub fn readable_streams(self) -> AsyncStream<QuicheConnectionChunk, 1024> {
        read_readable_streams(self.connection.into_inner())
    }

    /// Open bidirectional stream
    #[must_use] 
    pub fn open_bidi_stream(self) -> AsyncStream<QuicheStreamChunk, 1024> {
        open_bidirectional_stream(self.connection.into_inner(), self.is_server)
    }

    /// Open unidirectional stream
    #[must_use] 
    pub fn open_uni_stream(self) -> AsyncStream<QuicheStreamChunk, 1024> {
        open_unidirectional_stream(self.connection.into_inner(), self.is_server)
    }

    /// Process incoming packets
    #[must_use] 
    pub fn process_packets(self, packets: Vec<Bytes>) -> AsyncStream<QuichePacketChunk, 1024> {
        process_incoming_packets(
            self.connection.into_inner(),
            packets,
            self.local_addr,
            self.peer_addr,
        )
    }
}

// CORRECT: Connect to server using synchronous Quiche APIs
#[must_use] 
pub fn connect_to_server(
    server_name: &str,
    scid: &ConnectionId,
    local: SocketAddr,
    peer: SocketAddr,
    mut config: Config,
    socket: UdpSocket,
) -> AsyncStream<QuicheConnectionChunk, 1024> {
    let server_name = server_name.to_string();
    let scid_owned = scid.to_vec();
    AsyncStream::with_channel(move |sender| {
        // Create proper ConnectionId from bytes using real quiche API
        let scid = quiche::ConnectionId::from_ref(&scid_owned);
        
        // Use real quiche::connect method with proper signature
        match quiche::connect(Some(&server_name), &scid, local, peer, &mut config) {
            Ok(connection) => {
                // Connection established successfully - emit it for use
                let quiche_conn = QuicheConnection::new(connection, socket, local, peer, false); // client
                emit!(sender, QuicheConnectionChunk::connection_ready(quiche_conn));
            }
            Err(e) => {
                emit!(
                    sender,
                    QuicheConnectionChunk::bad_chunk(format!("Connection error: {e}"))
                );
            }
        }
    })
}

// CORRECT: Accept Quiche connection using synchronous APIs
#[must_use] 
pub fn accept_connection(
    scid: &ConnectionId,
    odcid: Option<&ConnectionId>,
    local: SocketAddr,
    peer: SocketAddr,
    mut config: Config,
    socket: UdpSocket,
) -> AsyncStream<QuicheConnectionChunk, 1024> {
    let scid = scid.clone();
    let odcid = odcid.cloned();
    let scid_owned = scid.to_vec();
    let odcid_owned = odcid.map(|o| o.to_vec());
    let local_addr = local;
    let peer_addr = peer;
    AsyncStream::with_channel(move |sender| {
        // Create proper ConnectionIds from bytes using real quiche API
        let scid = quiche::ConnectionId::from_vec(scid_owned);
        let odcid = odcid_owned.map(quiche::ConnectionId::from_vec);
        
        // Use real quiche::accept method with proper signature  
        match quiche::accept(
            &scid,
            odcid.as_ref(),
            local_addr,
            peer_addr,
            &mut config,
        ) {
            Ok(connection) => {
                // Connection accepted successfully - emit it for use
                let quiche_conn =
                    QuicheConnection::new(connection, socket, local_addr, peer_addr, true); // server
                emit!(sender, QuicheConnectionChunk::connection_ready(quiche_conn));
            }
            Err(e) => {
                emit!(
                    sender,
                    QuicheConnectionChunk::bad_chunk(format!("Accept error: {e}"))
                );
            }
        }
    })
}

/// Stream readable streams from Quiche connection  
#[must_use] 
pub fn read_readable_streams(connection: Connection) -> AsyncStream<QuicheConnectionChunk, 1024> {
    AsyncStream::with_channel(move |sender| {
        let mut readable_streams = Vec::new();
        let mut writable_streams = Vec::new();

        // Iterate through readable streams
        for stream_id in connection.readable() {
            readable_streams.push(stream_id);
        }

        // Iterate through writable streams
        for stream_id in connection.writable() {
            writable_streams.push(stream_id);
        }

        if !readable_streams.is_empty() || !writable_streams.is_empty() {
            emit!(
                sender,
                QuicheConnectionChunk::streams_available(readable_streams, writable_streams)
            );
        } else if connection.is_closed() {
            // Use default socket addresses since they're not available in this context
            // SECURITY: Handle hardcoded address parsing gracefully to prevent panics
            let default_addr = "0.0.0.0:0".parse()
                .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 0)));
            emit!(
                sender,
                QuicheConnectionChunk::connection_closed(default_addr, default_addr)
            );
        }
    })
}

/// Open bidirectional stream with proper sequence tracking
#[must_use] 
pub fn open_bidirectional_stream(
    mut connection: Connection,
    is_server: bool,
) -> AsyncStream<QuicheStreamChunk, 1024> {
    AsyncStream::with_channel(move |sender| {
        // Calculate next available bidirectional stream ID based on server/client role
        // Client bidi streams: 0, 4, 8, 12... (stream_id & 0x3 == 0)
        // Server bidi streams: 1, 5, 9, 13... (stream_id & 0x3 == 1)
        let base_stream_id = u64::from(is_server);

        // Check streams to find the next available ID
        // Start from base and increment by 4 until we find an unused one
        let mut stream_id = base_stream_id;
        let max_attempts = 1024; // Reasonable limit

        for _ in 0..max_attempts {
            // Try to create the stream by sending 0-length data
            match connection.stream_send(stream_id, b"", false) {
                Ok(_) => {
                    // Stream created successfully
                    emit!(sender, QuicheStreamChunk::stream_opened(stream_id, true));
                    return;
                }
                Err(quiche::Error::StreamLimit) => {
                    // Hit stream limit, cannot create more streams
                    emit!(
                        sender,
                        QuicheStreamChunk::bad_chunk(
                            "Cannot create bidirectional stream: stream limit reached".to_string()
                        )
                    );
                    return;
                }
                Err(quiche::Error::InvalidStreamState(_)) => {
                    // Stream already exists or invalid, try next
                    stream_id += 4;
                }
                Err(e) => {
                    // Other error
                    emit!(
                        sender,
                        QuicheStreamChunk::bad_chunk(format!(
                            "Failed to create bidirectional stream {stream_id}: {e}"
                        ))
                    );
                    return;
                }
            }
        }

        // Exhausted attempts
        emit!(
            sender,
            QuicheStreamChunk::bad_chunk(format!(
                "Could not find available bidirectional stream ID after {max_attempts} attempts"
            ))
        );
    })
}

/// Open unidirectional stream with proper sequence tracking
#[must_use] 
pub fn open_unidirectional_stream(
    mut connection: Connection,
    is_server: bool,
) -> AsyncStream<QuicheStreamChunk, 1024> {
    AsyncStream::with_channel(move |sender| {
        // Calculate next available unidirectional stream ID based on server/client role
        // Client uni streams: 2, 6, 10, 14... (stream_id & 0x3 == 2)
        // Server uni streams: 3, 7, 11, 15... (stream_id & 0x3 == 3)
        let base_stream_id = if is_server { 3u64 } else { 2u64 };

        // Check streams to find the next available ID
        // Start from base and increment by 4 until we find an unused one
        let mut stream_id = base_stream_id;
        let max_attempts = 1024; // Reasonable limit

        for _ in 0..max_attempts {
            // Try to create the stream by sending 0-length data
            match connection.stream_send(stream_id, b"", false) {
                Ok(_) => {
                    // Stream created successfully
                    emit!(sender, QuicheStreamChunk::stream_opened(stream_id, false));
                    return;
                }
                Err(quiche::Error::StreamLimit) => {
                    // Hit stream limit, cannot create more streams
                    emit!(
                        sender,
                        QuicheStreamChunk::bad_chunk(
                            "Cannot create unidirectional stream: stream limit reached".to_string()
                        )
                    );
                    return;
                }
                Err(quiche::Error::InvalidStreamState(_)) => {
                    // Stream already exists or invalid, try next
                    stream_id += 4;
                }
                Err(e) => {
                    // Other error
                    emit!(
                        sender,
                        QuicheStreamChunk::bad_chunk(format!(
                            "Failed to create unidirectional stream {stream_id}: {e}"
                        ))
                    );
                    return;
                }
            }
        }

        // Exhausted attempts
        emit!(
            sender,
            QuicheStreamChunk::bad_chunk(format!(
                "Could not find available unidirectional stream ID after {max_attempts} attempts"
            ))
        );
    })
}

/// Process incoming packets
#[must_use] 
pub fn process_incoming_packets(
    mut connection: Connection,
    packets: Vec<Bytes>,
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
) -> AsyncStream<QuichePacketChunk, 1024> {
    AsyncStream::with_channel(move |sender| {
        for packet in packets {
            // Convert Bytes to mutable buffer for quiche recv
            let mut packet_buf = packet.to_vec();
            match connection.recv(
                &mut packet_buf,
                quiche::RecvInfo {
                    to: local_addr,  // Correct: Use actual local socket address
                    from: peer_addr, // Correct: Use actual peer socket address
                },
            ) {
                Ok(bytes_processed) => {
                    emit!(
                        sender,
                        QuichePacketChunk::packet_processed(
                            bytes_processed,
                            peer_addr,  // from
                            local_addr  // to
                        )
                    );
                }
                Err(e) => {
                    emit!(
                        sender,
                        QuichePacketChunk::bad_chunk(format!("Packet processing error: {e}"))
                    );
                }
            }
        }
    })
}

/// Read data from specific stream using elite polling pattern
#[must_use] 
pub fn read_stream_data(
    mut connection: Connection,
    stream_id: u64,
) -> AsyncStream<QuicheStreamChunk, 1024> {
    AsyncStream::with_channel(move |sender| {
        let mut buffer = vec![0; 4096];
        let backoff = Backoff::new();

        loop {
            match connection.stream_recv(stream_id, &mut buffer) {
                Ok((bytes_read, fin)) => {
                    if bytes_read > 0 {
                        buffer.truncate(bytes_read);
                        emit!(
                            sender,
                            QuicheStreamChunk::stream_data(stream_id, buffer.clone(), fin)
                        );
                        backoff.reset();
                        buffer.resize(4096, 0); // Reset buffer for next read
                    }

                    if fin {
                        emit!(sender, QuicheStreamChunk::stream_finished(stream_id));
                        break;
                    }
                }
                Err(quiche::Error::Done) => {
                    // Elite backoff pattern - no data available, only use snooze
                    backoff.snooze();
                    continue;
                }
                Err(e) => {
                    emit!(
                        sender,
                        QuicheStreamChunk::bad_chunk(format!("Stream read error: {e}"))
                    );
                    break;
                }
            }

            // Check if connection is closed
            if connection.is_closed() {
                emit!(sender, QuicheStreamChunk::stream_finished(stream_id));
                break;
            }
        }
    })
}

/// Write data to specific stream
#[must_use] 
pub fn write_stream_data(
    mut connection: Connection,
    stream_id: u64,
    data: Vec<u8>,
    fin: bool,
) -> AsyncStream<QuicheWriteResult, 1024> {
    AsyncStream::with_channel(move |sender| {
        match connection.stream_send(stream_id, &data, fin) {
            Ok(bytes_written) => {
                emit!(
                    sender,
                    QuicheWriteResult::bytes_written(stream_id, bytes_written, fin)
                );
            }
            Err(quiche::Error::Done) => {
                // Stream is blocked
                emit!(sender, QuicheWriteResult::write_blocked(stream_id));
            }
            Err(e) => {
                emit!(
                    sender,
                    QuicheWriteResult::bad_chunk(format!("Stream write error: {e}"))
                );
            }
        }
    })
}

/// Debug wrapper for `quiche::Connection`
pub struct QuicheConnectionWrapper {
    inner: Connection,
}

impl std::fmt::Debug for QuicheConnectionWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicheConnectionWrapper")
            .field("inner", &"<quiche::Connection>")
            .finish()
    }
}

impl QuicheConnectionWrapper {
    #[inline]
    #[must_use] 
    pub fn new(connection: Connection) -> Self {
        Self { inner: connection }
    }

    #[inline]
    pub fn inner(&mut self) -> &mut Connection {
        &mut self.inner
    }

    #[inline]
    #[must_use] 
    pub fn into_inner(self) -> Connection {
        self.inner
    }
}

/// Quiche stream wrapper for individual stream operations
#[derive(Debug)]
pub struct QuicheStreamWrapper {
    connection: Option<QuicheConnectionWrapper>,
    stream_id: u64,
    error: Option<String>,
}

impl QuicheStreamWrapper {
    #[inline]
    #[must_use] 
    pub fn new(connection: Connection, stream_id: u64) -> Self {
        Self {
            connection: Some(QuicheConnectionWrapper::new(connection)),
            stream_id,
            error: None,
        }
    }

    #[inline]
    #[must_use] 
    pub fn stream_id(&self) -> u64 {
        self.stream_id
    }

    /// Read data from this stream
    #[must_use] 
    pub fn read_data(self) -> AsyncStream<QuicheStreamChunk, 1024> {
        if let Some(connection) = self.connection {
            read_stream_data(connection.into_inner(), self.stream_id)
        } else {
            AsyncStream::with_channel(move |sender| {
                emit!(
                    sender,
                    QuicheStreamChunk::bad_chunk("No connection available for read".to_string())
                );
            })
        }
    }

    /// Write data to this stream
    #[must_use] 
    pub fn write_data(self, data: Vec<u8>, fin: bool) -> AsyncStream<QuicheWriteResult, 1024> {
        if let Some(connection) = self.connection {
            write_stream_data(connection.into_inner(), self.stream_id, data, fin)
        } else {
            AsyncStream::with_channel(move |sender| {
                emit!(
                    sender,
                    QuicheWriteResult::bad_chunk("No connection available for write".to_string())
                );
            })
        }
    }

    /// Write bytes to this stream
    pub fn write_bytes(self, data: Bytes, fin: bool) -> AsyncStream<QuicheWriteResult, 1024> {
        if let Some(connection) = self.connection {
            write_stream_data(connection.into_inner(), self.stream_id, data.to_vec(), fin)
        } else {
            AsyncStream::with_channel(move |sender| {
                emit!(
                    sender,
                    QuicheWriteResult::bad_chunk(
                        "No connection available for write_bytes".to_string()
                    )
                );
            })
        }
    }
}

impl MessageChunk for QuicheStreamWrapper {
    fn bad_chunk(error_message: String) -> Self {
        Self {
            connection: None,
            stream_id: 0,
            error: Some(error_message),
        }
    }

    fn is_error(&self) -> bool {
        self.error.is_some()
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl Default for QuicheStreamWrapper {
    fn default() -> Self {
        Self::bad_chunk("Default QuicheStreamWrapper".to_string())
    }
}



