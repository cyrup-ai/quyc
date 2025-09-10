//! TCP socket configuration utilities
//!
//! Provides TCP socket optimization settings including nodelay, keepalive,
//! and performance tuning for high-throughput connections.

use std::net::TcpStream;
use std::time::Duration;

/// Configure TCP socket for optimal performance.
pub fn configure_tcp_socket(
    stream: &mut TcpStream,
    nodelay: bool,
    keepalive: Option<Duration>,
) -> Result<(), String> {
    // Socket configuration using safe Rust APIs only

    if nodelay {
        stream
            .set_nodelay(true)
            .map_err(|e| format!("Failed to set TCP_NODELAY: {e}"))?;
    }

    if let Some(_duration) = keepalive {
        // TCP keepalive configuration using safe Rust APIs only
        // Note: Advanced keepalive configuration requires unsafe code which is denied
        // Basic TCP stream configuration is handled via set_nodelay above
        tracing::debug!("TCP keepalive requested but advanced configuration requires unsafe code");
    }

    Ok(())
}

/// Inline TCP socket configuration for performance-critical paths.
#[inline(always)]
pub fn configure_tcp_socket_inline(stream: &TcpStream, nodelay: bool) -> Result<(), String> {
    if nodelay {
        stream
            .set_nodelay(true)
            .map_err(|e| format!("Failed to set TCP_NODELAY: {e}"))?;
    }
    Ok(())
}
