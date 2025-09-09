//! Happy Eyeballs (RFC 6555) dual-stack connection implementation
//!
//! Implements RFC 6555 Happy Eyeballs algorithm for optimal IPv4/IPv6 connectivity
//! with parallel connection attempts and intelligent fallback.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use super::basic_connection::connect_to_address_list;

/// Implement Happy Eyeballs (RFC 6555) for optimal dual-stack connectivity.
pub fn happy_eyeballs_connect(
    ipv6_addrs: &[SocketAddr],
    ipv4_addrs: &[SocketAddr],
    delay: Duration,
    timeout: Option<Duration>,
) -> Result<std::net::TcpStream, String> {
    use std::sync::mpsc;
    use std::thread;

    let start = Instant::now();
    let (tx, rx) = mpsc::channel();

    // Try IPv6 first
    let tx_v6 = tx.clone();
    let ipv6_addrs = ipv6_addrs.to_vec();
    let ipv6_timeout = timeout;
    thread::spawn(
        move || match connect_to_address_list(&ipv6_addrs, ipv6_timeout) {
            Ok(stream) => {
                let _ = tx_v6.send(Ok(stream));
            }
            Err(e) => {
                let _ = tx_v6.send(Err(format!("IPv6: {}", e)));
            }
        },
    );

    // Try IPv4 after delay
    let tx_v4 = tx;
    let ipv4_addrs = ipv4_addrs.to_vec();
    let ipv4_timeout = timeout;
    thread::spawn(move || {
        thread::sleep(delay);
        match connect_to_address_list(&ipv4_addrs, ipv4_timeout) {
            Ok(stream) => {
                let _ = tx_v4.send(Ok(stream));
            }
            Err(e) => {
                let _ = tx_v4.send(Err(format!("IPv4: {}", e)));
            }
        }
    });

    // Wait for first successful connection
    let mut errors = Vec::new();
    let overall_timeout = timeout.unwrap_or(Duration::from_secs(30));

    while start.elapsed() < overall_timeout {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(stream)) => return Ok(stream),
            Ok(Err(e)) => errors.push(e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    Err(format!("Happy Eyeballs failed: {:?}", errors))
}
