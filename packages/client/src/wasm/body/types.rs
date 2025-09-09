use std::{borrow::Cow, fmt};

/// dox
use bytes::Bytes;
#[cfg(target_arch = "wasm32")]
use js_sys::Uint8Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use super::super::multipart::Form;

/// The body of a `Request`.
///
/// In most cases, this is not needed directly, as the
/// [`RequestBuilder.body`][builder] method uses `Into<Body>`, which allows
/// passing many things (like a string or vector of bytes).
///
/// [builder]: ./struct.RequestBuilder.html#method.body
pub struct Body {
    pub(crate) inner: Inner,
}

pub(crate) enum Inner {
    Single(Single),
    /// MultipartForm holds a multipart/form-data body.
    MultipartForm(Form),
}

#[derive(Clone)]
pub(crate) enum Single {
    Bytes(Bytes),
    Text(Cow<'static, str>),
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl Default for Body {
    fn default() -> Body {
        Body {
            inner: Inner::Single(Single::Bytes(Bytes::new())),
        }
    }
}
