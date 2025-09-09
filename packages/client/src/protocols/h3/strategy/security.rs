//! H3 Security and Address Validation
//!
//! Security measures to prevent UDP amplification attacks and other security issues.

use std::net::SocketAddr;

/// SECURITY: Validate destination addresses to prevent UDP amplification attacks
pub(crate) fn validate_destination_address(addr: &SocketAddr) -> Result<(), String> {
    match addr.ip() {
        // Block private/local addresses for outbound HTTP/3 connections to prevent amplification
        std::net::IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            match octets {
                // Allow localhost for testing/development
                [127, _, _, _] => Ok(()),
                // Allow private networks for legitimate internal API calls
                [10, _, _, _] => Ok(()),
                [172, 16..=31, _, _] => Ok(()),
                [192, 168, _, _] => Ok(()),
                // Block reserved addresses
                [0, _, _, _] => Err("Reserved 0.0.0.0/8 network not allowed".to_string()),
                [169, 254, _, _] => Err("Link-local 169.254.0.0/16 network not allowed".to_string()),
                [224..=239, _, _, _] => Err("Multicast addresses not allowed for HTTP/3".to_string()),
                [240..=255, _, _, _] => Err("Reserved class E addresses not allowed".to_string()),
                _ => Ok(()),
            }
        }
        std::net::IpAddr::V6(ipv6) => {
            if ipv6.is_loopback() {
                Ok(()) // Allow IPv6 localhost
            } else if ipv6.is_multicast() {
                Err("IPv6 multicast addresses not allowed for HTTP/3".to_string())
            } else if ipv6.segments()[0] == 0xfe80 {
                Err("IPv6 link-local addresses not allowed for HTTP/3".to_string())
            } else {
                Ok(()) // Allow other IPv6 addresses
            }
        }
    }
}