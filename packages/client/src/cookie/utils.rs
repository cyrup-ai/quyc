//! Cookie handling utilities

use std::collections::HashMap;

use http::HeaderMap;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn add_cookie_header(
    headers: &mut HeaderMap,
    cookie_store: &dyn super::CookieStore,
    url: &url::Url,
) {
    if let Some(header) = cookie_store.cookies(url) {
        headers.insert(http::header::COOKIE, header);
    }
}

/// Format cookie key-value pairs into a cookie header string
#[must_use] 
pub fn format_cookie<S: ::std::hash::BuildHasher>(cookies: &HashMap<String, String, S>) -> String {
    cookies
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Parse cookie header string into key-value pairs
#[must_use] 
pub fn parse_cookie(cookie_header: &str) -> HashMap<String, String> {
    let mut cookies = HashMap::new();

    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(eq_pos) = pair.find('=') {
            let key = pair[..eq_pos].trim().to_string();
            let value = pair[eq_pos + 1..].trim().to_string();
            cookies.insert(key, value);
        }
    }

    cookies
}

/// Validate cookie name and value according to RFC 6265
/// 
/// # Errors
/// 
/// Returns an error message as a `String` if validation fails:
/// - If the cookie name is empty
/// - If the cookie name contains control characters or RFC 6265 separator characters: `(),/<>@[\\]{}`
/// - If the cookie value contains control characters (except tab character)
pub fn validate_cookie(name: &str, value: &str) -> Result<(), String> {
    // Basic validation - cookie name cannot be empty
    if name.is_empty() {
        return Err("Cookie name cannot be empty".to_string());
    }

    // Cookie name cannot contain control characters or separators
    for ch in name.chars() {
        if ch.is_control() || "(),/<>@[\\]{}".contains(ch) {
            return Err(format!("Invalid character '{ch}' in cookie name"));
        }
    }

    // Cookie value cannot contain control characters (except tab)
    for ch in value.chars() {
        if ch.is_control() && ch != '\t' {
            return Err(format!("Invalid character '{ch}' in cookie value"));
        }
    }

    Ok(())
}
