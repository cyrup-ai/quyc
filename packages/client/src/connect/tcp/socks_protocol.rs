//! SOCKS protocol implementation
//!
//! Provides complete SOCKS4/SOCKS5 protocol support for proxy connections
//! with authentication and address type handling.

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::str::FromStr;

/// Perform SOCKS handshake with full protocol support.
pub fn socks_handshake(
    stream: TcpStream,
    target_host: &str,
    target_port: u16,
    version: super::super::proxy::SocksVersion,
) -> Result<TcpStream, String> {
    match version {
        super::super::proxy::SocksVersion::V4 => socks4_handshake(stream, target_host, target_port),
        super::super::proxy::SocksVersion::V5 => socks5_handshake(stream, target_host, target_port),
    }
}

/// SOCKS4 handshake implementation.
pub fn socks4_handshake(
    mut stream: TcpStream,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, String> {
    // Try to parse as IP address first
    let target_ip = if let Ok(ipv4) = Ipv4Addr::from_str(target_host) {
        ipv4
    } else {
        // SOCKS4A - use 0.0.0.x to indicate hostname follows
        Ipv4Addr::new(0, 0, 0, 1)
    };

    let mut request = Vec::new();
    request.push(0x04); // Version
    request.push(0x01); // Connect command
    request.extend_from_slice(&target_port.to_be_bytes());
    request.extend_from_slice(&target_ip.octets());
    request.push(0x00); // User ID (empty)

    // Add hostname for SOCKS4A
    if target_ip == Ipv4Addr::new(0, 0, 0, 1) {
        request.extend_from_slice(target_host.as_bytes());
        request.push(0x00);
    }

    stream
        .write_all(&request)
        .map_err(|e| format!("Failed to send SOCKS4 request: {}", e))?;

    let mut response = [0u8; 8];
    stream
        .read_exact(&mut response)
        .map_err(|e| format!("Failed to read SOCKS4 response: {}", e))?;

    if response[1] != 0x5A {
        return Err(format!("SOCKS4 connection rejected: {}", response[1]));
    }

    Ok(stream)
}

/// SOCKS5 handshake implementation.
pub fn socks5_handshake(
    mut stream: TcpStream,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream, String> {
    // Authentication negotiation
    let auth_request = [0x05, 0x01, 0x00]; // Version 5, 1 method, no auth
    stream
        .write_all(&auth_request)
        .map_err(|e| format!("Failed to send SOCKS5 auth request: {}", e))?;

    let mut auth_response = [0u8; 2];
    stream
        .read_exact(&mut auth_response)
        .map_err(|e| format!("Failed to read SOCKS5 auth response: {}", e))?;

    if auth_response[0] != 0x05 || auth_response[1] != 0x00 {
        return Err("SOCKS5 authentication failed".to_string());
    }

    // Connection request
    let mut request = Vec::new();
    request.extend_from_slice(&[0x05, 0x01, 0x00]); // Version, Connect, Reserved

    // Address type and address
    if let Ok(ip) = IpAddr::from_str(target_host) {
        match ip {
            IpAddr::V4(ipv4) => {
                request.push(0x01); // IPv4
                request.extend_from_slice(&ipv4.octets());
            }
            IpAddr::V6(ipv6) => {
                request.push(0x04); // IPv6
                request.extend_from_slice(&ipv6.octets());
            }
        }
    } else {
        request.push(0x03); // Domain name
        request.push(target_host.len() as u8);
        request.extend_from_slice(target_host.as_bytes());
    }

    request.extend_from_slice(&target_port.to_be_bytes());

    stream
        .write_all(&request)
        .map_err(|e| format!("Failed to send SOCKS5 connect request: {}", e))?;

    // Read response
    let mut response = [0u8; 4];
    stream
        .read_exact(&mut response)
        .map_err(|e| format!("Failed to read SOCKS5 response header: {}", e))?;

    if response[1] != 0x00 {
        return Err(format!("SOCKS5 connection rejected: {}", response[1]));
    }

    // Skip bound address (variable length)
    match response[3] {
        0x01 => {
            // IPv4
            let mut addr = [0u8; 6]; // 4 bytes IP + 2 bytes port
            stream
                .read_exact(&mut addr)
                .map_err(|e| format!("Failed to read IPv4 bound address: {}", e))?;
        }
        0x03 => {
            // Domain name
            let mut len = [0u8; 1];
            stream
                .read_exact(&mut len)
                .map_err(|e| format!("Failed to read domain length: {}", e))?;
            let mut domain_and_port = vec![0u8; len[0] as usize + 2];
            stream
                .read_exact(&mut domain_and_port)
                .map_err(|e| format!("Failed to read domain bound address: {}", e))?;
        }
        0x04 => {
            // IPv6
            let mut addr = [0u8; 18]; // 16 bytes IP + 2 bytes port
            stream
                .read_exact(&mut addr)
                .map_err(|e| format!("Failed to read IPv6 bound address: {}", e))?;
        }
        _ => return Err("Invalid SOCKS5 address type in response".to_string()),
    }

    Ok(stream)
}
