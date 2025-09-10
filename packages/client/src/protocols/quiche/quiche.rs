//! Direct Quiche QUIC protocol implementation using ystream AsyncStream
//!
//! NO middleware, NO Futures - pure streaming from Quiche to AsyncStream

use std::net::SocketAddr;

use ystream::prelude::MessageChunk;
use ystream::{AsyncStream, emit};

/// Quiche data chunk for streaming
#[derive(Debug, Clone)]
pub struct QuicheChunk {
    pub stream_id: u64,
    pub data: Vec<u8>,
    pub fin: bool,
    pub error_message: Option<String>,
}

impl MessageChunk for QuicheChunk {
    fn bad_chunk(error: String) -> Self {
        Self {
            stream_id: 0,
            data: Vec::new(),
            fin: true,
            error_message: Some(error),
        }
    }

    fn is_error(&self) -> bool {
        self.error_message.is_some()
    }

    fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

/// Direct Quiche connection streaming
pub fn connect_quiche(addr: SocketAddr, server_name: &str) -> AsyncStream<QuicheChunk, 1024> {
    let server_name = server_name.to_string();

    AsyncStream::<QuicheChunk, 1024>::with_channel(move |sender| {
            // Create Quiche configuration
            let mut config = match quiche::Config::new(quiche::PROTOCOL_VERSION) {
                Ok(config) => config,
                Err(e) => {
                    emit!(
                        sender,
                        QuicheChunk::bad_chunk(format!("Config creation failed: {e}"))
                    );
                    return;
                }
            };

            // Configure for HTTP/3
            if let Err(e) = config.set_application_protos(&[b"h3"]) {
                emit!(
                    sender,
                    QuicheChunk::bad_chunk(format!("Failed to set protocols: {e}"))
                );
                return;
            }

            config.set_max_idle_timeout(30000);
            config.set_max_recv_udp_payload_size(1200);
            config.set_max_send_udp_payload_size(1200);
            config.set_initial_max_data(10_000_000);
            config.set_initial_max_stream_data_bidi_local(1_000_000);
            config.set_initial_max_stream_data_bidi_remote(1_000_000);
            config.set_initial_max_streams_bidi(100);
            config.set_disable_active_migration(true);

            // Create connection ID
            let scid = quiche::ConnectionId::from_ref(&[0; 16]);

            // Establish Quiche connection
            match quiche::connect(Some(&server_name), &scid, addr, addr, &mut config) {
                Ok(mut connection) => {
                    // LOOP pattern for continuous streaming
                    loop {
                        // Process readable streams
                        for stream_id in connection.readable() {
                            let mut buf = vec![0; 1024];
                            match connection.stream_recv(stream_id, &mut buf) {
                                Ok((len, fin)) => {
                                    buf.truncate(len);
                                    let chunk = QuicheChunk {
                                        stream_id,
                                        data: buf,
                                        fin,
                                        error_message: None,
                                    };
                                    emit!(sender, chunk);
                                    if fin {
                                        break;
                                    }
                                }
                                Err(quiche::Error::Done) => break,
                                Err(e) => {
                                    emit!(
                                        sender,
                                        QuicheChunk::bad_chunk(format!("Stream read error: {e}"))
                                    );
                                    break;
                                }
                            }
                        }

                        // Check if connection is closed
                        if connection.is_closed() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    emit!(
                        sender,
                        QuicheChunk::bad_chunk(format!("Connection failed: {e}"))
                    );
                }
            }
    })
}

/// Send data on Quiche stream
pub fn send_quiche_data(
    mut connection: quiche::Connection,
    stream_id: u64,
    data: Vec<u8>,
) -> AsyncStream<QuicheChunk, 1024> {
    AsyncStream::<QuicheChunk, 1024>::with_channel(move |sender| {
        match connection.stream_send(stream_id, &data, true) {
            Ok(_) => {
                let chunk = QuicheChunk {
                    stream_id,
                    data,
                    fin: true,
                    error_message: None,
                };
                emit!(sender, chunk);
            }
            Err(e) => {
                emit!(
                    sender,
                    QuicheChunk::bad_chunk(format!("Send failed: {e}"))
                );
            }
        }
    })
}
