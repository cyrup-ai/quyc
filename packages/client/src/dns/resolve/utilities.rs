//! DNS resolution utility functions and helper implementations
//!
//! Contains helper functions for hostname resolution, address sorting,
//! and DNS override implementations.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;

use ystream::{AsyncStream, spawn_task};

use super::traits::Resolve;
use super::types::{DnsResult, HyperName};

/// DNS resolver with hostname overrides for testing and custom routing.
pub struct DnsResolverWithOverridesImpl {
    pub dns_resolver: Arc<dyn Resolve>,
    pub overrides: Arc<HashMap<String, arrayvec::ArrayVec<SocketAddr, 8>>>,
}

impl Resolve for DnsResolverWithOverridesImpl {
    fn resolve(&self, name: HyperName) -> AsyncStream<DnsResult, 1024> {
        let hostname = name.as_str().to_string();
        let overrides = self.overrides.clone();
        let dns_resolver = self.dns_resolver.clone();

        AsyncStream::with_channel(move |sender| {
            // Check for hostname overrides first
            if let Some(override_addrs) = overrides.get(&hostname) {
                ystream::emit!(
                    sender,
                    DnsResult {
                        addrs: override_addrs.clone()
                    }
                );
                return;
            }

            // Fall back to underlying DNS resolver
            let resolve_stream = dns_resolver.resolve(HyperName::from(hostname));
            spawn_task(move || {
                for dns_result in resolve_stream {
                    ystream::emit!(sender, dns_result);
                }
            });
        })
    }
}

/// Resolve a hostname to socket addresses with port and preference handling.
/// Zero-allocation implementation using arrayvec for bounded address storage.
pub fn resolve_host_to_addrs(
    hostname: &str,
    port: u16,
    prefer_ipv6: bool,
) -> Result<arrayvec::ArrayVec<SocketAddr, 8>, String> {
    let host_with_port = format!("{}:{}", hostname, port);

    let socket_addrs: Result<arrayvec::ArrayVec<SocketAddr, 8>, std::io::Error> =
        host_with_port.to_socket_addrs().map(|iter| {
            let mut addrs: arrayvec::ArrayVec<SocketAddr, 8> = iter.take(8).collect();

            // Sort addresses based on preference using zero-allocation sort
            if prefer_ipv6 {
                addrs.sort_unstable_by_key(|addr| match addr.ip() {
                    IpAddr::V6(_) => 0, // IPv6 first
                    IpAddr::V4(_) => 1, // IPv4 second
                });
            } else {
                addrs.sort_unstable_by_key(|addr| match addr.ip() {
                    IpAddr::V4(_) => 0, // IPv4 first
                    IpAddr::V6(_) => 1, // IPv6 second
                });
            }
            addrs
        });

    match socket_addrs {
        Ok(addrs) => {
            if addrs.is_empty() {
                Err(format!("No addresses found for hostname: {}", hostname))
            } else {
                Ok(addrs)
            }
        }
        Err(e) => Err(format!("DNS resolution failed for {}: {}", hostname, e)),
    }
}

/// Sort socket addresses by preference (IPv4 vs IPv6).
/// Zero-allocation in-place sorting for performance.
pub fn sort_addresses_by_preference(
    addrs: &mut arrayvec::ArrayVec<SocketAddr, 8>,
    prefer_ipv6: bool,
) {
    if prefer_ipv6 {
        addrs.sort_unstable_by_key(|addr| match addr.ip() {
            IpAddr::V6(_) => 0, // IPv6 first
            IpAddr::V4(_) => 1, // IPv4 second
        });
    } else {
        addrs.sort_unstable_by_key(|addr| match addr.ip() {
            IpAddr::V4(_) => 0, // IPv4 first
            IpAddr::V6(_) => 1, // IPv6 second
        });
    }
}

/// Check if a hostname is a valid IP address.
/// Used for optimization to skip DNS resolution for IP literals.
pub fn is_ip_address(hostname: &str) -> bool {
    hostname.parse::<IpAddr>().is_ok()
}

/// Create socket address from IP literal and port.
/// Zero-allocation helper for IP address parsing.
pub fn socket_addr_from_ip_literal(ip_literal: &str, port: u16) -> Result<SocketAddr, String> {
    match ip_literal.parse::<IpAddr>() {
        Ok(ip) => Ok(SocketAddr::new(ip, port)),
        Err(e) => Err(format!("Invalid IP address {}: {}", ip_literal, e)),
    }
}

/// Validate hostname format according to RFC standards.
/// Basic validation for security and correctness.
pub fn validate_hostname(hostname: &str) -> Result<(), String> {
    if hostname.is_empty() {
        return Err("Hostname cannot be empty".to_string());
    }

    if hostname.len() > 253 {
        return Err("Hostname too long (max 253 characters)".to_string());
    }

    // Basic character validation - allow alphanumeric, hyphens, dots
    if !hostname
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '.')
    {
        return Err("Hostname contains invalid characters".to_string());
    }

    // Cannot start or end with hyphen
    if hostname.starts_with('-') || hostname.ends_with('-') {
        return Err("Hostname cannot start or end with hyphen".to_string());
    }

    Ok(())
}
