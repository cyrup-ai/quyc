//! Auto Protocol Strategy with Fallback Support
//!
//! Automatically selects the best protocol and falls back to alternatives on failure.

use std::sync::Arc;
use http::Version;

use crate::protocols::strategy_trait::ProtocolStrategy;
use crate::protocols::h2::strategy::H2Strategy;
use crate::protocols::h3::strategy::H3Strategy;
use crate::protocols::strategy::ProtocolConfigs;
use crate::protocols::core::HttpVersion;
use crate::protocols::intelligence::{ProtocolIntelligence, AltSvcEndpoint};
use crate::http::{HttpRequest, HttpResponse};

/// Auto-selecting Protocol Strategy with Fallback
///
/// Uses `ProtocolIntelligence` to learn domain capabilities and automatically
/// selects the best protocol based on historical success/failure data.
pub struct AutoStrategy {
    /// HTTP/3 strategy
    h3_strategy: H3Strategy,
    /// HTTP/2 strategy
    h2_strategy: H2Strategy,
    /// Protocol preference order
    prefer: Vec<HttpVersion>,
    /// Protocol intelligence cache for learning domain capabilities
    intelligence: Arc<ProtocolIntelligence>,
}

impl AutoStrategy {
    /// Create a new auto strategy with preference order
    #[must_use] 
    pub fn new(prefer: Vec<HttpVersion>, configs: ProtocolConfigs) -> Self {
        Self {
            h3_strategy: H3Strategy::new(configs.h3.clone()),
            h2_strategy: H2Strategy::new(configs.h2.clone()),
            prefer,
            intelligence: Arc::new(ProtocolIntelligence::new()),
        }
    }
    
    /// Extract domain from request URL
    fn extract_domain(&self, request: &HttpRequest) -> String {
        request.url().host_str().unwrap_or("localhost").to_string()
    }
    
    /// Choose strategy based on protocol
    fn get_strategy(&self, protocol: HttpVersion) -> &dyn ProtocolStrategy {
        match protocol {
            HttpVersion::Http3 => &self.h3_strategy,
            HttpVersion::Http2 => &self.h2_strategy,
        }
    }
    
    /// Verify if response indicates successful connection
    fn verify_connection_success(&self, response: &HttpResponse) -> bool {
        // For streaming responses, we check if the connection was actually established
        
        // If status indicates error, connection failed
        if response.is_error() {
            tracing::debug!("Connection failed: status code indicates error");
            return false;
        }
        
        // For HTTP/3, check for actual response data to determine success
        if response.version() == Version::HTTP_3 {
            // Check if we have headers (indicates successful handshake)
            let headers = response.headers();
            if headers.is_empty() {
                tracing::debug!("HTTP/3 connection failed: no headers received");
                return false;
            }
            
            // Try to get body data to verify data flow
            let body_data = response.body();
            if body_data.is_empty() {
                tracing::debug!("HTTP/3 connection failed: no body data received");
                return false;
            }
            tracing::debug!("HTTP/3 connection verified: body data available");
        }
        
        true
    }
    
    /// Check if this request should skip HTTP/3 entirely
    fn should_skip_http3(&self, request: &HttpRequest) -> bool {
        let url = request.url();
        
        // Skip HTTP/3 for localhost/127.0.0.1 over HTTP (not HTTPS)
        if url.scheme() == "http"
            && let Some(host) = url.host_str()
                && (host == "localhost" || host == "127.0.0.1" || host == "::1") {
                    tracing::debug!("Skipping HTTP/3 for {} over HTTP - QUIC requires HTTPS", host);
                    return true;
                }
        
        false
    }
    
    /// Execute request with intelligent protocol selection and learning
    fn execute_with_intelligence(&self, request: HttpRequest) -> HttpResponse {
        let domain = self.extract_domain(&request);
        
        // Check if we should skip HTTP/3 entirely for this request
        if self.should_skip_http3(&request) {
            tracing::debug!("Using HTTP/2 directly for {}", domain);
            return self.h2_strategy.execute(request);
        }
        
        // Get intelligent protocol preference for this domain
        let preferred_protocol = self.intelligence.get_preferred_protocol(&domain);
        
        tracing::debug!(
            target: "quyc::protocols::auto",
            domain = %domain,
            preferred_protocol = ?preferred_protocol,
            "Selected protocol based on domain intelligence"
        );
        
        // Try preferred protocol first
        let primary_strategy = self.get_strategy(preferred_protocol);
        let primary_response = primary_strategy.execute(request.clone());
        
        // Use a simple success check - in real implementation this could be more sophisticated
        if self.verify_connection_success(&primary_response) {
            // Track success for learning
            self.intelligence.track_success(&domain, preferred_protocol);
            
            // Extract and process Alt-Svc header for future protocol discovery
            self.extract_and_process_alt_svc(&domain, &primary_response);
            
            tracing::debug!(
                target: "quyc::protocols::auto",
                domain = %domain,
                protocol = ?preferred_protocol,
                "Protocol succeeded, tracking success"
            );
            
            return primary_response;
        }
        
        // Primary protocol failed, track failure
        self.intelligence.track_failure(&domain, preferred_protocol);
        
        tracing::debug!(
            target: "quyc::protocols::auto",
            domain = %domain,
            failed_protocol = ?preferred_protocol,
            "Protocol failed, trying Alt-Svc endpoints and fallback"
        );
        
        // Try Alt-Svc discovered endpoints before fallback protocol
        if let Some(alt_svc_response) = self.try_alt_svc_endpoints(&domain, &request) {
            tracing::info!(
                target: "quyc::protocols::auto",
                domain = %domain,
                failed_protocol = ?preferred_protocol,
                "Alt-Svc endpoint succeeded, using discovered service"
            );
            return alt_svc_response;
        }
        
        // Try fallback protocol
        let fallback_protocol = match preferred_protocol {
            HttpVersion::Http3 => HttpVersion::Http2,
            HttpVersion::Http2 => HttpVersion::Http3, // Reverse fallback
        };
        
        // Only try fallback if intelligence suggests we should
        if self.intelligence.should_retry_protocol(&domain, fallback_protocol) {
            let fallback_strategy = self.get_strategy(fallback_protocol);
            let fallback_response = fallback_strategy.execute(request);
            
            if self.verify_connection_success(&fallback_response) {
                // Track fallback success
                self.intelligence.track_success(&domain, fallback_protocol);
                
                // Extract Alt-Svc header from successful fallback response
                self.extract_and_process_alt_svc(&domain, &fallback_response);
                
                tracing::info!(
                    target: "quyc::protocols::auto",
                    domain = %domain,
                    failed_protocol = ?preferred_protocol,
                    successful_protocol = ?fallback_protocol,
                    "Fallback protocol succeeded, domain learned"
                );
                
                return fallback_response;
            }
            // Track fallback failure too
            self.intelligence.track_failure(&domain, fallback_protocol);
            
            tracing::warn!(
                target: "quyc::protocols::auto",
                domain = %domain,
                primary_protocol = ?preferred_protocol,
                fallback_protocol = ?fallback_protocol,
                "Both protocols failed for domain"
            );
            
            return fallback_response;
        }
        
        // No suitable fallback available
        tracing::warn!(
            target: "quyc::protocols::auto",
            domain = %domain,
            protocol = ?preferred_protocol,
            "No fallback protocol available for domain"
        );
        
        primary_response
    }
    
    /// Extract Alt-Svc header from successful response and update domain intelligence
    /// 
    /// Implements RFC 7838 Alt-Svc header processing for service discovery.
    fn extract_and_process_alt_svc(&self, domain: &str, response: &HttpResponse) {
        // Extract Alt-Svc header from response
        if let Some(alt_svc_header) = response.header("alt-svc") {
            // Convert http::HeaderValue to string safely
            match alt_svc_header.to_str() {
                Ok(alt_svc_str) if !alt_svc_str.is_empty() => {
                    match self.intelligence.update_alt_svc_for_domain(domain, alt_svc_str) {
                        Ok(()) => {
                            tracing::debug!(
                                target: "quyc::protocols::auto",
                                domain = %domain,
                                alt_svc_header = %alt_svc_str,
                                "Successfully processed Alt-Svc header"
                            );
                        },
                        Err(e) => {
                            tracing::warn!(
                                target: "quyc::protocols::auto",
                                domain = %domain,
                                alt_svc_header = %alt_svc_str,
                                error = %e,
                                "Failed to process Alt-Svc header"
                            );
                        }
                    }
                },
                Ok(_) => {
                    // Empty Alt-Svc header, ignore
                },
                Err(_) => {
                    tracing::warn!(
                        target: "quyc::protocols::auto",
                        domain = %domain,
                        "Alt-Svc header contains invalid UTF-8, ignoring"
                    );
                }
            }
        }
    }
    
    /// Try Alt-Svc discovered endpoints for the domain
    /// 
    /// Tests alternative service endpoints discovered via RFC 7838 Alt-Svc headers.
    /// Returns Some(response) if any Alt-Svc endpoint succeeds, None otherwise.
    fn try_alt_svc_endpoints(&self, domain: &str, original_request: &HttpRequest) -> Option<HttpResponse> {
        let alt_svc_endpoints = self.intelligence.get_alt_svc_endpoints_for_domain(domain);
        
        if alt_svc_endpoints.is_empty() {
            tracing::debug!(
                target: "quyc::protocols::auto",
                domain = %domain,
                "No Alt-Svc endpoints available for domain"
            );
            return None;
        }
        
        tracing::debug!(
            target: "quyc::protocols::auto",
            domain = %domain,
            endpoint_count = alt_svc_endpoints.len(),
            "Trying Alt-Svc discovered endpoints"
        );
        
        // Try each Alt-Svc endpoint in order
        for endpoint in alt_svc_endpoints {
            if let Some(response) = self.try_single_alt_svc_endpoint(&endpoint, original_request) {
                return Some(response);
            }
        }
        
        tracing::debug!(
            target: "quyc::protocols::auto",
            domain = %domain,
            "All Alt-Svc endpoints failed"
        );
        
        None
    }
    
    /// Try a single Alt-Svc endpoint
    /// 
    /// Creates a modified request for the Alt-Svc endpoint and tests the connection.
    /// Returns Some(response) if successful, None if failed.
    fn try_single_alt_svc_endpoint(&self, endpoint: &AltSvcEndpoint, original_request: &HttpRequest) -> Option<HttpResponse> {
        // Create modified request for Alt-Svc endpoint
        let alt_svc_request = match self.create_alt_svc_request(endpoint, original_request) {
            Ok(request) => request,
            Err(e) => {
                tracing::warn!(
                    target: "quyc::protocols::auto",
                    protocol = %endpoint.protocol,
                    port = endpoint.port,
                    host = ?endpoint.host,
                    error = %e,
                    "Failed to create Alt-Svc request"
                );
                return None;
            }
        };
        
        // Select strategy based on Alt-Svc protocol
        let strategy = match endpoint.protocol.as_str() {
            "h3" => &self.h3_strategy as &dyn ProtocolStrategy,
            "h2" => &self.h2_strategy as &dyn ProtocolStrategy,
            _ => {
                tracing::debug!(
                    target: "quyc::protocols::auto",
                    protocol = %endpoint.protocol,
                    "Unsupported Alt-Svc protocol, skipping endpoint"
                );
                return None;
            }
        };
        
        // Execute request with Alt-Svc endpoint
        let response = strategy.execute(alt_svc_request);
        
        // Extract domain from original request for intelligence tracking
        let domain = if let Some(host) = original_request.url().host_str() { host } else {
            tracing::error!(
                target: "quyc::protocols::auto",
                "Failed to extract domain from Alt-Svc request URL: {}",
                original_request.url()
            );
            return None; // Fail fast - don't corrupt intelligence tracking with invalid domain
        };
        
        // Verify Alt-Svc endpoint success
        if self.verify_connection_success(&response) {
            tracing::info!(
                target: "quyc::protocols::auto",
                protocol = %endpoint.protocol,
                port = endpoint.port,
                host = ?endpoint.host,
                domain = %domain,
                "Alt-Svc endpoint succeeded"
            );
            
            // Update Alt-Svc endpoint validation status to Valid
            if let Err(e) = self.intelligence.update_alt_svc_endpoint_validation_status(
                domain, 
                &endpoint.protocol, 
                endpoint.port, 
                crate::protocols::intelligence::AltSvcValidationStatus::Valid
            ) {
                tracing::warn!(
                    target: "quyc::protocols::auto",
                    domain = %domain,
                    protocol = %endpoint.protocol,
                    port = endpoint.port,
                    error = %e,
                    "Failed to update Alt-Svc endpoint validation status to Valid"
                );
            }
            
            // Track protocol success in intelligence system
            let http_version = match endpoint.protocol.as_str() {
                "h3" => Some(crate::protocols::core::HttpVersion::Http3),
                "h2" => Some(crate::protocols::core::HttpVersion::Http2),
                _ => {
                    tracing::warn!(
                        target: "quyc::protocols::auto",
                        protocol = %endpoint.protocol,
                        "Unknown Alt-Svc protocol for intelligence tracking, skipping tracking"
                    );
                    None // Don't track unknown protocols
                }
            };
            
            if let Some(version) = http_version {
                self.intelligence.track_success(domain, version);
            }
            
            tracing::debug!(
                target: "quyc::protocols::auto",
                domain = %domain,
                protocol = %endpoint.protocol,
                port = endpoint.port,
                "Alt-Svc endpoint validation and protocol intelligence updated for success"
            );
            
            Some(response)
        } else {
            tracing::debug!(
                target: "quyc::protocols::auto",
                protocol = %endpoint.protocol,
                port = endpoint.port,
                host = ?endpoint.host,
                domain = %domain,
                "Alt-Svc endpoint failed"
            );
            
            // Update Alt-Svc endpoint validation status to Invalid
            if let Err(e) = self.intelligence.update_alt_svc_endpoint_validation_status(
                domain, 
                &endpoint.protocol, 
                endpoint.port, 
                crate::protocols::intelligence::AltSvcValidationStatus::Invalid
            ) {
                tracing::warn!(
                    target: "quyc::protocols::auto",
                    domain = %domain,
                    protocol = %endpoint.protocol,
                    port = endpoint.port,
                    error = %e,
                    "Failed to update Alt-Svc endpoint validation status to Invalid"
                );
            }
            
            // Track protocol failure in intelligence system
            let http_version = match endpoint.protocol.as_str() {
                "h3" => Some(crate::protocols::core::HttpVersion::Http3),
                "h2" => Some(crate::protocols::core::HttpVersion::Http2),
                _ => {
                    tracing::warn!(
                        target: "quyc::protocols::auto",
                        protocol = %endpoint.protocol,
                        "Unknown Alt-Svc protocol for intelligence tracking, skipping tracking"
                    );
                    None // Don't track unknown protocols
                }
            };
            
            if let Some(version) = http_version {
                self.intelligence.track_failure(domain, version);
            }
            
            tracing::debug!(
                target: "quyc::protocols::auto",
                domain = %domain,
                protocol = %endpoint.protocol,
                port = endpoint.port,
                "Alt-Svc endpoint validation and protocol intelligence updated for failure"
            );
            
            None
        }
    }
    
    /// Create modified request for Alt-Svc endpoint
    /// 
    /// Constructs a new `HttpRequest` with the Alt-Svc endpoint's host and port,
    /// copying all properties from the original request.
    fn create_alt_svc_request(&self, endpoint: &AltSvcEndpoint, original_request: &HttpRequest) -> Result<HttpRequest, String> {
        use crate::http::url::parse_url;
        
        // Parse original URL
        let mut url = parse_url(original_request.url().as_str())
            .map_err(|_| "Failed to parse original request URL")?;
        
        // Set Alt-Svc host if specified (None means same host as original)
        if let Some(ref alt_host) = endpoint.host {
            url.set_host(Some(alt_host))
                .map_err(|_| "Failed to set Alt-Svc host")?;
        }
        
        // Set Alt-Svc port
        url.set_port(Some(endpoint.port))
            .map_err(|()| "Failed to set Alt-Svc port")?;
        
        // Create new request using proper constructor with all original properties
        let mut request = HttpRequest::new(
            original_request.method().clone(),
            url,
            Some(original_request.headers().clone()),
            original_request.body().cloned(),
            original_request.timeout(),
        );
        
        // Copy all configuration properties from original request
        request.cors = original_request.cors;
        request.follow_redirects = original_request.follow_redirects;
        request.max_redirects = original_request.max_redirects;
        request.compress = original_request.compress;
        
        // Copy authentication
        request.auth = original_request.auth.clone();
        
        // Copy cache settings
        request.cache_control = original_request.cache_control.clone();
        request.etag = original_request.etag.clone();
        
        // Copy metadata
        request.user_agent = original_request.user_agent.clone();
        request.referer = original_request.referer.clone();
        
        // Copy protocol-specific options
        request.h2_prior_knowledge = original_request.h2_prior_knowledge;
        request.h3_alt_svc = original_request.h3_alt_svc;
        
        // Copy retry configuration if available (using getter method)
        if let Some(retry_attempts) = original_request.retry_attempts() {
            request = request.with_retry_attempts(retry_attempts);
        }
        
        // Copy version
        request = request.with_version(original_request.version());
        
        // Copy stream ID if present
        if let Some(stream_id) = original_request.stream_id {
            request = request.with_stream_id(stream_id);
        }
        
        Ok(request)
    }
}

impl ProtocolStrategy for AutoStrategy {
    fn execute(&self, request: HttpRequest) -> HttpResponse {
        self.execute_with_intelligence(request)
    }
    
    fn protocol_name(&self) -> &'static str {
        "Auto (with fallback)"
    }
    
    fn supports_push(&self) -> bool {
        // Support push if any protocol supports it (currently H2 does, H3 doesn't)
        self.h2_strategy.supports_push() || self.h3_strategy.supports_push()
    }
    
    fn max_concurrent_streams(&self) -> usize {
        // Use the higher limit between protocols
        std::cmp::max(
            self.h2_strategy.max_concurrent_streams(),
            self.h3_strategy.max_concurrent_streams()
        )
    }
}

// Additional methods specific to AutoStrategy (not part of ProtocolStrategy trait)
impl AutoStrategy {
    /// Get protocol intelligence statistics
    #[must_use] 
    pub fn intelligence_stats(&self) -> crate::protocols::intelligence::ProtocolIntelligenceStats {
        self.intelligence.stats()
    }
    
    /// Clear protocol intelligence cache (useful for testing)
    pub fn clear_intelligence(&self) {
        self.intelligence.clear();
    }
}