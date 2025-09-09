//! HTTP CONNECT tunnel establishment
//!
//! Implements HTTP CONNECT method for establishing tunnels through HTTP proxies
//! with authentication support and proper response parsing.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use http::Uri;

/// Establish HTTP CONNECT tunnel through proxy.
pub fn establish_connect_tunnel(
    mut proxy_stream: TcpStream,
    target_uri: &Uri,
    auth: Option<&str>,
) -> Result<TcpStream, String> {
    let host = target_uri.host().ok_or("Target URI missing host")?;
    let port = target_uri.port_u16().unwrap_or(443);

    // Send CONNECT request
    let connect_request = if let Some(auth) = auth {
        format!(
            "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\nProxy-Authorization: Basic {}\r\n\r\n",
            host, port, host, port, auth
        )
    } else {
        format!(
            "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
            host, port, host, port
        )
    };

    proxy_stream
        .write_all(connect_request.as_bytes())
        .map_err(|e| format!("Failed to send CONNECT request: {}", e))?;

    // Read response
    let mut reader = BufReader::new(&proxy_stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("Failed to read CONNECT response: {}", e))?;

    if !response_line.contains("200") {
        return Err(format!("CONNECT failed: {}", response_line.trim()));
    }

    // Skip remaining headers
    let mut line = String::new();
    loop {
        line.clear();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read CONNECT headers: {}", e))?;
        if line.trim().is_empty() {
            break;
        }
    }

    Ok(proxy_stream)
}
