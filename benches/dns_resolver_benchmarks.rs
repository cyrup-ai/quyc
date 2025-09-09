//! DNS Resolver Performance Benchmarks
//!
//! Comprehensive benchmarks for DNS resolution performance including:
//! - Single hostname resolution time
//! - Concurrent resolution throughput  
//! - Error handling overhead
//! - Memory usage profiling

use std::sync::Arc;
use std::time::Duration;

use ystream::prelude::*;
use quyc::http::resolver::{ResolvedAddress, Resolver};

fn main() {
    println!("🏁 DNS Resolver Performance Benchmarks\n");

    bench_single_resolution();
    bench_ip_fast_path();
    bench_concurrent_resolution();
    bench_error_handling();
    bench_timeout_configurations();
    bench_rate_limiting();
    bench_memory_patterns();
}

/// Benchmark single hostname resolution
fn bench_single_resolution() {
    println!("📊 1. Single Hostname Resolution Performance");

    let resolver = Resolver::new();
    let iterations = 1000;

    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let stream = resolver.resolve("localhost", 80);
        let _addresses: Vec<ResolvedAddress> = stream.collect();
    }
    let duration = start.elapsed();

    let avg_per_resolution = duration / iterations;
    let resolutions_per_sec = 1_000_000_000 / avg_per_resolution.as_nanos() as f64;

    println!("   {} resolutions in {:?}", iterations, duration);
    println!("   Average: {:?} per resolution", avg_per_resolution);
    println!("   Throughput: {:.0} resolutions/sec", resolutions_per_sec);
    println!();
}

/// Benchmark IP address fast path
fn bench_ip_fast_path() {
    println!("📊 2. IP Address Fast Path Performance");

    let resolver = Resolver::new();
    let iterations = 10000; // More iterations since this should be very fast

    // IPv4 fast path
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let stream = resolver.resolve("127.0.0.1", 80);
        let _addresses: Vec<ResolvedAddress> = stream.collect();
    }
    let ipv4_duration = start.elapsed();

    // IPv6 fast path
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let stream = resolver.resolve("::1", 80);
        let _addresses: Vec<ResolvedAddress> = stream.collect();
    }
    let ipv6_duration = start.elapsed();

    println!(
        "   IPv4 fast path: {} resolutions in {:?} ({:.0} res/sec)",
        iterations,
        ipv4_duration,
        iterations as f64 / ipv4_duration.as_secs_f64()
    );
    println!(
        "   IPv6 fast path: {} resolutions in {:?} ({:.0} res/sec)",
        iterations,
        ipv6_duration,
        iterations as f64 / ipv6_duration.as_secs_f64()
    );
    println!();
}

/// Benchmark concurrent resolution throughput
fn bench_concurrent_resolution() {
    println!("📊 3. Concurrent Resolution Throughput");

    let resolver = Arc::new(Resolver::new());
    let thread_counts = vec![1, 2, 4, 8];
    let resolutions_per_thread = 100;

    for thread_count in thread_counts {
        let start = std::time::Instant::now();
        let mut handles = vec![];

        for _ in 0..thread_count {
            let resolver = Arc::clone(&resolver);
            let handle = std::thread::spawn(move || {
                for _ in 0..resolutions_per_thread {
                    let stream = resolver.resolve("localhost", 80);
                    let _addresses: Vec<ResolvedAddress> = stream.collect();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();
        let total_resolutions = thread_count * resolutions_per_thread;
        let throughput = total_resolutions as f64 / duration.as_secs_f64();

        println!(
            "   {} threads: {} resolutions in {:?} ({:.0} res/sec)",
            thread_count, total_resolutions, duration, throughput
        );
    }
    println!();
}

/// Benchmark error handling overhead
fn bench_error_handling() {
    println!("📊 4. Error Handling Performance");

    let resolver = Resolver::new();
    let iterations = 1000;

    // Valid hostname (baseline)
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let stream = resolver.resolve("localhost", 80);
        let _addresses: Vec<ResolvedAddress> = stream.collect();
    }
    let valid_duration = start.elapsed();

    // Invalid hostname (error path)
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let stream = resolver.resolve("invalid..hostname", 80);
        let _addresses: Vec<ResolvedAddress> = stream.collect();
    }
    let invalid_duration = start.elapsed();

    // Empty hostname (validation error)
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let stream = resolver.resolve("", 80);
        let _addresses: Vec<ResolvedAddress> = stream.collect();
    }
    let empty_duration = start.elapsed();

    println!(
        "   Valid hostname: {:?} ({:.0} res/sec)",
        valid_duration,
        iterations as f64 / valid_duration.as_secs_f64()
    );
    println!(
        "   Invalid hostname: {:?} ({:.0} res/sec)",
        invalid_duration,
        iterations as f64 / invalid_duration.as_secs_f64()
    );
    println!(
        "   Empty hostname: {:?} ({:.0} res/sec)",
        empty_duration,
        iterations as f64 / empty_duration.as_secs_f64()
    );

    let error_overhead =
        (invalid_duration.as_nanos() as f64 / valid_duration.as_nanos() as f64 - 1.0) * 100.0;
    println!("   Error handling overhead: {:.1}%", error_overhead);
    println!();
}

/// Benchmark different timeout configurations
fn bench_timeout_configurations() {
    println!("📊 5. Timeout Configuration Impact");

    let timeouts = vec![
        Duration::from_millis(100),
        Duration::from_millis(500),
        Duration::from_secs(1),
        Duration::from_secs(5),
    ];

    let iterations = 100;

    for timeout in timeouts {
        let resolver = Resolver::new().with_timeout(timeout);

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let stream = resolver.resolve("localhost", 80);
            let _addresses: Vec<ResolvedAddress> = stream.collect();
        }
        let duration = start.elapsed();

        println!(
            "   {}ms timeout: {:?} total ({:.0} res/sec)",
            timeout.as_millis(),
            duration,
            iterations as f64 / duration.as_secs_f64()
        );
    }
    println!();
}

/// Benchmark rate limiting overhead
fn bench_rate_limiting() {
    println!("📊 6. Rate Limiting Performance Impact");

    let iterations = 1000;

    // No rate limiting (baseline)
    let unlimited_resolver = Resolver::new().with_rate_limit(10000);
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let stream = unlimited_resolver.resolve("127.0.0.1", 80);
        let _addresses: Vec<ResolvedAddress> = stream.collect();
    }
    let unlimited_duration = start.elapsed();

    // With rate limiting
    let limited_resolver = Resolver::new().with_rate_limit(100);
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let stream = limited_resolver.resolve("127.0.0.1", 80);
        let _addresses: Vec<ResolvedAddress> = stream.collect();
    }
    let limited_duration = start.elapsed();

    println!(
        "   No rate limit: {:?} ({:.0} res/sec)",
        unlimited_duration,
        iterations as f64 / unlimited_duration.as_secs_f64()
    );
    println!(
        "   With rate limit: {:?} ({:.0} res/sec)",
        limited_duration,
        iterations as f64 / limited_duration.as_secs_f64()
    );

    let overhead =
        (limited_duration.as_nanos() as f64 / unlimited_duration.as_nanos() as f64 - 1.0) * 100.0;
    println!("   Rate limiting overhead: {:.1}%", overhead);
    println!();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns() {
    println!("📊 7. Memory Allocation Patterns");

    let resolver = Resolver::new();

    // Simulate different usage patterns
    println!("   Testing memory usage patterns:");

    // Pattern 1: Many single-address resolutions
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let stream = resolver.resolve("127.0.0.1", 80);
        let addresses: Vec<ResolvedAddress> = stream.collect();
        drop(addresses); // Explicit drop to measure deallocation
    }
    let single_duration = start.elapsed();

    // Pattern 2: Fewer multi-address resolutions
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let stream = resolver.resolve("localhost", 80);
        let addresses: Vec<ResolvedAddress> = stream.collect();
        drop(addresses); // Explicit drop to measure deallocation
    }
    let multi_duration = start.elapsed();

    println!("   1000 single-address resolutions: {:?}", single_duration);
    println!("   100 multi-address resolutions: {:?}", multi_duration);

    // Statistics overhead
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let stream = resolver.resolve("127.0.0.1", 80);
        let _addresses: Vec<ResolvedAddress> = stream.collect();
        let _stats = resolver.stats();
    }
    let stats_duration = start.elapsed();

    println!("   1000 resolutions with stats: {:?}", stats_duration);
    println!();
}
