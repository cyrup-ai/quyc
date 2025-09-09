//! Telemetry type definitions

use std::time::Duration;

/// Telemetry event types
#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    RequestStarted {
        request_id: String,
        url: String,
    },
    RequestCompleted {
        request_id: String,
        status: u16,
        duration: Duration,
    },
    RequestFailed {
        request_id: String,
        error: String,
    },
    ConnectionEstablished {
        connection_id: String,
        protocol: String,
    },
    ConnectionClosed {
        connection_id: String,
    },
}

/// Telemetry configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub sample_rate: f64,
    pub buffer_size: usize,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_rate: 1.0,
            buffer_size: 1000,
        }
    }
}
