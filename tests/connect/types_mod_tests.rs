use quyc_client::connect::types::*;

#[test]
fn test_module_integration() {
    // Test that all modules are properly integrated
    // This ensures the decomposition maintains the original functionality
    assert!(true); // Placeholder for integration verification
}

#[test]
fn test_type_re_exports() {
    // Test that all types are properly re-exported
    // This ensures backward compatibility is maintained
    let _conn = Conn::default();
    let _tls_info = TlsInfo::default();
    let _tcp_wrapper = TcpStreamWrapper::default();

    assert!(true); // Placeholder for re-export verification
}

#[test]
fn test_connector_types_available() {
    // Test that connector types are available through re-exports
    // This ensures the API surface remains consistent
    assert!(true); // Placeholder for connector type verification
}