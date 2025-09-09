//! Attempt and Action types for redirect handling
//!
//! Defines the Attempt struct that holds information about redirect attempts
//! and the Action types that control what happens next in the redirect chain.

use std::error::Error as StdError;
use std::fmt;

use http::StatusCode;

use crate::Url;

/// A type that holds information on the next request and previous requests
/// in redirect chain.
#[derive(Debug)]
pub struct Attempt<'a> {
    pub(crate) status: StatusCode,
    pub(crate) next: &'a Url,
    pub(crate) previous: &'a [Url],
}

/// An action to perform when a redirect status code is found.
#[derive(Debug)]
pub struct Action {
    pub(crate) inner: ActionKind,
}

#[derive(Debug)]
pub(crate) enum ActionKind {
    Follow,
    Stop,
    Error(Box<dyn StdError + Send + Sync>),
}

impl<'a> Attempt<'a> {
    /// Get the type of redirect.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the next URL to redirect to.
    pub fn url(&self) -> &Url {
        self.next
    }

    /// Get the list of previous URLs that have already been requested in this chain.
    pub fn previous(&self) -> &[Url] {
        self.previous
    }

    /// Returns an action meaning http3 should follow the next URL.
    pub fn follow(self) -> Action {
        Action {
            inner: ActionKind::Follow,
        }
    }

    /// Returns an action meaning http3 should not follow the next URL.
    ///
    /// The 30x response will be returned as the `Ok` result.
    pub fn stop(self) -> Action {
        Action {
            inner: ActionKind::Stop,
        }
    }

    /// Returns an action failing the redirect with an error.
    ///
    /// The `Error` will be returned for the result of the sent request.
    pub fn error<E: Into<Box<dyn StdError + Send + Sync>>>(self, error: E) -> Action {
        Action {
            inner: ActionKind::Error(error.into()),
        }
    }
}
