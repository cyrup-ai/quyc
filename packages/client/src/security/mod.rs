//! Security and protection modules for HTTP/3 client
//!
//! This module provides comprehensive protection against resource exhaustion
//! attacks from malicious servers, including DNS amplification, slowloris,
//! gzip bombs, and redirect loops.
//!
//! Note: Client protection functionality has been integrated into the canonical `HttpClient`.
