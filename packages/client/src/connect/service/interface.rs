//! Service interface implementation for `ConnectorService`
//!
//! Provides the main `connect()` method with elite polling patterns
//! and connection result wrapping using pure `AsyncStream` architecture.

use ystream::{AsyncStream, emit, spawn_task};
use http::Uri;

use super::super::chunks::TcpConnectionChunk;
use super::core::ConnectorService;

impl ConnectorService {
    /// Direct connection method - replaces `Service::call` with `AsyncStream`
    pub fn connect(&mut self, dst: Uri) -> AsyncStream<TcpConnectionChunk> {
        let connector_service = self.clone();

        AsyncStream::with_channel(move |sender| {
            spawn_task(move || {
                let connection_stream =
                    if let Some(_proxy) = connector_service.intercepted.matching(&dst) {
                        connector_service.connect_via_proxy(&dst, "proxy")
                    } else {
                        connector_service.connect_with_maybe_proxy(&dst, false)
                    };

                // Forward all connection events from underlying streams
                while let Some(chunk) = connection_stream.try_next() {
                    emit!(sender, chunk);
                }
            });
        })
    }
}

// Pure AsyncStream implementation - no service traits needed
// The connect() method above provides the complete streaming interface
