//! Part implementation for multipart form fields
//!
//! Handles Part creation, configuration, and WASM-specific operations
//! including blob conversion and form data appending.

use std::borrow::Cow;
use std::fmt;

use http::HeaderMap;
use mime_guess::Mime;

use super::super::Body;
use super::types::{Part, PartMetadata, PartProps};

impl Part {
    /// Makes a text parameter.
    pub fn text<T>(value: T) -> Part
    where
        T: Into<Cow<'static, str>>,
    {
        let body = match value.into() {
            Cow::Borrowed(slice) => Body::from(slice),
            Cow::Owned(string) => Body::from(string),
        };
        Part::new(body)
    }

    /// Makes a new parameter from arbitrary bytes.
    pub fn bytes<T>(value: T) -> Part
    where
        T: Into<Cow<'static, [u8]>>,
    {
        let body = match value.into() {
            Cow::Borrowed(slice) => Body::from(slice),
            Cow::Owned(vec) => Body::from(vec),
        };
        Part::new(body)
    }

    /// Makes a new parameter from an arbitrary stream.
    pub fn stream<T: Into<Body>>(value: T) -> Part {
        Part::new(value.into())
    }

    fn new(value: Body) -> Part {
        Part {
            meta: PartMetadata::new(),
            value: value.into_part(),
        }
    }

    /// Tries to set the mime of this part.
    pub fn mime_str(self, mime: &str) -> std::result::Result<Part, crate::HttpError> {
        Ok(self.mime(mime.parse().map_err(crate::error::builder)?))
    }

    // Re-export when mime 0.4 is available, with split MediaType/MediaRange.
    fn mime(self, mime: Mime) -> Part {
        self.with_inner(move |inner| inner.mime(mime))
    }

    /// Sets the filename, builder style.
    pub fn file_name<T>(self, filename: T) -> Part
    where
        T: Into<Cow<'static, str>>,
    {
        self.with_inner(move |inner| inner.file_name(filename))
    }

    /// Sets custom headers for the part.
    pub fn headers(self, headers: HeaderMap) -> Part {
        self.with_inner(move |inner| inner.headers(headers))
    }

    fn with_inner<F>(self, func: F) -> Self
    where
        F: FnOnce(PartMetadata) -> PartMetadata,
    {
        Part {
            meta: func(self.meta),
            value: self.value,
        }
    }

    pub(crate) fn append_to_form(
        &self,
        name: &str,
        form: &web_sys::FormData,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let single = self
            .value
            .as_single()
            .expect("A part's body can't be multipart itself");

        let mut mime_type = self.metadata().mime.as_ref();

        // The JS fetch API doesn't support file names and mime types for strings. So we do our best
        // effort to use `append_with_str` and fallback to `append_with_blob_*` if that's not
        // possible.
        if let super::super::body::Single::Text(text) = single {
            if mime_type.is_none() || mime_type == Some(&mime_guess::mime::TEXT_PLAIN) {
                if self.metadata().file_name.is_none() {
                    return form.append_with_str(name, text);
                }
            } else {
                mime_type = Some(&mime_guess::mime::TEXT_PLAIN);
            }
        }

        let blob = self.blob(mime_type)?;

        if let Some(file_name) = &self.metadata().file_name {
            form.append_with_blob_and_filename(name, &blob, file_name)
        } else {
            form.append_with_blob(name, &blob)
        }
    }

    fn blob(&self, mime_type: Option<&Mime>) -> std::result::Result<web_sys::Blob, crate::HttpError> {
        use web_sys::Blob;
        use web_sys::BlobPropertyBag;
        let mut properties = BlobPropertyBag::new();
        if let Some(mime) = mime_type {
            properties.type_(mime.as_ref());
        }

        let js_value = self
            .value
            .as_single()
            .expect("A part's body can't be set to a multipart body")
            .to_js_value();

        let body_array = js_sys::Array::new();
        body_array.push(&js_value);

        Blob::new_with_u8_array_sequence_and_options(body_array.as_ref(), &properties)
            .map_err(crate::error::wasm)
            .map_err(crate::error::builder)
    }
}

impl fmt::Debug for Part {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut dbg = f.debug_struct("Part");
        dbg.field("value", &self.value);
        self.meta.fmt_fields(&mut dbg);
        dbg.finish()
    }
}
