//! `JSONPath` matching and deserialization logic
//!
//! Contains methods for `JSONPath` expression matching and object deserialization
//! during streaming JSON processing.

use serde::de::DeserializeOwned;

use super::types::JsonPathDeserializer;

impl<T> JsonPathDeserializer<'_, T> where T: DeserializeOwned {}
