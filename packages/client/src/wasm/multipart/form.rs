//! Form implementation for multipart/form-data
//!
//! Handles Form construction, field addition, and conversion to WASM FormData
//! for use with the fetch API in browser environments.

use std::borrow::Cow;
use std::fmt;

use web_sys::FormData;

use super::types::{Form, FormParts, Part, PartProps};

impl Form {
    /// Add a data field with supplied name and value.
    ///
    /// # Examples
    ///
    /// ```
    /// let form = crate::client::HttpClientipart::Form::new()
    ///     .text("username", "seanmonstar")
    ///     .text("password", "secret");
    /// ```
    pub fn text<T, U>(self, name: T, value: U) -> Form
    where
        T: Into<Cow<'static, str>>,
        U: Into<Cow<'static, str>>,
    {
        self.part(name, Part::text(value))
    }

    /// Adds a customized Part.
    pub fn part<T>(self, name: T, part: Part) -> Form
    where
        T: Into<Cow<'static, str>>,
    {
        self.with_inner(move |inner| inner.part(name, part))
    }

    fn with_inner<F>(self, func: F) -> Self
    where
        F: FnOnce(FormParts<Part>) -> FormParts<Part>,
    {
        Form {
            inner: func(self.inner),
        }
    }

    pub(crate) fn to_form_data(&self) -> std::result::Result<FormData, crate::HttpError> {
        let form = FormData::new()
            .map_err(crate::error::wasm)
            .map_err(crate::error::builder)?;

        for (name, part) in self.inner.fields.iter() {
            part.append_to_form(name, &form)
                .map_err(crate::error::wasm)
                .map_err(crate::error::builder)?;
        }
        Ok(form)
    }
}

impl fmt::Debug for Form {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt_fields("Form", f)
    }
}

impl<P: fmt::Debug> FormParts<P> {
    pub(crate) fn fmt_fields(&self, ty_name: &'static str, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(ty_name)
            .field("parts", &self.fields)
            .finish()
    }
}
