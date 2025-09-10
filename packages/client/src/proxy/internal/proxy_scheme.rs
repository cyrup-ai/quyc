//! Proxy scheme configuration
//!
//! This module defines the ProxyScheme enum and its methods
//! for handling different proxy types and authentication.

use std::fmt;
use http::header::HeaderValue;

/// Proxy scheme configuration
#[derive(Clone)]
pub enum ProxyScheme {
    Http {
        auth: Option<HeaderValue>,
        host: String,
        port: u16,
    },
    Https {
        auth: Option<HeaderValue>,
        host: String,
        port: u16,
    },
    Socks5 {
        auth: Option<(String, String)>,
        host: String,
        port: u16,
    },
}

impl ProxyScheme {
    pub fn uri(&self) -> crate::Url {
        match self {
            ProxyScheme::Http { host, port, .. } => {
                format!("http://{}:{}", host, port).parse()
                    .unwrap_or_else(|_| crate::Url::parse("http://localhost").unwrap_or_else(|parse_error| {
                        log::error!("HTTP proxy URL parsing failed: {}", parse_error);
                        crate::Url::parse("data:text/plain,http-proxy-error").expect("data URL must parse")
                    }))
            }
            ProxyScheme::Https { host, port, .. } => {
                format!("https://{}:{}", host, port).parse()
                    .unwrap_or_else(|_| crate::Url::parse("https://localhost").unwrap_or_else(|parse_error| {
                        log::error!("HTTPS proxy URL parsing failed: {}", parse_error);
                        crate::Url::parse("data:text/plain,https-proxy-error").expect("data URL must parse")
                    }))
            }
            ProxyScheme::Socks5 { host, port, .. } => {
                format!("socks5://{}:{}", host, port).parse()
                    .unwrap_or_else(|_| crate::Url::parse("socks5://localhost:1080").unwrap_or_else(|parse_error| {
                        log::error!("SOCKS5 proxy URL parsing failed: {}", parse_error);
                        crate::Url::parse("data:text/plain,socks5-proxy-error").expect("data URL must parse")
                    }))
            }
        }
    }

    pub fn basic_auth(&self) -> Option<(&str, &str)> {
        match self {
            ProxyScheme::Http { .. } | ProxyScheme::Https { .. } => {
                // Basic auth is handled via headers for HTTP/HTTPS
                None
            }
            ProxyScheme::Socks5 { auth, .. } => {
                auth.as_ref().map(|(u, p)| (u.as_str(), p.as_str()))
            }
        }
    }

    pub fn raw_auth(&self) -> Option<(&str, &str)> {
        self.basic_auth()
    }

    pub fn host(&self) -> &str {
        match self {
            ProxyScheme::Http { host, .. } 
            | ProxyScheme::Https { host, .. } => host,
            ProxyScheme::Socks5 { host, .. } => host,
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            ProxyScheme::Http { port, .. } 
            | ProxyScheme::Https { port, .. } => *port,
            ProxyScheme::Socks5 { port, .. } => *port,
        }
    }
}

impl fmt::Debug for ProxyScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyScheme::Http { host, port, .. } => {
                write!(f, "Http({}:{})", host, port)
            }
            ProxyScheme::Https { host, port, .. } => {
                write!(f, "Https({}:{})", host, port)
            }
            ProxyScheme::Socks5 { host, port, .. } => {
                write!(f, "Socks5({}:{})", host, port)
            }
        }
    }
}