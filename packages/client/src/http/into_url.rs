#![allow(dead_code)]

use url::Url;


use crate::error::constructors::url;

/// Conditional compilation macro for hyper-specific code
macro_rules! if_hyper {
    ($($item:item)*) => {
        $(
            #[cfg(not(target_arch = "wasm32"))]
            $item
        )*
    };
}

/// A trait to try to convert some type into a `Url`.
///
/// This trait is "sealed", such that only types within http3 can
/// implement it.
pub trait IntoUrl: IntoUrlSealed {}

impl IntoUrl for Url {}
impl IntoUrl for String {}
impl IntoUrl for &str {}
impl IntoUrl for &String {}

pub trait IntoUrlSealed {
    // Besides parsing as a valid `Url`, the `Url` must be a valid
    // `http::Uri`, in that it makes sense to use in a network request.
    ///
    /// # Errors
    /// 
    /// Returns `HttpError` if:
    /// - URL string is malformed or contains invalid syntax
    /// - URL scheme is not supported for network requests (must be http/https)
    /// - URL lacks required components (host) for network operations
    /// - URL contains characters or structures incompatible with HTTP requests
    fn into_url(self) -> std::result::Result<Url, crate::HttpError>;

    fn as_str(&self) -> &str;
}

impl IntoUrlSealed for Url {
    fn into_url(self) -> std::result::Result<Url, crate::HttpError> {
        // With blob url the `self.has_host()` check is always false, so we
        // remove the `blob:` scheme and check again if the url is valid.
        #[cfg(target_arch = "wasm32")]
        if self.scheme() == "blob"
            && self.path().starts_with("http") // Check if the path starts with http or https to avoid validating a `blob:blob:...` url.
            && self.as_str()[5..].into_url().is_ok()
        {
            return Ok(self);
        }

        if self.has_host() {
            Ok(self)
        } else {
            Err(url(format!("Bad scheme in URL: {self}")))
        }
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl IntoUrlSealed for &str {
    fn into_url(self) -> std::result::Result<Url, crate::HttpError> {
        Url::parse(self).map_err(crate::error::builder)?.into_url()
    }

    fn as_str(&self) -> &str {
        self
    }
}

impl IntoUrlSealed for &String {
    fn into_url(self) -> std::result::Result<Url, crate::HttpError> {
        (&**self).into_url()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl IntoUrlSealed for String {
    fn into_url(self) -> std::result::Result<Url, crate::HttpError> {
        (&*self).into_url()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

if_hyper! {
    pub(crate) fn try_uri(url: &Url) -> std::result::Result<http::Uri, crate::HttpError> {
        url.as_str()
            .parse()
            .map_err(|_| {
                let inner = crate::error::types::Inner {
                    kind: crate::error::types::Kind::Request,
                    source: None,
                    url: None,
                };
                crate::HttpError { inner: Box::new(inner) }
            })
    }
}
