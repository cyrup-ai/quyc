//! Build logic for ConnectorBuilder
//!
//! Provides the final build method to create configured connectors.

use hyper_util::client::legacy::connect::HttpConnector;

use super::super::service::ConnectorService;
use super::super::types::{Connector, ConnectorKind};
use super::types::ConnectorBuilder;
use crate::error::BoxError;

impl ConnectorBuilder {
    /// Build the connector with configured settings
    pub fn build(self) -> Result<Connector, BoxError> {
        let service = ConnectorService::new(
            self.http_connector.unwrap_or_else(|| HttpConnector::new()),
            #[cfg(feature = "default-tls")]
            self.tls_connector,
            #[cfg(feature = "__rustls")]
            self.rustls_config,
            self.proxies,
            self.user_agent,
            self.local_address,
            self.interface,
            self.nodelay,
            self.connect_timeout,
            self.happy_eyeballs_timeout,
            self.tls_info,
        )?;

        let kind = {
            #[cfg(feature = "__tls")]
            {
                ConnectorKind::BuiltDefault(service)
            }
            #[cfg(not(feature = "__tls"))]
            {
                ConnectorKind::BuiltHttp(service)
            }
        };

        Ok(Connector { inner: kind })
    }
}
