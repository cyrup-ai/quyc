//! NoProxy implementation and environment parsing
//!
//! Handles no-proxy configuration from environment variables and string parsing
//! with comprehensive pattern matching for proxy exclusion rules.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use super::types::NoProxy;

impl NoProxy {
    /// Returns a new no-proxy configuration based on environment variables (or `None` if no variables are set)
    /// see [self::NoProxy::from_string()] for the string format
    pub fn from_env() -> Option<NoProxy> {
        let raw = std::env::var("NO_PROXY")
            .or_else(|_| std::env::var("no_proxy"))
            .unwrap_or_default();

        Self::from_string(&raw)
    }

    /// Returns a new no-proxy configuration based on a `no_proxy` string (or `None` if no variables are set)
    /// The rules are as follows:
    /// * The environment variable `NO_PROXY` is checked, if it is not set, `no_proxy` is checked
    /// * If neither environment variable is set, `None` is returned
    /// * Entries are expected to be comma-separated (whitespace between entries is ignored)
    /// * IP addresses (both IPv4 and IPv6) are allowed, as are optional subnet masks (by adding /size,
    ///   for example "`192.168.1.0/24`").
    /// * An entry "`*`" matches all hostnames (this is the only wildcard allowed)
    /// * Any other entry is considered a domain name (and may contain a leading dot, for example `google.com`
    ///   and `.google.com` are equivalent) and would match both that domain AND all subdomains.
    ///
    /// For example, if `"NO_PROXY=google.com, 192.168.1.0/24"` was set, all the following would match
    /// (and therefore would bypass the proxy):
    /// * `http://google.com/`
    /// * `http://www.google.com/`
    /// * `http://192.168.1.42/`
    ///
    /// The URL `http://notgoogle.com/` would not match.
    pub fn from_string(no_proxy_list: &str) -> Option<Self> {
        if no_proxy_list.trim().is_empty() {
            return None;
        }

        Some(NoProxy {
            inner: no_proxy_list.into(),
        })
    }

    /// Check if a host should bypass the proxy based on no-proxy rules
    pub fn matches(&self, host: &str) -> bool {
        for pattern in self.inner.split(',') {
            let pattern = pattern.trim();
            if pattern.is_empty() {
                continue;
            }

            // Wildcard match
            if pattern == "*" {
                return true;
            }

            // Exact match or subdomain match
            if host == pattern || host.ends_with(&format!(".{}", pattern)) {
                return true;
            }

            // Handle leading dot patterns
            if pattern.starts_with('.') && host.ends_with(pattern) {
                return true;
            }

            // Try to parse as IP address or CIDR notation
            if let Some((network_addr, prefix_len)) = parse_cidr_pattern(pattern) {
                // Parse the host as an IP address
                if let Ok(host_ip) = host.parse::<IpAddr>() {
                    if ip_in_subnet(host_ip, network_addr, prefix_len) {
                        return true;
                    }
                }
            } else if let Ok(pattern_ip) = pattern.parse::<IpAddr>() {
                // Direct IP address match (no CIDR)
                if let Ok(host_ip) = host.parse::<IpAddr>() {
                    if host_ip == pattern_ip {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get the raw no-proxy string
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

/// Parse a CIDR pattern (e.g., "192.168.1.0/24" or "2001:db8::/32")
/// Returns Some((network_address, prefix_length)) if valid CIDR notation, None otherwise
fn parse_cidr_pattern(pattern: &str) -> Option<(IpAddr, u8)> {
    let parts: Vec<&str> = pattern.split('/').collect();
    if parts.len() != 2 {
        return None; // Not CIDR notation
    }
    
    let network_str = parts[0];
    let prefix_str = parts[1];
    
    // Parse network address
    let network_addr = network_str.parse::<IpAddr>().ok()?;
    
    // Parse prefix length
    let prefix_len = prefix_str.parse::<u8>().ok()?;
    
    // Validate prefix length based on IP version
    let max_prefix = match network_addr {
        IpAddr::V4(_) => 32,
        IpAddr::V6(_) => 128,
    };
    
    if prefix_len <= max_prefix {
        Some((network_addr, prefix_len))
    } else {
        None
    }
}

/// Check if an IP address is within a subnet
fn ip_in_subnet(ip: IpAddr, network: IpAddr, prefix_len: u8) -> bool {
    match (ip, network) {
        (IpAddr::V4(ip_v4), IpAddr::V4(net_v4)) => {
            ipv4_in_subnet(ip_v4, net_v4, prefix_len)
        }
        (IpAddr::V6(ip_v6), IpAddr::V6(net_v6)) => {
            ipv6_in_subnet(ip_v6, net_v6, prefix_len)
        }
        _ => false, // Different IP versions don't match
    }
}

/// Check if an IPv4 address is within an IPv4 subnet using bit masking
fn ipv4_in_subnet(ip: Ipv4Addr, network: Ipv4Addr, prefix_len: u8) -> bool {
    if prefix_len > 32 {
        return false;
    }
    
    let ip_u32 = u32::from(ip);
    let network_u32 = u32::from(network);
    
    if prefix_len == 0 {
        return true; // 0.0.0.0/0 matches everything
    }
    
    // Create subnet mask: shift left (32 - prefix_len) bits and invert
    let mask = !((1u32 << (32 - prefix_len)) - 1);
    
    // Apply mask to both IP addresses and compare
    (ip_u32 & mask) == (network_u32 & mask)
}

/// Check if an IPv6 address is within an IPv6 subnet using bit masking
fn ipv6_in_subnet(ip: Ipv6Addr, network: Ipv6Addr, prefix_len: u8) -> bool {
    if prefix_len > 128 {
        return false;
    }
    
    let ip_bytes = ip.octets();
    let network_bytes = network.octets();
    
    if prefix_len == 0 {
        return true; // ::/0 matches everything
    }
    
    // Calculate how many full bytes and remaining bits to check
    let full_bytes = (prefix_len / 8) as usize;
    let remaining_bits = prefix_len % 8;
    
    // Check full bytes
    if ip_bytes[..full_bytes] != network_bytes[..full_bytes] {
        return false;
    }
    
    // Check remaining bits in the partial byte
    if remaining_bits > 0 && full_bytes < 16 {
        let mask = 0xFFu8 << (8 - remaining_bits);
        let ip_masked = ip_bytes[full_bytes] & mask;
        let network_masked = network_bytes[full_bytes] & mask;
        
        if ip_masked != network_masked {
            return false;
        }
    }
    
    true
}
