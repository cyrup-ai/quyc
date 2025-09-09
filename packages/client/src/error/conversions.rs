use http::header::{InvalidHeaderName, InvalidHeaderValue};

use super::types::HttpError;

impl From<InvalidHeaderName> for HttpError {
    fn from(error: InvalidHeaderName) -> Self {
        HttpError::InvalidHeader {
            message: error.to_string(),
            name: "unknown".to_string(),
            value: None,
            error_source: Some(error.to_string()),
        }
    }
}

impl From<InvalidHeaderValue> for HttpError {
    fn from(error: InvalidHeaderValue) -> Self {
        HttpError::InvalidHeader {
            message: error.to_string(),
            name: "unknown".to_string(),
            value: None,
            error_source: Some(error.to_string()),
        }
    }
}

// Removed hyper::Error conversion - no longer needed in ystream architecture

#[cfg(feature = "__rustls")]
impl From<rustls::Error> for HttpError {
    fn from(error: rustls::Error) -> Self {
        HttpError::Tls {
            message: error.to_string(),
        }
    }
}

impl From<std::io::Error> for HttpError {
    fn from(error: std::io::Error) -> Self {
        HttpError::IoError(error.to_string())
    }
}

impl From<url::ParseError> for HttpError {
    fn from(error: url::ParseError) -> Self {
        HttpError::UrlParseError {
            message: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for HttpError {
    fn from(error: serde_json::Error) -> Self {
        HttpError::Json {
            message: error.to_string(),
        }
    }
}

impl From<http::Error> for HttpError {
    fn from(error: http::Error) -> Self {
        HttpError::Request {
            message: error.to_string(),
        }
    }
}

impl From<http::uri::InvalidUri> for HttpError {
    fn from(error: http::uri::InvalidUri) -> Self {
        HttpError::InvalidUrl {
            message: error.to_string(),
        }
    }
}

impl From<http::status::InvalidStatusCode> for HttpError {
    fn from(error: http::status::InvalidStatusCode) -> Self {
        HttpError::StatusCode {
            code: 0, // Invalid status code
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for HttpError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        HttpError::Other(error.to_string())
    }
}

impl From<String> for HttpError {
    fn from(error: String) -> Self {
        HttpError::Other(error)
    }
}

impl From<&str> for HttpError {
    fn from(error: &str) -> Self {
        HttpError::Other(error.to_string())
    }
}
