//! Tower integration for redirect handling
//!
//! Provides TowerRedirectPolicy for integrating redirect handling
//! with Tower-based HTTP clients and middleware.

use std::sync::Arc;

use http::{HeaderMap, HeaderValue, StatusCode, Uri};

use super::attempt::ActionKind;
use super::headers::{make_referer, remove_sensitive_headers};
use super::policy::Policy;
use crate::Url;

#[derive(Clone)]
pub(crate) struct TowerRedirectPolicy {
    policy: Arc<Policy>,
    referer: bool,
    urls: Vec<Url>,
    https_only: bool,
}

impl TowerRedirectPolicy {
    pub(crate) fn new(policy: Policy) -> Self {
        Self {
            policy: Arc::new(policy),
            referer: false,
            urls: Vec::new(),
            https_only: false,
        }
    }

    pub(crate) fn with_referer(&mut self, referer: bool) -> &mut Self {
        self.referer = referer;
        self
    }

    pub(crate) fn with_https_only(&mut self, https_only: bool) -> &mut Self {
        self.https_only = https_only;
        self
    }

    // Handle redirect attempt
    pub(crate) fn handle_redirect(
        &mut self,
        status: StatusCode,
        location: &HeaderValue,
        previous: &Uri,
    ) -> Result<ActionKind, crate::Error> {
        let previous_url = match Url::parse(&previous.to_string()) {
            Ok(url) => url,
            Err(e) => return Err(crate::HttpError::builder(e.to_string())),
        };

        let next_url = match location.to_str().ok().and_then(|s| Url::parse(s).ok()) {
            Some(url) => url,
            None => return Err(crate::HttpError::builder("Invalid redirect location")),
        };

        self.urls.push(previous_url.clone());

        match self.policy.check(status, &next_url, &self.urls) {
            ActionKind::Follow => {
                if next_url.scheme() != "http" && next_url.scheme() != "https" {
                    return Err(crate::HttpError::url(format!(
                        "Bad scheme in URL: {}",
                        next_url
                    )));
                }

                if self.https_only && next_url.scheme() != "https" {
                    return Err(crate::HttpError::redirect(format!(
                        "HTTPS required but got: {}",
                        next_url
                    )));
                }
                Ok(ActionKind::Follow)
            }
            ActionKind::Stop => Ok(ActionKind::Stop),
            ActionKind::Error(e) => {
                Err(crate::HttpError::redirect(format!("Redirect error: {}", e)))
            }
        }
    }

    // Update headers for redirect request
    pub(crate) fn update_headers(&mut self, headers: &mut HeaderMap, uri: &Uri) {
        use http::header::REFERER;

        if let Ok(next_url) = Url::parse(&uri.to_string()) {
            remove_sensitive_headers(headers, &next_url, &self.urls);
            if self.referer {
                if let Some(previous_url) = self.urls.last() {
                    if let Some(v) = make_referer(&next_url, previous_url) {
                        headers.insert(REFERER, v);
                    }
                }
            }
        }
    }
}
