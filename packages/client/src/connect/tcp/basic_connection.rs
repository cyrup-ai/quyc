//! Basic TCP connection establishment
//!
//! Provides fundamental TCP connection functionality with timeout support
//! and address list iteration for connection reliability.

use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// Connect to first available address with timeout support.
pub fn connect_to_address_list(
    addrs: &[SocketAddr],
    timeout: Option<Duration>,
) -> Result<TcpStream, String> {
    if addrs.is_empty() {
        return Err("No addresses to connect to".to_string());
    }

    for addr in addrs {
        match timeout {
            Some(t) => {
                match TcpStream::connect_timeout(addr, t) {
                    Ok(stream) => return Ok(stream),
                    Err(e) => {
                        // Log error and continue to next address
                        tracing::debug!("Failed to connect to {}: {}", addr, e);
                    }
                }
            }
            None => match TcpStream::connect(addr) {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    tracing::debug!("Failed to connect to {}: {}", addr, e);
                }
            },
        }
    }

    Err("Failed to connect to any address".to_string())
}
