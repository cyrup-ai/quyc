use std::error::Error as StdError;
use std::fmt;

/// A Result alias where the Err case is `hyper::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents errors that can occur handling HTTP streams.
#[derive(Clone)]
pub struct Error {
    pub inner: Box<Inner>,
}

pub struct Inner {
    pub kind: Kind,
    pub source: Option<Box<dyn StdError + Send + Sync>>,
    pub url: Option<url::Url>,
}

impl Clone for Inner {
    fn clone(&self) -> Self {
        Inner {
            kind: self.kind.clone(),
            source: None, // Cannot clone trait objects, so we lose the source
            url: self.url.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Kind {
    Builder,
    Request,
    Redirect,
    #[cfg(not(target_arch = "wasm32"))]
    Status(crate::StatusCode, Option<hyper::ext::ReasonPhrase>),
    #[cfg(target_arch = "wasm32")]
    Status(crate::StatusCode),
    Body,
    Decode,
    Upgrade,
    /// Connection/connector creation failures
    Connect,
    /// Request or response timeout
    Timeout,
    /// Payload exceeds maximum size limit
    PayloadTooLarge,
    /// Stream processing error
    Stream,
}

impl Error {
    pub fn new(kind: Kind) -> Error {
        Error {
            inner: Box::new(Inner { kind, source: None, url: None }),
        }
    }

    #[must_use = "Error builder methods return a new Error and should be used"]
    pub fn with<E: Into<Box<dyn StdError + Send + Sync>>>(mut self, source: E) -> Error {
        self.inner.source = Some(source.into());
        self
    }

    #[must_use] 
    pub fn with_url(mut self, url: url::Url) -> Self {
        // Store URL context for better error reporting and debugging
        let mut inner = (*self.inner).clone();
        inner.url = Some(url);
        self.inner = Box::new(inner);
        self
    }

    #[allow(dead_code)]
    pub(super) fn kind(&self) -> &Kind {
        &self.inner.kind
    }
    
    /// Get the URL associated with this error, if any
    #[must_use] 
    pub fn url(&self) -> Option<&url::Url> {
        self.inner.url.as_ref()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("quyc::Error");

        f.field("kind", &self.inner.kind);

        if let Some(ref source) = self.inner.source {
            f.field("source", source);
        }

        if let Some(ref url) = self.inner.url {
            f.field("url", url);
        }

        f.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner.kind {
            Kind::Builder => f.write_str("builder error"),
            Kind::Request => f.write_str("error sending request"),
            Kind::Body => f.write_str("request or response body error"),
            Kind::Decode => f.write_str("error decoding response body"),
            Kind::Redirect => f.write_str("error following redirect"),
            Kind::Upgrade => f.write_str("error upgrading connection"),
            Kind::Connect => f.write_str("connection/connector creation error"),
            Kind::Timeout => f.write_str("request timeout"),
            Kind::PayloadTooLarge => f.write_str("payload too large"),
            Kind::Stream => f.write_str("stream processing error"),
            #[cfg(target_arch = "wasm32")]
            Kind::Status(ref code) => {
                let prefix = if code.is_client_error() {
                    "HTTP status client error"
                } else {
                    debug_assert!(code.is_server_error());
                    "HTTP status server error"
                };
                write!(f, "{prefix} ({code})")
            }
            #[cfg(not(target_arch = "wasm32"))]
            Kind::Status(code, reason) => {
                let prefix = if code.is_client_error() {
                    "HTTP status client error"
                } else {
                    debug_assert!(code.is_server_error());
                    "HTTP status server error"
                };
                if let Some(reason) = reason {
                    write!(f, "{prefix} ({} {:?})", code.as_str(), reason)
                } else {
                    write!(f, "{prefix} ({code})")
                }
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.inner
            .source
            .as_ref()
            .map(|err| &**err as &(dyn StdError + 'static))
    }
}

use ystream::prelude::MessageChunk;

impl MessageChunk for Error {
    fn bad_chunk(error: String) -> Self {
        Error::new(Kind::Request).with(std::io::Error::other(error))
    }

    fn is_error(&self) -> bool {
        true // Error types are always errors
    }

    fn error(&self) -> Option<&str> {
        Some("HTTP error occurred")
    }
}
