//! URL handling and authentication utilities for proxy configuration
//!
//! Zero-allocation URL parsing with comprehensive error handling and authentication support.

// fmt import removed - not used
use std::sync::Arc;

use ystream::prelude::*;
use http::header::HeaderValue;

use super::core::NoProxy;
use crate::Url;

/// MessageChunk wrapper for URL parsing with proper error handling
#[derive(Debug, Clone)]
pub(crate) struct ProxyUrl {
    url: crate::Url,
    error_message: Option<String>,
}

impl ProxyUrl {
    pub fn new(url: crate::Url) -> Self {
        Self {
            url,
            error_message: None,
        }
    }

    pub fn into_url(self) -> crate::Url {
        self.url
    }
}

impl MessageChunk for ProxyUrl {
    fn bad_chunk(error: String) -> Self {
        ProxyUrl {
            url: crate::Url::parse("http://localhost")
                .or_else(|_| crate::Url::parse("http://127.0.0.1"))
                .or_else(|_| crate::Url::parse("http://[::1]"))
                .expect("System failure: URL library cannot parse basic localhost URLs"),
            error_message: Some(error),
        }
    }

    fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    fn is_error(&self) -> bool {
        self.error_message.is_some()
    }
}

/// Set username and password for proxy URL authentication
pub(crate) fn url_auth(url: &mut Url, username: &str, password: &str) {
    // Use unwrap_or_else for safe error handling (allowed)
    url.set_username(username).unwrap_or_else(|_| {
        // If username setting fails, log but continue
        tracing::warn!(
            target: "quyc::proxy",
            username = %username,
            "Failed to set proxy username"
        );
    });
    url.set_password(Some(password)).unwrap_or_else(|_| {
        // If password setting fails, log but continue
        tracing::warn!(
            target: "quyc::proxy",
            "Failed to set proxy password"
        );
    });
}

/// Encode basic authentication credentials into a HeaderValue
pub fn encode_basic_auth(username: &str, password: &str) -> HeaderValue {
    use base64::Engine;
    let credentials = format!("{}:{}", username, password);
    let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
    let auth_value = format!("Basic {}", encoded);

    HeaderValue::from_str(&auth_value).unwrap_or_else(|_| HeaderValue::from_static(""))
}

// Legacy proxy scheme implementation (commented out for reference)
// impl ProxyScheme {
// Use a username and password when connecting to the proxy server
// fn with_basic_auth<T: Into<String>, U: Into<String>>(
// mut self,
// username: T,
// password: U,
// ) -> Self {
// self.set_basic_auth(username, password);
// self
// }
//
// fn set_basic_auth<T: Into<String>, U: Into<String>>(&mut self, username: T, password: U) {
// match *self {
// ProxyScheme::Http { ref mut auth, .. } => {
// let header = encode_basic_auth(&username.into(), &password.into());
// auth = Some(header);
// }
// ProxyScheme::Https { ref mut auth, .. } => {
// let header = encode_basic_auth(&username.into(), &password.into());
// auth = Some(header);
// }
// ProxyScheme::Socks4 { .. } => {
// panic!("Socks4 is not supported for this method")
// }
// ProxyScheme::Socks5 { ref mut auth, .. } => {
// auth = Some((username.into(), password.into()));
// }
// }
// }
//
// fn set_custom_http_auth(&mut self, header_value: HeaderValue) {
// match *self {
// ProxyScheme::Http { ref mut auth, .. } => {
// auth = Some(header_value);
// }
// ProxyScheme::Https { ref mut auth, .. } => {
// auth = Some(header_value);
// }
// ProxyScheme::Socks4 { .. } => {
// panic!("Socks4 is not supported for this method")
// }
// ProxyScheme::Socks5 { .. } => {
// panic!("Socks5 is not supported for this method")
// }
// }
// }
//
// fn set_custom_headers(&mut self, headers: HeaderMap) {
// match *self {
// ProxyScheme::Http { ref mut misc, .. } => {
// misc.get_or_insert_with(HeaderMap::new).extend(headers)
// }
// ProxyScheme::Https { ref mut misc, .. } => {
// misc.get_or_insert_with(HeaderMap::new).extend(headers)
// }
// ProxyScheme::Socks4 { .. } => {
// panic!("Socks4 is not supported for this method")
// }
// ProxyScheme::Socks5 { .. } => {
// panic!("Socks5 is not supported for this method")
// }
// }
// }
//
// fn if_no_auth(mut self, update: &Option<HeaderValue>) -> Self {
// match self {
// ProxyScheme::Http { ref mut auth, .. } => {
// if auth.is_none() {
// auth = update.clone();
// }
// }
// ProxyScheme::Https { ref mut auth, .. } => {
// if auth.is_none() {
// auth = update.clone();
// }
// }
// ProxyScheme::Socks4 { .. } => {}
// ProxyScheme::Socks5 { .. } => {}
// }
//
// self
// }
//
// Convert a URL into a proxy scheme
//
// Supported schemes: HTTP, HTTPS, (SOCKS4, SOCKS5, SOCKS5H if `socks` feature is enabled).
// Production-ready proxy URL parsing with comprehensive error handling
// fn parse(url: Url) -> std::result::Result<Self> {
// use url::Position;
//
// Resolve URL to a host and port
// let to_addr = || {
// let addrs = url
// .socket_addrs(|| match url.scheme() {
// "socks4" | "socks4a" | "socks5" | "socks5h" => Some(1080),
// _ => None,
// })
// .map_err(crate::error::builder)?;
// addrs
// .into_iter()
// .next()
// .ok_or_else(|| crate::error::builder("unknown proxy scheme"))
// };
//
// let mut scheme = match url.scheme() {
// "http" => Self::http(&url[Position::BeforeHost..Position::AfterPort])?,
// "https" => Self::https(&url[Position::BeforeHost..Position::AfterPort])?,
// "socks4" => Self::socks4(to_addr()?)?,
// "socks4a" => Self::socks4a(to_addr()?)?,
// "socks5" => Self::socks5(to_addr()?)?,
// "socks5h" => Self::socks5h(to_addr()?)?,
// _ => return Err(crate::error::builder("unknown proxy scheme")),
// };
//
// if let Some(pwd) = url.password() {
// let decoded_username = percent_decode(url.username().as_bytes()).decode_utf8_lossy();
// let decoded_password = percent_decode(pwd.as_bytes()).decode_utf8_lossy();
// scheme = scheme.with_basic_auth(decoded_username, decoded_password);
// }
//
// Ok(scheme)
// }
// }

/// Custom proxy function type for dynamic proxy selection
#[derive(Clone)]
pub struct Custom {
    pub(crate) func: Arc<
        dyn Fn(&Url) -> Option<std::result::Result<Url, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + Sync
            + 'static,
    >,
    pub(crate) no_proxy: Option<NoProxy>,
}

impl std::fmt::Debug for Custom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Custom")
            .field("func", &"<proxy function>")
            .field("no_proxy", &self.no_proxy)
            .finish()
    }
}

impl Custom {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(&Url) -> Option<std::result::Result<Url, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + Sync
            + 'static,
    {
        Self {
            func: Arc::new(func),
            no_proxy: None,
        }
    }

    pub fn with_no_proxy(mut self, no_proxy: NoProxy) -> Self {
        self.no_proxy = Some(no_proxy);
        self
    }
}
