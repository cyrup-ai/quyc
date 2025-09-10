//! Escape utilities for debug output

#![allow(dead_code)]

use std::fmt;

pub(crate) struct Escape<'a>(&'a [u8]);

#[cfg(not(target_arch = "wasm32"))]
impl<'a> Escape<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Escape(bytes)
    }
}

impl fmt::Debug for Escape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b\"{}\"", self)?;
        Ok(())
    }
}

/// HTML escape function for compatibility
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// URL decode function for compatibility
pub fn url_decode(input: &str) -> Result<String, std::fmt::Error> {
    urlencoding::decode(input)
        .map(std::borrow::Cow::into_owned)
        .map_err(|_| std::fmt::Error)
}

/// URL encode function for compatibility
pub fn url_encode(input: &str) -> String {
    urlencoding::encode(input).into_owned()
}

impl fmt::Display for Escape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &c in self.0 {
            // https://doc.rust-lang.org/reference.html#byte-escapes
            if c == b'\n' {
                write!(f, "\\n")?;
            } else if c == b'\r' {
                write!(f, "\\r")?;
            } else if c == b'\t' {
                write!(f, "\\t")?;
            } else if c == b'\\' || c == b'"' {
                write!(f, "\\{}", c as char)?;
            } else if c == b'\0' {
                write!(f, "\\0")?;
            // ASCII printable
            } else if c >= 0x20 && c < 0x7f {
                write!(f, "{}", c as char)?;
            } else {
                write!(f, "\\x{c:02x}")?;
            }
        }
        Ok(())
    }
}
