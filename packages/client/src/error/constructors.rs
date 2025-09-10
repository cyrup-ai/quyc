use super::types::{Error, Kind};

pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Creates an `Error` for a builder error.
pub fn builder<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Builder).with(e.into())
}

/// Creates an `Error` for a request error.
pub fn request<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

/// Creates an `Error` for a redirect error.
pub fn redirect<E: Into<BoxError>>(e: E, url: crate::Url) -> Error {
    Error::new(Kind::Redirect).with(e.into()).with_url(url)
}

/// Creates an `Error` for a body error.
pub fn body<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Body).with(e.into())
}

/// Creates an `Error` for a decode error.
pub fn decode<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Decode).with(e.into())
}

/// Creates an `Error` for an upgrade error.
pub fn upgrade<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Upgrade).with(e.into())
}

// Additional constructors needed by other modules
pub fn url_invalid_uri(url: crate::Url) -> Error {
    Error::new(Kind::Builder)
        .with(super::helpers::BadScheme)
        .with_url(url)
}

pub fn url_bad_scheme(url: crate::Url) -> Error {
    Error::new(Kind::Builder)
        .with(super::helpers::BadScheme)
        .with_url(url)
}

pub fn status_code(
    url: crate::Url,
    status: crate::StatusCode,
    #[cfg(not(target_arch = "wasm32"))] reason: Option<hyper::ext::ReasonPhrase>,
) -> Error {
    Error::new(Kind::Status(
        status,
        #[cfg(not(target_arch = "wasm32"))]
        reason,
    ))
    .with_url(url)
}

// Additional error constructors needed by the codebase
pub fn configuration<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Builder).with(e.into())
}

pub fn invalid_header<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn invalid_url<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Builder).with(e.into())
}

pub fn url<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Builder).with(e.into())
}

pub fn deserialization_error<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Decode).with(e.into())
}

pub fn network_error<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn timeout<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn http_status<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn tls_error<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn serialization_error<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn invalid_response<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn client_error<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn url_parse_error<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Builder).with(e.into())
}

pub fn download_interrupted<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn invalid_content_length<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn chunk_processing_error<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

pub fn generic<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

/// Creates an `Error` for security validation failures.
pub fn security_error<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request).with(e.into())
}

/// Creates an `Error` for connection/connector creation failures.
pub fn connector_error<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Connect).with(e.into())
}
