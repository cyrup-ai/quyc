#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

use super::super::multipart::Form;
use super::types::{Body, Inner, Single};

impl Body {
    /// Returns a reference to the internal data of the `Body`.
    ///
    /// `None` is returned, if the underlying data is a multipart form.
    #[inline]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match &self.inner {
            Inner::Single(single) => Some(single.as_bytes()),
            Inner::MultipartForm(_) => None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn to_js_value(&self) -> std::result::Result<JsValue, crate::HttpError> {
        match &self.inner {
            Inner::Single(single) => Ok(single.to_js_value()),
            Inner::MultipartForm(form) => {
                let form_data = form.to_form_data()?;
                let js_value: &JsValue = form_data.as_ref();
                Ok(js_value.to_owned())
            }
        }
    }

    pub(crate) fn as_single(&self) -> Option<&Single> {
        match &self.inner {
            Inner::Single(single) => Some(single),
            Inner::MultipartForm(_) => None,
        }
    }

    #[inline]
    pub(crate) fn from_form(f: Form) -> Body {
        Self {
            inner: Inner::MultipartForm(f),
        }
    }

    /// into_part turns a regular body into the body of a multipart/form-data part.
    pub(crate) fn into_part(self) -> Body {
        match self.inner {
            Inner::Single(single) => Self {
                inner: Inner::Single(single),
            },
            Inner::MultipartForm(form) => Self {
                inner: Inner::MultipartForm(form),
            },
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match &self.inner {
            Inner::Single(single) => single.is_empty(),
            Inner::MultipartForm(form) => form.is_empty(),
        }
    }

    pub(crate) fn try_clone(&self) -> Option<Body> {
        match &self.inner {
            Inner::Single(single) => Some(Self {
                inner: Inner::Single(single.clone()),
            }),
            Inner::MultipartForm(_) => None,
        }
    }
}
