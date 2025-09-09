//! # Production DNS Resolver Usage Examples
//!
//! Comprehensive examples demonstrating all features of the production DNS resolver
//! including security considerations, performance optimization, and error handling.

use std::sync::Arc;
use std::time::Duration;

use ystream::prelude::*;
use quyc::http::resolver::{ResolvedAddress, Resolver};

fn main() {
    println!("🌐 Production DNS Resolver Usage Examples\n");

    // Initialize tracing for observability
    tracing_subscriber::fmt::init();

    basic_dns_resolution();
    ip_address_fast_path();
    timeout_configuration();
    rate_limiting_example();
    ipv6_preference_example();
    error_handling_patterns();
    concurrent_resolution_example();
    integration_with_http_clients();
    performance_monitoring();
    security_considerations();
}

/// Basic DNS resolution example
fn basic_dns_resolution() {
    println!("📋 1. Basic DNS Resolution");

    let resolver = Resolver::new();

    // Resolve a hostname to IP addresses
    let stream = resolver.resolve("google.com", 443);
    let addresses: Vec<ResolvedAddress> = stream.collect();

    println!(
        "   Resolved google.com:443 to {} addresses:",
        addresses.len()
    );
    for (i, addr) in addresses.iter().enumerate() {
        if !addr.is_error() {
            println!(
                "     {}. {} ({})",
                i + 1,
                addr.to_socket_addr(),
                if addr.ip.is_ipv6() { "IPv6" } else { "IPv4" }
            );
        }
    }
    println!();
}

/// IP address fast path optimization
fn ip_address_fast_path() {
    println!("📋 2. IP Address Fast Path Optimization");

    let resolver = Resolver::new();

    // IPv4 address - bypasses DNS lookup
    let start = std::time::Instant::now();
    let stream = resolver.resolve("8.8.8.8", 53);
    let addresses: Vec<ResolvedAddress> = stream.collect();
    let duration = start.elapsed();

    println!(
        "   IPv4 fast path (8.8.8.8:53): {} addresses in {:?}",
        addresses.len(),
        duration
    );

    // IPv6 address - also bypasses DNS lookup
    let start = std::time::Instant::now();
    let stream = resolver.resolve("2001:4860:4860::8888", 53);
    let addresses: Vec<ResolvedAddress> = stream.collect();
    let duration = start.elapsed();

    println!(
        "   IPv6 fast path (2001:4860:4860::8888:53): {} addresses in {:?}",
        addresses.len(),
        duration
    );
    println!();
}

/// Timeout configuration and behavior
fn timeout_configuration() {
    println!("📋 3. Timeout Configuration");

    // Configure different timeout values
    let fast_resolver = Resolver::new().with_timeout(Duration::from_millis(500));
    let normal_resolver = Resolver::new().with_timeout(Duration::from_secs(5));
    let patient_resolver = Resolver::new().with_timeout(Duration::from_secs(30));

    let hostname = "example.com";

    for (name, resolver) in [
        ("Fast (500ms)", &fast_resolver),
        ("Normal (5s)", &normal_resolver),
        ("Patient (30s)", &patient_resolver),
    ] {
        let start = std::time::Instant::now();
        let stream = resolver.resolve(hostname, 80);
        let addresses: Vec<ResolvedAddress> = stream.collect();
        let duration = start.elapsed();

        if !addresses.is_empty() && addresses[0].is_error() {
            println!(
                "   {} timeout: {} in {:?}",
                name,
                addresses[0].error().unwrap_or("Unknown error"),
                duration
            );
        } else {
            println!(
                "   {} resolver: {} addresses in {:?}",
                name,
                addresses.len(),
                duration
            );
        }
    }
    println!();
}

/// Rate limiting demonstration
fn rate_limiting_example() {
    println!("📋 4. Rate Limiting Protection");

    // Configure aggressive rate limiting for demonstration
    let resolver = Resolver::new().with_rate_limit(3); // 3 requests per second

    println!("   Making 5 rapid requests with 3/sec limit:");
    for i in 1..=5 {
        let stream = resolver.resolve("localhost", 80);
        let addresses: Vec<ResolvedAddress> = stream.collect();

        if !addresses.is_empty() {
            if addresses[0].is_error() {
                if let Some(error) = addresses[0].error() {
                    if error.contains("Rate limit") {
                        println!("     Request {}: ❌ {}", i, error);
                        continue;
                    }
                }
            }
            println!("     Request {}: ✅ {} addresses", i, addresses.len());
        }
    }
    println!();
}

/// IPv6 preference configuration
fn ipv6_preference_example() {
    println!("📋 5. IPv6 Preference Configuration");

    let ipv4_first = Resolver::new().with_ipv6_preference(false);
    let ipv6_first = Resolver::new().with_ipv6_preference(true);

    for (name, resolver) in [("IPv4 first", &ipv4_first), ("IPv6 first", &ipv6_first)] {
        let stream = resolver.resolve("localhost", 80);
        let addresses: Vec<ResolvedAddress> = stream.collect();

        print!("   {} resolver: ", name);
        for (i, addr) in addresses.iter().take(3).enumerate() {
            if !addr.is_error() {
                print!(
                    "{}{}",
                    if i > 0 { ", " } else { "" },
                    if addr.ip.is_ipv6() { "IPv6" } else { "IPv4" }
                );
            }
        }
        println!();
    }
    println!();
}

/// Comprehensive error handling patterns
fn error_handling_patterns() {
    println!("📋 6. Error Handling Patterns");

    let resolver = Resolver::new();

    // Test various error conditions
    let test_cases = vec![
        ("Empty hostname", ""),
        ("Invalid characters", "host@name.com"),
        ("Too long hostname", &"a".repeat(300)),
        ("Invalid format", "host..name"),
        (
            "Non-existent domain",
            "this-domain-definitely-does-not-exist-12345.invalid",
        ),
    ];

    for (test_name, hostname) in test_cases {
        let stream = resolver.resolve(hostname, 80);
        let addresses: Vec<ResolvedAddress> = stream.collect();

        if !addresses.is_empty() && addresses[0].is_error() {
            println!(
                "   {}: ❌ {}",
                test_name,
                addresses[0].error().unwrap_or("Unknown error")
            );
        } else {
            println!("   {}: ✅ Resolved successfully", test_name);
        }
    }
    println!();
}

/// Concurrent resolution demonstration
fn concurrent_resolution_example() {
    println!("📋 7. Concurrent DNS Resolution");

    let resolver = Arc::new(Resolver::new());
    let mut handles = vec![];

    let hostnames = vec![
        "google.com",
        "github.com",
        "stackoverflow.com",
        "rust-lang.org",
    ];

    println!("   Resolving {} hostnames concurrently:", hostnames.len());

    for hostname in hostnames {
        let resolver = Arc::clone(&resolver);
        let hostname = hostname.to_string();

        let handle = std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let stream = resolver.resolve(&hostname, 443);
            let addresses: Vec<ResolvedAddress> = stream.collect();
            let duration = start.elapsed();

            (hostname, addresses.len(), duration)
        });

        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        let (hostname, count, duration) = handle.join().unwrap();
        println!("     {}: {} addresses in {:?}", hostname, count, duration);
    }
    println!();
}

/// Integration with HTTP clients
fn integration_with_http_clients() {
    println!("📋 8. HTTP Client Integration");

    let resolver = Resolver::new();

    // Simulate HTTP client usage pattern
    let url = "https://api.github.com:443";
    let (hostname, port) = parse_url(url);

    println!("   Resolving {} for HTTP client:", url);

    let stream = resolver.resolve(&hostname, port);

    // Process addresses as they arrive (streaming pattern)
    let mut connection_attempts = 0;
    for address in stream {
        if address.is_error() {
            println!(
                "     ❌ DNS Error: {}",
                address.error().unwrap_or("Unknown")
            );
            break;
        }

        connection_attempts += 1;
        println!(
            "     ✅ Attempt {}: Connect to {}",
            connection_attempts,
            address.to_socket_addr()
        );

        // In real HTTP client, you would attempt connection here
        // For demo, we'll just show the first few addresses
        if connection_attempts >= 2 {
            println!("     ... (showing first 2 addresses)");
            break;
        }
    }
    println!();
}

/// Performance monitoring and statistics
fn performance_monitoring() {
    println!("📋 9. Performance Monitoring");

    let resolver = Resolver::new();

    // Perform various resolutions to generate statistics
    let test_hosts = vec!["google.com", "127.0.0.1", "localhost", "invalid..host"];

    for host in test_hosts {
        let _addresses: Vec<ResolvedAddress> = resolver.resolve(host, 80).collect();
    }

    let stats = resolver.stats();
    println!("   DNS Resolver Statistics:");
    println!("     • Total requests: {}", stats.request_count);
    println!("     • Successful resolutions: {}", stats.success_count);
    println!("     • Failed resolutions: {}", stats.failure_count);
    println!("     • Success rate: {:.1}%", stats.success_rate());
    println!("     • Failure rate: {:.1}%", stats.failure_rate());
    println!("     • Configured timeout: {:?}", stats.timeout);
    println!(
        "     • Rate limit: {} req/sec",
        stats.max_requests_per_second
    );
    println!();
}

/// Security considerations and best practices
fn security_considerations() {
    println!("📋 10. Security Considerations");

    let resolver = Resolver::new();

    println!("   ✅ Hostname validation prevents DNS injection");
    println!("   ✅ Rate limiting prevents DNS server abuse");
    println!("   ✅ Timeout handling prevents hanging connections");
    println!("   ✅ Input sanitization blocks malicious hostnames");
    println!("   ✅ IPv6/IPv4 preference configurable for security policies");

    // Demonstrate security validation
    let malicious_inputs = vec![
        "../../../../etc/passwd",
        "<script>alert('xss')</script>",
        "host`command`",
        "\x00\x01\x02",
    ];

    println!("   Testing malicious input handling:");
    for input in malicious_inputs {
        let stream = resolver.resolve(input, 80);
        let addresses: Vec<ResolvedAddress> = stream.collect();

        if !addresses.is_empty() && addresses[0].is_error() {
            println!("     ❌ Blocked malicious input: {:?}", input);
        }
    }
    println!();
}

/// Helper function to parse URL into hostname and port
fn parse_url(url: &str) -> (String, u16) {
    // Simple URL parsing for demo - in production use a proper URL parser
    let url = url.replace("https://", "").replace("http://", "");
    if let Some(colon_pos) = url.rfind(':') {
        let hostname = url[..colon_pos].to_string();
        let port = url[colon_pos + 1..].parse().unwrap_or(443);
        (hostname, port)
    } else {
        (url.to_string(), 443)
    }
}
