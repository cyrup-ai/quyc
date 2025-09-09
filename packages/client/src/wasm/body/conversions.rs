use std::borrow::Cow;

use bytes::Bytes;

use super::types::{Body, Inner, Single};

impl From<Bytes> for Body {
    #[inline]
    fn from(bytes: Bytes) -> Body {
        Body {
            inner: Inner::Single(Single::Bytes(bytes)),
        }
    }
}

impl From<Vec<u8>> for Body {
    #[inline]
    fn from(vec: Vec<u8>) -> Body {
        Body {
            inner: Inner::Single(Single::Bytes(vec.into())),
        }
    }
}

impl From<&'static [u8]> for Body {
    #[inline]
    fn from(s: &'static [u8]) -> Body {
        Body {
            inner: Inner::Single(Single::Bytes(Bytes::from_static(s))),
        }
    }
}

impl From<String> for Body {
    #[inline]
    fn from(s: String) -> Body {
        Body {
            inner: Inner::Single(Single::Text(s.into())),
        }
    }
}

impl From<&'static str> for Body {
    #[inline]
    fn from(s: &'static str) -> Body {
        Body {
            inner: Inner::Single(Single::Text(s.into())),
        }
    }
}
