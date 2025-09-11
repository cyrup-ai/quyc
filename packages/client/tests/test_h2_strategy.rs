#[cfg(test)]
mod tests {
    use quyc_client::protocols::h2::strategy::H2Strategy;
    use quyc_client::protocols::strategy::H2Config;
    use quyc_client::protocols::strategy_trait::ProtocolStrategy;
    use std::time::Duration;

    #[test]
    fn test_h2_strategy_creation() {
        // Create H2Config with standard settings
        let config = H2Config {
            max_concurrent_streams: 100,
            initial_window_size: 65535,
            max_frame_size: 16384,
            enable_push: false,
            enable_connect_protocol: false,
            keepalive_interval: Some(Duration::from_secs(30)),
            keepalive_timeout: Duration::from_secs(10),
            adaptive_window: true,
            max_send_buffer_size: 1024 * 1024,
        };
        
        // Create H2Strategy
        let strategy = H2Strategy::new(config);
        
        // Test basic properties
        assert_eq!(strategy.protocol_name(), "HTTP/2");
        assert!(!strategy.supports_push());
        assert_eq!(strategy.max_concurrent_streams(), 100);
        
        println!("✅ H2Strategy creation test passed!");
    }
}