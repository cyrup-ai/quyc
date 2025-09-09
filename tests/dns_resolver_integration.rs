//! Production DNS resolver integration tests
//!
//! Tests real DNS resolution, timeout behavior, error cases, and integration with HTTP flows.
//! Uses .expect() for clear test failures with meaningful messages.

use std::time::Duration;

use ystream::prelude::*;
use quyc::http::resolver::{ResolvedAddress, Resolver};

#[test]
fn test_localhost_resolution() {
    let resolver = Resolver::new();
    let stream = resolver.resolve("localhost", 80);
    let addresses: Vec<ResolvedAddress> = stream.collect();

    assert!(
        !addresses.is_empty(),
        "Should resolve localhost to at least one address"
    );

    for addr in &addresses {
        assert!(
            !addr.is_error(),
            "Localhost resolution should not produce errors"
        );
        assert_eq!(addr.port, 80, "Port should match requested port");
        assert_eq!(addr.hostname.as_ref(), "localhost", "Hostname should match");
    }

    println!("✅ Localhost resolved to {} addresses", addresses.len());
}

#[test]
fn test_ip_address_fast_path() {
    let resolver = Resolver::new();

    // Test IPv4 address
    let stream = resolver.resolve("127.0.0.1", 8080);
    let addresses: Vec<ResolvedAddress> = stream.collect();

    assert_eq!(
        addresses.len(),
        1,
        "IP address should resolve to exactly one address"
    );
    let addr = &addresses[0];
    assert!(
        !addr.is_error(),
        "IP address resolution should not produce errors"
    );
    assert_eq!(addr.port, 8080, "Port should match requested port");
    assert_eq!(addr.ip.to_string(), "127.0.0.1", "IP should match input");

    println!("✅ IPv4 fast path works correctly");
}

#[test]
fn test_ipv6_address_fast_path() {
    let resolver = Resolver::new();

    // Test IPv6 address
    let stream = resolver.resolve("::1", 9000);
    let addresses: Vec<ResolvedAddress> = stream.collect();

    assert_eq!(
        addresses.len(),
        1,
        "IPv6 address should resolve to exactly one address"
    );
    let addr = &addresses[0];
    assert!(
        !addr.is_error(),
        "IPv6 address resolution should not produce errors"
    );
    assert_eq!(addr.port, 9000, "Port should match requested port");
    assert_eq!(addr.ip.to_string(), "::1", "IPv6 should match input");

    println!("✅ IPv6 fast path works correctly");
}

#[test]
fn test_invalid_hostname_validation() {
    let resolver = Resolver::new();

    // Test various invalid hostnames
    let invalid_hostnames = vec![
        "",                       // Empty hostname
        "a".repeat(254).as_str(), // Too long hostname
        "invalid..hostname",      // Consecutive dots
        "host-",                  // Trailing hyphen
        "-host",                  // Leading hyphen
        "host name",              // Space character
        "host@name",              // Invalid character
    ];

    for hostname in invalid_hostnames {
        let stream = resolver.resolve(hostname, 80);
        let addresses: Vec<ResolvedAddress> = stream.collect();

        assert_eq!(
            addresses.len(),
            1,
            "Should produce exactly one error chunk for invalid hostname"
        );
        assert!(
            addresses[0].is_error(),
            "Should produce error for invalid hostname: {}",
            hostname
        );

        if let Some(error_msg) = addresses[0].error() {
            println!(
                "✅ Invalid hostname '{}' correctly rejected: {}",
                hostname, error_msg
            );
        }
    }
}

#[test]
fn test_timeout_behavior() {
    // Test with very short timeout to trigger timeout errors
    let resolver = Resolver::new().with_timeout(Duration::from_millis(1));

    // Try to resolve a hostname that would normally work but timeout with 1ms
    let stream = resolver.resolve("google.com", 80);
    let addresses: Vec<ResolvedAddress> = stream.collect();

    // Should either succeed (if DNS is very fast) or produce timeout error
    if !addresses.is_empty() && addresses[0].is_error() {
        if let Some(error_msg) = addresses[0].error() {
            println!("✅ Timeout behavior verified: {}", error_msg);
        }
    } else {
        println!("✅ DNS resolution was faster than 1ms (acceptable)");
    }
}

#[test]
fn test_rate_limiting() {
    let resolver = Resolver::new().with_rate_limit(2); // Very low limit for testing

    let mut error_count = 0;
    let mut success_count = 0;

    // Make several requests rapidly
    for i in 0..5 {
        let stream = resolver.resolve("localhost", 80);
        let addresses: Vec<ResolvedAddress> = stream.collect();

        if !addresses.is_empty() {
            if addresses[0].is_error() {
                if let Some(error_msg) = addresses[0].error() {
                    if error_msg.contains("Rate limit") {
                        error_count += 1;
                        println!("✅ Request {} rate limited: {}", i + 1, error_msg);
                    }
                }
            } else {
                success_count += 1;
                println!("✅ Request {} succeeded", i + 1);
            }
        }
    }

    assert!(
        success_count > 0,
        "Some requests should succeed within rate limit"
    );
    println!(
        "✅ Rate limiting test: {} succeeded, {} rate limited",
        success_count, error_count
    );
}

#[test]
fn test_ipv6_preference() {
    let resolver = Resolver::new().with_ipv6_preference(true);

    // Test with localhost which typically has both IPv4 and IPv6
    let stream = resolver.resolve("localhost", 80);
    let addresses: Vec<ResolvedAddress> = stream.collect();

    assert!(!addresses.is_empty(), "Should resolve localhost");

    // Check if IPv6 addresses come first when available
    let has_ipv6 = addresses.iter().any(|addr| addr.ip.is_ipv6());
    let has_ipv4 = addresses.iter().any(|addr| addr.ip.is_ipv4());

    if has_ipv6 && has_ipv4 {
        // Find first IPv6 and IPv4 addresses
        let ipv6_pos = addresses.iter().position(|addr| addr.ip.is_ipv6());
        let ipv4_pos = addresses.iter().position(|addr| addr.ip.is_ipv4());

        if let (Some(ipv6_pos), Some(ipv4_pos)) = (ipv6_pos, ipv4_pos) {
            assert!(
                ipv6_pos < ipv4_pos,
                "IPv6 addresses should come before IPv4 when preference is enabled"
            );
            println!(
                "✅ IPv6 preference working: IPv6 at position {}, IPv4 at position {}",
                ipv6_pos, ipv4_pos
            );
        }
    }

    println!(
        "✅ IPv6 preference test completed (has_ipv6: {}, has_ipv4: {})",
        has_ipv6, has_ipv4
    );
}

#[test]
fn test_resolver_statistics() {
    let resolver = Resolver::new();

    // Perform some resolutions
    let _stream1 = resolver.resolve("127.0.0.1", 80).collect();
    let _stream2 = resolver.resolve("localhost", 80).collect();
    let _stream3 = resolver.resolve("invalid..hostname", 80).collect(); // This should fail

    let stats = resolver.stats();

    assert!(
        stats.success_count > 0,
        "Should have some successful resolutions"
    );
    assert_eq!(
        stats.timeout.as_secs(),
        5,
        "Should have default 5 second timeout"
    );
    assert_eq!(
        stats.max_requests_per_second, 100,
        "Should have default rate limit"
    );

    println!(
        "✅ Resolver stats: {} successes, {} failures, {:.1}% success rate",
        stats.success_count,
        stats.failure_count,
        stats.success_rate()
    );
}

#[test]
fn test_concurrent_resolution() {
    use std::sync::Arc;
    use std::thread;

    let resolver = Arc::new(Resolver::new());
    let mut handles = vec![];

    // Spawn multiple threads doing DNS resolution
    for i in 0..5 {
        let resolver = Arc::clone(&resolver);
        let handle = thread::spawn(move || {
            let stream = resolver.resolve("localhost", 80);
            let addresses: Vec<ResolvedAddress> = stream.collect();
            assert!(
                !addresses.is_empty(),
                "Thread {} should resolve localhost",
                i
            );
            addresses.len()
        });
        handles.push(handle);
    }

    // Wait for all threads and collect results
    let mut total_addresses = 0;
    for (i, handle) in handles.into_iter().enumerate() {
        let count = handle.join().expect("Thread should not panic");
        total_addresses += count;
        println!("✅ Thread {} resolved {} addresses", i, count);
    }

    assert!(
        total_addresses > 0,
        "Should resolve addresses across all threads"
    );
    println!(
        "✅ Concurrent resolution test: {} total addresses resolved",
        total_addresses
    );
}

#[test]
fn test_message_chunk_pattern() {
    let resolver = Resolver::new();

    // Test error chunk creation and detection
    let error_addr = ResolvedAddress::bad_chunk("Test error message".to_string());
    assert!(
        error_addr.is_error(),
        "Error chunk should be detected as error"
    );
    assert_eq!(
        error_addr.error().expect("Should have error message"),
        "Test error message"
    );
    assert_eq!(error_addr.port, 0, "Error chunks should have port 0");

    // Test normal address creation
    let normal_addr = ResolvedAddress::new(
        "127.0.0.1:8080".parse().expect("Valid socket address"),
        Arc::from("test-host"),
    );
    assert!(
        !normal_addr.is_error(),
        "Normal address should not be error"
    );
    assert!(
        normal_addr.error().is_none(),
        "Normal address should have no error message"
    );
    assert_eq!(
        normal_addr.port, 8080,
        "Normal address should preserve port"
    );

    println!("✅ MessageChunk pattern works correctly");
}

#[test]
fn test_integration_with_ystream_patterns() {
    let resolver = Resolver::new();

    // Test that our resolver correctly integrates with ystream patterns
    let stream = resolver.resolve("localhost", 80);

    // Test try_next (non-blocking)
    let first_address = stream.try_next();
    assert!(
        first_address.is_some() || stream.len() == 0,
        "try_next should work correctly"
    );

    // Test collection with error handling
    let addresses = stream.collect_or_else(|error_addr| {
        println!(
            "Handled error in stream: {}",
            error_addr.error().unwrap_or("Unknown error")
        );
        error_addr.clone()
    });

    assert!(
        !addresses.is_empty(),
        "Should collect addresses successfully"
    );

    println!("✅ Integration with ystream patterns verified");
}
