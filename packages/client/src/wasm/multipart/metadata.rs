//! Metadata and utility implementations for multipart parts
//!
//! Handles PartMetadata configuration and debug formatting utilities
//! for multipart form components.

use std::borrow::Cow;
use std::fmt;

use http::HeaderMap;
use mime_guess::Mime;

use super::types::PartMetadata;

impl PartMetadata {
    pub(crate) fn mime(mut self, mime: Mime) -> Self {
        self.mime = Some(mime);
        self
    }

    pub(crate) fn file_name<T>(mut self, filename: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.file_name = Some(filename.into());
        self
    }

    pub(crate) fn headers<T>(mut self, headers: T) -> Self
    where
        T: Into<HeaderMap>,
    {
        self.headers = headers.into();
        self
    }
}

impl PartMetadata {
    pub(crate) fn fmt_fields<'f, 'fa, 'fb>(
        &self,
        debug_struct: &'f mut fmt::DebugStruct<'fa, 'fb>,
    ) -> &'f mut fmt::DebugStruct<'fa, 'fb> {
        debug_struct
            .field("mime", &self.mime)
            .field("file_name", &self.file_name)
            .field("headers", &self.headers)
    }
}
