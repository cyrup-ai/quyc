//! Debug trait implementations for proxy types
//!
//! Provides Debug formatting for proxy configuration types
//! with appropriate field visibility and formatting.

use std::fmt;

use super::types::{Extra, Proxy};

impl fmt::Debug for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Proxy")
            .field("intercept", &self.intercept)
            .finish()
    }
}

impl fmt::Debug for Extra {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Extra")
            .field("auth", &self.auth)
            .field("misc", &self.misc)
            .finish()
    }
}
