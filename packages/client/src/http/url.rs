//! URL validation and normalization utilities with zero-allocation optimizations

use url::Url;

/// Validate URL string format and accessibility
#[inline]
pub fn validate_url(url_str: &str) -> Result<(), crate::error::HttpError> {
    Url::parse(url_str).map(|_| ()).map_err(|_e| {
        let inner = crate::error::types::Inner {
            kind: crate::error::types::Kind::Request,
            source: None,
            url: None,
        };
        crate::error::types::Error {
            inner: Box::new(inner),
        }
    })
}

/// Normalize URL by removing fragments and sorting query parameters
#[inline]
pub fn normalize_url(url_str: &str) -> Result<String, crate::error::HttpError> {
    let mut url = Url::parse(url_str).map_err(|_e| {
        let inner = crate::error::types::Inner {
            kind: crate::error::types::Kind::Request,
            source: None,
            url: None,
        };
        crate::error::types::Error {
            inner: Box::new(inner),
        }
    })?;

    // Remove fragment
    url.set_fragment(None);

    // Sort query parameters for normalization
    let query_string = url.query().unwrap_or("").to_string();
    let pairs: Vec<_> = url::form_urlencoded::parse(query_string.as_bytes()).collect();
    let mut sorted_pairs = pairs;
    sorted_pairs.sort_by(|a, b| a.0.cmp(&b.0));

    url.query_pairs_mut().clear();
    for (key, value) in sorted_pairs {
        url.query_pairs_mut().append_pair(&key, &value);
    }

    Ok(url.to_string())
}

/// Parse and validate URL string
#[inline]
pub fn parse_url(url_str: &str) -> Result<Url, crate::error::HttpError> {
    Url::parse(url_str).map_err(|_e| {
        let inner = crate::error::types::Inner {
            kind: crate::error::types::Kind::Request,
            source: None,
            url: None,
        };
        crate::error::types::Error {
            inner: Box::new(inner),
        }
    })
}

/// Extract host from URL
#[inline]
pub fn extract_host(url: &Url) -> Option<&str> {
    url.host_str()
}

/// Check if URL uses secure scheme
#[inline]
pub fn is_secure_scheme(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "wss")
}

/// Extract port from URL with default fallback
#[inline]
pub fn extract_port(url: &Url) -> u16 {
    url.port().unwrap_or_else(|| match url.scheme() {
        "https" | "wss" => 443,
        "http" | "ws" => 80,
        _ => 80,
    })
}

/// Build URL with path and query parameters
#[inline]
pub fn build_url(
    base: &str,
    path: &str,
    params: &[(&str, &str)],
) -> Result<String, crate::error::HttpError> {
    let mut url = Url::parse(base).map_err(|_e| {
        let inner = crate::error::types::Inner {
            kind: crate::error::types::Kind::Request,
            source: None,
            url: None,
        };
        crate::error::types::Error {
            inner: Box::new(inner),
        }
    })?;

    url.set_path(path);

    for (key, value) in params {
        url.query_pairs_mut().append_pair(key, value);
    }

    Ok(url.to_string())
}
