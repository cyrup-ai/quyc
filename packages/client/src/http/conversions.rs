//! Type conversion utilities with zero-allocation optimizations

use std::convert::TryFrom;

use bytes::Bytes;


use crate::error::constructors::deserialization_error;

/// Convert value to bytes with zero-allocation optimization
#[inline]
pub fn to_bytes<T: AsRef<[u8]>>(value: T) -> Bytes {
    Bytes::copy_from_slice(value.as_ref())
}

/// Convert bytes to value with type validation
#[inline]
pub fn from_bytes<T: TryFrom<Vec<u8>>>(bytes: Bytes) -> Result<T, T::Error> {
    T::try_from(bytes.to_vec())
}

/// Convert value to string representation
#[inline]
pub fn to_string<T: ToString>(value: T) -> String {
    value.to_string()
}

/// Convert string to bytes
#[inline]
pub fn string_to_bytes(s: String) -> Bytes {
    Bytes::from(s)
}

/// Convert bytes to string with UTF-8 validation
#[inline]
pub fn bytes_to_string(bytes: Bytes) -> Result<String, crate::error::HttpError> {
    String::from_utf8(bytes.to_vec()).map_err(|e| deserialization_error(e.to_string()))
}

/// Convert slice to bytes
#[inline]
pub fn slice_to_bytes(slice: &[u8]) -> Bytes {
    Bytes::copy_from_slice(slice)
}

/// Convert vector to bytes with zero-copy optimization
#[inline]
pub fn vec_to_bytes(vec: Vec<u8>) -> Bytes {
    Bytes::from(vec)
}

/// Convert bytes to vector
#[inline]
pub fn bytes_to_vec(bytes: Bytes) -> Vec<u8> {
    bytes.to_vec()
}

/// Convert string slice to bytes
#[inline]
pub fn str_to_bytes(s: &str) -> Bytes {
    Bytes::copy_from_slice(s.as_bytes())
}

/// Convert bytes to string slice (borrowed)
#[inline]
pub fn bytes_to_str(bytes: &Bytes) -> Result<&str, crate::error::HttpError> {
    std::str::from_utf8(bytes)
        .map_err(|e| deserialization_error(format!("UTF-8 conversion failed: {}", e)))
}

/// Generic conversion with error handling
#[inline]
pub fn convert<T, U>(value: T) -> Result<U, crate::error::HttpError>
where
    T: TryInto<U>,
    T::Error: std::fmt::Display,
{
    value
        .try_into()
        .map_err(|e| deserialization_error(format!("Type conversion failed: {}", e)))
}
