//! Core types for multipart/form-data handling
//!
//! Defines the fundamental types used in multipart form construction
//! including Form, Part, FormParts, and PartMetadata structures.

use std::borrow::Cow;
use std::fmt;

use http::HeaderMap;
use mime_guess::Mime;

use super::super::Body;

/// An async multipart/form-data request.
pub struct Form {
    pub(crate) inner: FormParts<Part>,
}

impl Form {
    pub(crate) fn is_empty(&self) -> bool {
        self.inner.fields.is_empty()
    }
}

/// A field in a multipart form.
pub struct Part {
    pub(crate) meta: PartMetadata,
    pub(crate) value: Body,
}

pub(crate) struct FormParts<P> {
    pub(crate) fields: Vec<(Cow<'static, str>, P)>,
}

pub(crate) struct PartMetadata {
    pub(crate) mime: Option<Mime>,
    pub(crate) file_name: Option<Cow<'static, str>>,
    pub(crate) headers: HeaderMap,
}

pub(crate) trait PartProps {
    fn metadata(&self) -> &PartMetadata;
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl Form {
    /// Creates a new async Form without any content.
    pub fn new() -> Form {
        Form {
            inner: FormParts::new(),
        }
    }
}

impl<P: PartProps> FormParts<P> {
    pub(crate) fn new() -> Self {
        FormParts { fields: Vec::new() }
    }

    /// Adds a customized Part.
    pub(crate) fn part<T>(mut self, name: T, part: P) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.fields.push((name.into(), part));
        self
    }
}

impl PartMetadata {
    pub(crate) fn new() -> Self {
        PartMetadata {
            mime: None,
            file_name: None,
            headers: HeaderMap::default(),
        }
    }
}

impl PartProps for Part {
    fn metadata(&self) -> &PartMetadata {
        &self.meta
    }
}
