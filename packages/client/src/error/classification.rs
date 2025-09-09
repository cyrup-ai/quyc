use std::error::Error as StdError;
use std::io;

use super::helpers::TimedOut;
use super::types::{Error, Kind};

impl Error {
    /// Returns true if the error is from a type Builder.
    pub fn is_builder(&self) -> bool {
        matches!(self.inner.kind, Kind::Builder)
    }

    /// Returns true if the error is from a `RedirectPolicy`.
    pub fn is_redirect(&self) -> bool {
        matches!(self.inner.kind, Kind::Redirect)
    }

    /// Returns true if the error is from `Response::error_for_status`.
    pub fn is_status(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            matches!(self.inner.kind, Kind::Status(_, _))
        }
        #[cfg(target_arch = "wasm32")]
        {
            matches!(self.inner.kind, Kind::Status(_))
        }
    }

    /// Returns true if the error is related to a timeout.
    pub fn is_timeout(&self) -> bool {
        let mut source = self.source();

        while let Some(err) = source {
            if err.is::<TimedOut>() {
                return true;
            }
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(hyper_err) = err.downcast_ref::<hyper::Error>() {
                if hyper_err.is_timeout() {
                    return true;
                }
            }
            if let Some(io) = err.downcast_ref::<io::Error>() {
                if io.kind() == io::ErrorKind::TimedOut {
                    return true;
                }
            }
            source = err.source();
        }

        false
    }

    /// Returns true if the error is related to the request
    pub fn is_request(&self) -> bool {
        matches!(self.inner.kind, Kind::Request)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Returns true if the error is related to connect
    pub fn is_connect(&self) -> bool {
        let mut source = self.source();

        while let Some(err) = source {
            // Note: Removed legacy client error check since we use pure AsyncStream architecture
            // Connection errors will be handled by other error types

            source = err.source();
        }

        false
    }

    /// Returns true if the error is related to the request or response body
    pub fn is_body(&self) -> bool {
        matches!(self.inner.kind, Kind::Body)
    }

    /// Returns true if the error is related to decoding the response's body
    pub fn is_decode(&self) -> bool {
        matches!(self.inner.kind, Kind::Decode)
    }

    /// Returns the status code, if the error was generated from a response.
    pub fn status(&self) -> Option<crate::StatusCode> {
        match self.inner.kind {
            #[cfg(target_arch = "wasm32")]
            Kind::Status(code) => Some(code),
            #[cfg(not(target_arch = "wasm32"))]
            Kind::Status(code, _) => Some(code),
            _ => None,
        }
    }
}
