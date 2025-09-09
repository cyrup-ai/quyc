//! Header manipulation utilities for redirects
//!
//! Functions for handling sensitive headers during redirects and
//! creating appropriate referer headers.

use http::{HeaderMap, HeaderValue};

use crate::Url;
use http::header::{AUTHORIZATION, COOKIE, PROXY_AUTHORIZATION, REFERER, WWW_AUTHENTICATE};

/// Remove sensitive headers when redirecting across different hosts/ports
pub(crate) fn remove_sensitive_headers(headers: &mut HeaderMap, next: &Url, previous: &[Url]) {
    if let Some(previous) = previous.last() {
        let cross_host = next.host_str() != previous.host_str()
            || next.port_or_known_default() != previous.port_or_known_default();
        if cross_host {
            headers.remove(AUTHORIZATION);
            headers.remove(COOKIE);
            headers.remove("cookie2");
            headers.remove(PROXY_AUTHORIZATION);
            headers.remove(WWW_AUTHENTICATE);
        }
    }
}

/// Create a referer header value from previous URL, handling HTTPS->HTTP downgrade
pub(crate) fn make_referer(next: &Url, previous: &Url) -> Option<HeaderValue> {
    if next.scheme() == "http" && previous.scheme() == "https" {
        return None;
    }

    let mut referer = previous.clone();
    let _ = referer.set_username("");
    let _ = referer.set_password(None);
    referer.set_fragment(None);
    referer.as_str().parse().ok()
}
