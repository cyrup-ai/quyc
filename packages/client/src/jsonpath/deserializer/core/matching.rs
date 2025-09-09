//! JSONPath matching and deserialization logic
//!
//! Contains methods for JSONPath expression matching and object deserialization
//! during streaming JSON processing.

use serde::de::DeserializeOwned;

use super::types::JsonPathDeserializer;

impl<'a, T> JsonPathDeserializer<'a, T> where T: DeserializeOwned {}
