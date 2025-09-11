//! SOCKS proxy configuration
//!
//! This module contains SOCKS protocol support including version selection
//! and authentication methods for SOCKS4 and SOCKS5 proxies.

/// SOCKS protocol version enumeration.
#[derive(Clone, Copy, Debug)]
pub enum SocksVersion {
    V4,
    V5,
}

/// Proxy authentication methods for SOCKS5
#[derive(Clone, Debug)]
pub enum SocksAuth {
    None,
    UsernamePassword { username: String, password: String },
}

/// SOCKS proxy configuration
#[derive(Clone, Debug)]
pub struct SocksConfig {
    pub version: SocksVersion,
    pub auth: SocksAuth,
    pub target_host: String,
    pub target_port: u16,
}

impl SocksConfig {
    /// Create new SOCKS5 configuration with no authentication
    #[must_use] 
    pub fn socks5_no_auth(target_host: String, target_port: u16) -> Self {
        Self {
            version: SocksVersion::V5,
            auth: SocksAuth::None,
            target_host,
            target_port,
        }
    }

    /// Create new SOCKS5 configuration with username/password authentication
    #[must_use] 
    pub fn socks5_auth(
        target_host: String,
        target_port: u16,
        username: String,
        password: String,
    ) -> Self {
        Self {
            version: SocksVersion::V5,
            auth: SocksAuth::UsernamePassword { username, password },
            target_host,
            target_port,
        }
    }

    /// Create new SOCKS4 configuration
    #[must_use] 
    pub fn socks4(target_host: String, target_port: u16) -> Self {
        Self {
            version: SocksVersion::V4,
            auth: SocksAuth::None, // SOCKS4 doesn't support authentication
            target_host,
            target_port,
        }
    }
}
