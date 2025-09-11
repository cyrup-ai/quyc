//! HTTP Cookies

use std::convert::TryInto;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use bytes::Bytes;
use ystream::prelude::MessageChunk;
use http::{HeaderValue, header::SET_COOKIE};

/// Actions for a persistent cookie store providing session support.
pub trait CookieStore: Send + Sync {
    /// Store a set of Set-Cookie header values received from `url`
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url);
    /// Get any Cookie values in the store for `url`
    fn cookies(&self, url: &url::Url) -> Option<HeaderValue>;
}

/// A single HTTP cookie.
#[derive(Clone)]
pub struct Cookie<'a>(cookie::Cookie<'a>);

impl MessageChunk for Cookie<'_> {
    fn bad_chunk(error: String) -> Self {
        // Create an error cookie with invalid name/value
        let error_cookie = cookie::Cookie::build(("ERROR", error)).build();
        Cookie(error_cookie)
    }

    fn error(&self) -> Option<&str> {
        if self.0.name() == "ERROR" {
            Some(self.0.value())
        } else {
            None
        }
    }

    fn is_error(&self) -> bool {
        self.0.name() == "ERROR"
    }
}

impl Default for Cookie<'_> {
    fn default() -> Self {
        let default_cookie = cookie::Cookie::build(("default", "")).build();
        Cookie(default_cookie)
    }
}

/// A good default `CookieStore` implementation.
///
/// This is the implementation used when simply calling `cookie_store(true)`.
/// This type is exposed to allow creating one and filling it with some
/// existing cookies more easily, before creating a `Client`.
///
/// For more advanced scenarios, such as needing to serialize the store or
/// manipulate it between requests, you may refer to the
/// [http3_cookie_store crate](https://crates.io/crates/http3_cookie_store).
#[derive(Debug, Default)]
pub struct Jar(Arc<RwLock<Option<cookie_store::CookieStore>>>);

// ===== impl Cookie =====

impl<'a> Cookie<'a> {
    fn parse(value: &'a HeaderValue) -> Cookie<'a> {
        match std::str::from_utf8(value.as_bytes())
            .map_err(cookie::ParseError::from)
            .and_then(cookie::Cookie::parse)
        {
            Ok(cookie) => Cookie(cookie),
            Err(e) => Cookie::bad_chunk(format!("Cookie parse error: {e}")),
        }
    }

    /// The name of the cookie.
    #[must_use] 
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// The value of the cookie.
    #[must_use] 
    pub fn value(&self) -> &str {
        self.0.value()
    }

    /// Returns true if the '`HttpOnly`' directive is enabled.
    #[must_use] 
    pub fn http_only(&self) -> bool {
        self.0.http_only().unwrap_or(false)
    }

    /// Returns true if the 'Secure' directive is enabled.
    #[must_use] 
    pub fn secure(&self) -> bool {
        self.0.secure().unwrap_or(false)
    }

    /// Returns true if  '`SameSite`' directive is 'Lax'.
    #[must_use] 
    pub fn same_site_lax(&self) -> bool {
        self.0.same_site() == Some(cookie::SameSite::Lax)
    }

    /// Returns true if  '`SameSite`' directive is 'Strict'.
    #[must_use] 
    pub fn same_site_strict(&self) -> bool {
        self.0.same_site() == Some(cookie::SameSite::Strict)
    }

    /// Returns the path directive of the cookie, if set.
    #[must_use] 
    pub fn path(&self) -> Option<&str> {
        self.0.path()
    }

    /// Returns the domain directive of the cookie, if set.
    #[must_use] 
    pub fn domain(&self) -> Option<&str> {
        self.0.domain()
    }

    /// Get the Max-Age information.
    #[must_use] 
    pub fn max_age(&self) -> Option<std::time::Duration> {
        self.0.max_age().and_then(|d| d.try_into().ok())
    }

    /// The cookie expiration time.
    #[must_use] 
    pub fn expires(&self) -> Option<SystemTime> {
        match self.0.expires() {
            Some(cookie::Expiration::DateTime(offset)) => Some(SystemTime::from(offset)),
            None | Some(cookie::Expiration::Session) => None,
        }
    }
}

impl fmt::Debug for Cookie<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub(crate) fn extract_response_cookie_headers(
    headers: &hyper::HeaderMap,
) -> impl Iterator<Item = &HeaderValue> + '_ {
    headers.get_all(SET_COOKIE).iter()
}

pub(crate) fn extract_response_cookies(
    headers: &hyper::HeaderMap,
) -> impl Iterator<Item = Result<Cookie<'_>, CookieParseError>> + '_ {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .map(|value_str| match cookie::Cookie::parse(value_str) {
            Ok(cookie) => Ok(Cookie(cookie)),
            Err(e) => Err(CookieParseError(e)),
        })
}

/// Error representing a parse failure of a 'Set-Cookie' header.
pub(crate) struct CookieParseError(cookie::ParseError);

impl fmt::Debug for CookieParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for CookieParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for CookieParseError {}

// ===== impl Jar =====

impl Jar {
    /// Add a cookie to this jar.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::{cookie::Jar, Url};
    ///
    /// let cookie = "foo=bar; Domain=yolo.local";
    /// let url = "https://yolo.local".parse::<Url>()?;
    ///
    /// let jar = Jar::default();
    /// jar.add_cookie_str(cookie, &url);
    ///
    /// // and now add to a `ClientBuilder`?
    /// ```
    pub fn add_cookie_str(&self, cookie: &str, url: &url::Url) {
        let cookies = cookie::Cookie::parse(cookie)
            .ok()
            .map(cookie::Cookie::into_owned)
            .into_iter();
        if let Ok(mut store_guard) = self.0.write()
            && let Some(ref mut store) = store_guard.as_mut() {
                store.store_response_cookies(cookies, url);
            }
    }
}

impl Clone for Jar {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl CookieStore for Jar {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        let cookies: Vec<_> = cookie_headers
            .filter_map(|val| {
                cookie::Cookie::parse(val.to_str().unwrap_or(""))
                    .map(cookie::Cookie::into_owned)
                    .ok()
            })
            .collect();

        if let Ok(mut store_guard) = self.0.write()
            && let Some(ref mut store) = store_guard.as_mut() {
                store.store_response_cookies(cookies.into_iter(), url);
            }
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let s = match self.0.read() {
            Ok(store_guard) => {
                if let Some(store) = store_guard.as_ref() {
                    store
                        .get_request_values(url)
                        .map(|(name, value)| format!("{name}={value}"))
                        .collect::<Vec<_>>()
                        .join("; ")
                } else {
                    return None;
                }
            }
            Err(_) => return None,
        };

        if s.is_empty() {
            return None;
        }

        HeaderValue::from_maybe_shared(Bytes::from(s)).ok()
    }
}
