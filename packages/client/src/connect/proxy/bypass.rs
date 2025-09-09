//! Proxy bypass rules
//!
//! This module contains proxy bypass configuration for conditional proxy usage
//! based on hosts, domains, and IP addresses.

use http::Uri;

/// Proxy bypass rules for conditional proxy usage
#[derive(Clone, Debug)]
pub struct ProxyBypass {
    pub no_proxy_hosts: Vec<String>,
    pub no_proxy_domains: Vec<String>,
    pub no_proxy_ips: Vec<std::net::IpAddr>,
}

impl ProxyBypass {
    /// Create new bypass configuration
    pub fn new() -> Self {
        Self {
            no_proxy_hosts: Vec::new(),
            no_proxy_domains: Vec::new(),
            no_proxy_ips: Vec::new(),
        }
    }

    /// Add host to bypass list
    pub fn add_host(mut self, host: String) -> Self {
        self.no_proxy_hosts.push(host);
        self
    }

    /// Add domain to bypass list (matches subdomains)
    pub fn add_domain(mut self, domain: String) -> Self {
        self.no_proxy_domains.push(domain);
        self
    }

    /// Add IP address to bypass list
    pub fn add_ip(mut self, ip: std::net::IpAddr) -> Self {
        self.no_proxy_ips.push(ip);
        self
    }

    /// Check if URI should bypass proxy
    pub fn should_bypass(&self, uri: &Uri) -> bool {
        let host = match uri.host() {
            Some(h) => h,
            None => return false,
        };

        // Check exact host matches
        if self.no_proxy_hosts.contains(&host.to_string()) {
            return true;
        }

        // Check domain matches
        for domain in &self.no_proxy_domains {
            if host.ends_with(domain) || host == domain {
                return true;
            }
        }

        // Check IP matches
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            if self.no_proxy_ips.contains(&ip) {
                return true;
            }
        }

        false
    }
}

impl Default for ProxyBypass {
    fn default() -> Self {
        Self::new()
    }
}
