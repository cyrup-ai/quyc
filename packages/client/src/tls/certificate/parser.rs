//! Internal certificate parsing implementation details

use std::collections::HashMap;
use std::time::SystemTime;

use der::{Decode, Encode, Reader};
use x509_cert::Certificate as X509CertCert;
// Using available const_oid constants based on actual const_oid 0.9 API
use const_oid::db::rfc5912::{SECP_224_R_1, SECP_256_R_1, SECP_384_R_1, SECP_521_R_1, ID_EC_PUBLIC_KEY};
use const_oid::db::rfc8410::{ID_X_25519, ID_X_448, ID_ED_25519, ID_ED_448};
use der::{AnyRef, Length, SliceReader, Tag};

use spki::AlgorithmIdentifier;

use super::super::errors::TlsError;
use super::super::types::ParsedCertificate;

/// Extract name attributes from x509-cert Name structure
pub fn extract_name_attributes(name: &x509_cert::name::Name, attrs: &mut HashMap<String, String>) {
    use der::asn1::{Ia5StringRef, PrintableStringRef, Utf8StringRef};

    // Common OIDs for DN components
    const OID_CN: &str = "2.5.4.3"; // commonName
    const OID_O: &str = "2.5.4.10"; // organizationName
    const OID_OU: &str = "2.5.4.11"; // organizationalUnitName
    const OID_C: &str = "2.5.4.6"; // countryName
    const OID_ST: &str = "2.5.4.8"; // stateOrProvinceName
    const OID_L: &str = "2.5.4.7"; // localityName

    // Iterate through RDNs (Relative Distinguished Names)
    for rdn in &name.0 {
        // Each RDN contains one or more AttributeTypeAndValue
        for atv in rdn.0.iter() {
            let oid_string = atv.oid.to_string();

            // Extract the value as string using proper ASN.1 type handling
            // Try different ASN.1 string types as shown in x509-cert tests
            let string_value = if let Ok(ps) = PrintableStringRef::try_from(&atv.value) {
                Some(ps.to_string())
            } else if let Ok(utf8s) = Utf8StringRef::try_from(&atv.value) {
                Some(utf8s.to_string())
            } else if let Ok(ia5s) = Ia5StringRef::try_from(&atv.value) {
                Some(ia5s.to_string())
            } else {
                None
            };

            if let Some(value_str) = string_value {
                match oid_string.as_str() {
                    OID_CN => {
                        attrs.insert("CN".to_string(), value_str);
                    }
                    OID_O => {
                        attrs.insert("O".to_string(), value_str);
                    }
                    OID_OU => {
                        attrs.insert("OU".to_string(), value_str);
                    }
                    OID_C => {
                        attrs.insert("C".to_string(), value_str);
                    }
                    OID_ST => {
                        attrs.insert("ST".to_string(), value_str);
                    }
                    OID_L => {
                        attrs.insert("L".to_string(), value_str);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Extract Subject Alternative Names from certificate extension
fn extract_subject_alt_names(ext: &x509_cert::ext::Extension) -> (Vec<String>, Vec<std::net::IpAddr>) {
    let mut san_dns_names = Vec::new();
    let mut san_ip_addresses = Vec::new();

    // Parse SubjectAltName extension properly using ASN.1
    // SubjectAltName ::= GeneralNames
    // GeneralNames ::= SEQUENCE OF GeneralName
    use der::{Decode, Reader, SliceReader, Tag, TagNumber};

    let ext_data = ext.extn_value.as_bytes();

    // Parse the OCTET STRING wrapper first
    match der::asn1::OctetString::from_der(ext_data) {
        Ok(octet_string) => {
            // Now parse the actual SubjectAltName SEQUENCE
            let san_data = octet_string.as_bytes();
            let mut reader = if let Ok(reader) = SliceReader::new(san_data) { reader } else {
                tracing::warn!("Failed to create DER reader for SAN data");
                return (san_dns_names, san_ip_addresses);
            };

            // Read the SEQUENCE header
            if let Ok(header) = reader.peek_header()
                && header.tag == Tag::Sequence {
                    // Consume the header
                    if reader.peek_header().is_ok() {} else {
                        tracing::warn!("Failed to consume sequence header");
                        return (san_dns_names, san_ip_addresses);
                    }
                    if reader.read_slice(header.length).is_ok() {} else {
                        tracing::warn!("Failed to read sequence data");
                        return (san_dns_names, san_ip_addresses);
                    }

                    // Parse each GeneralName in the sequence
                    while !reader.is_finished() {
                        if let Ok(name_header) = reader.peek_header() {
                            match name_header.tag.number() {
                                TagNumber::N2 => {
                                    // dNSName [2] IMPLICIT IA5String
                                    if let Ok(dns_header) = reader.peek_header()
                                        && let Ok(dns_bytes) =
                                            reader.read_vec(dns_header.length)
                                            && let Ok(dns_name) =
                                                std::str::from_utf8(&dns_bytes)
                                            {
                                                san_dns_names
                                                    .push(dns_name.to_string());
                                            }
                                }
                                TagNumber::N7 => {
                                    // iPAddress [7] IMPLICIT OCTET STRING
                                    if let Ok(ip_header) = reader.peek_header()
                                        && let Ok(ip_bytes) =
                                            reader.read_vec(ip_header.length)
                                        {
                                            // IPv4 = 4 bytes, IPv6 = 16 bytes
                                            match ip_bytes.len() {
                                                4 => {
                                                    let octets: [u8; 4] =
                                                        if let Ok(octets) = ip_bytes.try_into() { octets } else {
                                                            tracing::warn!("Invalid IPv4 address bytes");
                                                            continue;
                                                        };
                                                    san_ip_addresses
                                                        .push(std::net::IpAddr::V4(
                                                        std::net::Ipv4Addr::from(
                                                            octets,
                                                        ),
                                                    ));
                                                }
                                                16 => {
                                                    let octets: [u8; 16] =
                                                        if let Ok(octets) = ip_bytes.try_into() { octets } else {
                                                            tracing::warn!("Invalid IPv6 address bytes");
                                                            continue;
                                                        };
                                                    san_ip_addresses
                                                        .push(std::net::IpAddr::V6(
                                                        std::net::Ipv6Addr::from(
                                                            octets,
                                                        ),
                                                    ));
                                                }
                                                _ => {
                                                    // Invalid IP address length
                                                }
                                            }
                                        }
                                }
                                _ => {
                                    // Skip other GeneralName types
                                    // (rfc822Name, x400Address, directoryName, ediPartyName, uniformResourceIdentifier, registeredID)
                                    let _ = reader.peek_header();
                                    let _ = reader.read_slice(name_header.length);
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
        }
        Err(e) => {
            tracing::error!("Failed to parse SubjectAltName extension: {}", e);
        }
    }

    (san_dns_names, san_ip_addresses)
}

/// Extract Basic Constraints from certificate extension
fn extract_basic_constraints(ext: &x509_cert::ext::Extension) -> bool {
    // Parse BasicConstraints extension
    // Structure: SEQUENCE { cA BOOLEAN DEFAULT FALSE, ... }
    let ext_data = ext.extn_value.as_bytes();

    // Look for the CA boolean flag
    // In DER encoding, BOOLEAN TRUE is 0x01 0x01 0xFF
    if ext_data.len() >= 3 {
        for i in 0..ext_data.len() - 2 {
            if ext_data[i] == 0x01
                && ext_data[i + 1] == 0x01
                && ext_data[i + 2] == 0xFF
            {
                return true;
            }
        }
    }
    false
}

/// Extract Key Usage from certificate extension
fn extract_key_usage(ext: &x509_cert::ext::Extension) -> Vec<String> {
    let mut key_usage = Vec::new();
    
    // Parse KeyUsage extension
    // Structure: BIT STRING with specific bit positions
    let ext_data = ext.extn_value.as_bytes();

    // KeyUsage bits (from RFC 5280):
    // 0: digitalSignature
    // 1: nonRepudiation/contentCommitment
    // 2: keyEncipherment
    // 3: dataEncipherment
    // 4: keyAgreement
    // 5: keyCertSign
    // 6: cRLSign
    // 7: encipherOnly
    // 8: decipherOnly

    // Find the bit string in the extension data
    // BIT STRING starts with tag 0x03
    for i in 0..ext_data.len() {
        if ext_data[i] == 0x03 && i + 2 < ext_data.len() {
            // Next byte is length, then unused bits, then the actual bits
            if i + 3 < ext_data.len() {
                let bits = ext_data[i + 3];

                if bits & 0x80 != 0 {
                    key_usage.push("digitalSignature".to_string());
                }
                if bits & 0x40 != 0 {
                    key_usage.push("contentCommitment".to_string());
                }
                if bits & 0x20 != 0 {
                    key_usage.push("keyEncipherment".to_string());
                }
                if bits & 0x10 != 0 {
                    key_usage.push("dataEncipherment".to_string());
                }
                if bits & 0x08 != 0 {
                    key_usage.push("keyAgreement".to_string());
                }
                if bits & 0x04 != 0 {
                    key_usage.push("keyCertSign".to_string());
                }
                if bits & 0x02 != 0 {
                    key_usage.push("cRLSign".to_string());
                }

                // Check second byte if present for last two bits
                if i + 4 < ext_data.len() && ext_data[i + 1] > 1 {
                    let bits2 = ext_data[i + 4];
                    if bits2 & 0x80 != 0 {
                        key_usage.push("encipherOnly".to_string());
                    }
                    if bits2 & 0x40 != 0 {
                        key_usage.push("decipherOnly".to_string());
                    }
                }
            }
            break;
        }
    }
    
    key_usage
}

/// Extract validity dates from certificate
fn extract_validity_dates(cert: &X509CertCert) -> (SystemTime, SystemTime) {
    let validity = &cert.tbs_certificate.validity;
    let not_before = validity.not_before.to_system_time();
    let not_after = validity.not_after.to_system_time();
    (not_before, not_after)
}

/// Extract certificate details using x509-cert
pub fn extract_certificate_details(
    cert: &X509CertCert,
) -> Result<
    (
        Vec<String>,
        Vec<std::net::IpAddr>,
        bool,
        Vec<String>,
        SystemTime,
        SystemTime,
    ),
    TlsError,
> {
    // Initialize collections
    let mut san_dns_names = Vec::new();
    let mut san_ip_addresses = Vec::new();
    let mut is_ca = false;
    let mut key_usage = Vec::new();

    // OIDs for extensions
    const OID_SUBJECT_ALT_NAME: &str = "2.5.29.17";
    const OID_BASIC_CONSTRAINTS: &str = "2.5.29.19";
    const OID_KEY_USAGE: &str = "2.5.29.15";

    // Process extensions using helper functions
    if let Some(extensions) = &cert.tbs_certificate.extensions {
        for ext in extensions {
            let oid_string = ext.extn_id.to_string();

            match oid_string.as_str() {
                OID_SUBJECT_ALT_NAME => {
                    let (dns_names, ip_addresses) = extract_subject_alt_names(ext);
                    san_dns_names = dns_names;
                    san_ip_addresses = ip_addresses;
                }
                OID_BASIC_CONSTRAINTS => {
                    is_ca = extract_basic_constraints(ext);
                }
                OID_KEY_USAGE => {
                    key_usage = extract_key_usage(ext);
                }
                _ => {}
            }
        }
    }

    // Extract validity dates using helper function
    let (not_before, not_after) = extract_validity_dates(cert);

    Ok((
        san_dns_names,
        san_ip_addresses,
        is_ca,
        key_usage,
        not_before,
        not_after,
    ))
}

/// Parse certificate from `X509Certificate` struct to extract actual certificate information
pub fn parse_x509_certificate_from_der_internal(cert: &X509CertCert) -> Result<ParsedCertificate, TlsError> {
    // Extract subject DN using x509-cert API
    let mut subject = HashMap::new();
    extract_name_attributes(&cert.tbs_certificate.subject, &mut subject);

    // Extract issuer DN using x509-cert API
    let mut issuer = HashMap::new();
    extract_name_attributes(&cert.tbs_certificate.issuer, &mut issuer);

    // Extract basic certificate info using x509-cert
    let (san_dns_names, san_ip_addresses, is_ca, key_usage, not_before, not_after) =
        extract_certificate_details(cert)?;

    // Extract OCSP and CRL URLs from certificate extensions
    let mut ocsp_urls = Vec::new();
    let mut crl_urls = Vec::new();

    // Iterate through all extensions to find Authority Information Access and CRL Distribution Points
    if let Some(extensions) = &cert.tbs_certificate.extensions {
        for ext in extensions {
            let oid_str = ext.extn_id.to_string();

            // Authority Information Access extension (1.3.6.1.5.5.7.1.1)
                            if let Ok(header) = reader.peek_header()
                                && header.tag == Tag::Sequence {
                                    // Consume the header
                                    if reader.peek_header().is_ok() {} else {
                                        tracing::warn!("Failed to consume sequence header");
                                        continue;
                                    }
                                    if reader.read_slice(header.length).is_ok() {} else {
                                        tracing::warn!("Failed to read sequence data");
                                        continue;
                                    }

                                    // Parse each GeneralName in the sequence
                                    while !reader.is_finished() {
                                        if let Ok(name_header) = reader.peek_header() {
                                            match name_header.tag.number() {
                                                TagNumber::N2 => {
                                                    // dNSName [2] IMPLICIT IA5String
                                                    if let Ok(dns_header) = reader.peek_header()
                                                        && let Ok(dns_bytes) =
                                                            reader.read_vec(dns_header.length)
                                                            && let Ok(dns_name) =
                                                                std::str::from_utf8(&dns_bytes)
                                                            {
                                                                san_dns_names
                                                                    .push(dns_name.to_string());
                                                            }
                                                }
                                                TagNumber::N7 => {
                                                    // iPAddress [7] IMPLICIT OCTET STRING
                                                    if let Ok(ip_header) = reader.peek_header()
                                                        && let Ok(ip_bytes) =
                                                            reader.read_vec(ip_header.length)
                                                        {
                                                            // IPv4 = 4 bytes, IPv6 = 16 bytes
                                                            match ip_bytes.len() {
                                                                4 => {
                                                                    let octets: [u8; 4] =
                                                                        if let Ok(octets) = ip_bytes.try_into() { octets } else {
                                                                            tracing::warn!("Invalid IPv4 address bytes");
                                                                            continue;
                                                                        };
                                                                    san_ip_addresses
                                                                        .push(std::net::IpAddr::V4(
                                                                        std::net::Ipv4Addr::from(
                                                                            octets,
                                                                        ),
                                                                    ));
                                                                }
                                                                16 => {
                                                                    let octets: [u8; 16] =
                                                                        if let Ok(octets) = ip_bytes.try_into() { octets } else {
                                                                            tracing::warn!("Invalid IPv6 address bytes");
                                                                            continue;
                                                                        };
                                                                    san_ip_addresses
                                                                        .push(std::net::IpAddr::V6(
                                                                        std::net::Ipv6Addr::from(
                                                                            octets,
                                                                        ),
                                                                    ));
                                                                }
                                                                _ => {
                                                                    // Invalid IP address length
                                                                }
                                                            }
                                                        }
                                                }
                                                _ => {
                                                    // Skip other GeneralName types
                                                    // (rfc822Name, x400Address, directoryName, ediPartyName, uniformResourceIdentifier, registeredID)
                                                    let _ = reader.peek_header();
                                                    let _ = reader.read_slice(name_header.length);
                                                }
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse SubjectAltName extension: {}", e);
                        }
                    }
                }
                OID_BASIC_CONSTRAINTS => {
                    // Parse BasicConstraints extension
                    // Structure: SEQUENCE { cA BOOLEAN DEFAULT FALSE, ... }
                    let ext_data = ext.extn_value.as_bytes();

                    // Look for the CA boolean flag
                    // In DER encoding, BOOLEAN TRUE is 0x01 0x01 0xFF
                    if ext_data.len() >= 3 {
                        for i in 0..ext_data.len() - 2 {
                            if ext_data[i] == 0x01
                                && ext_data[i + 1] == 0x01
                                && ext_data[i + 2] == 0xFF
                            {
                                is_ca = true;
                                break;
                            }
                        }
                    }
                }
                OID_KEY_USAGE => {
                    // Parse KeyUsage extension
                    // Structure: BIT STRING with specific bit positions
                    let ext_data = ext.extn_value.as_bytes();

                    // KeyUsage bits (from RFC 5280):
                    // 0: digitalSignature
                    // 1: nonRepudiation/contentCommitment
                    // 2: keyEncipherment
                    // 3: dataEncipherment
                    // 4: keyAgreement
                    // 5: keyCertSign
                    // 6: cRLSign
                    // 7: encipherOnly
                    // 8: decipherOnly

                    // Find the bit string in the extension data
                    // BIT STRING starts with tag 0x03
                    for i in 0..ext_data.len() {
                        if ext_data[i] == 0x03 && i + 2 < ext_data.len() {
                            // Next byte is length, then unused bits, then the actual bits
                            if i + 3 < ext_data.len() {
                                let bits = ext_data[i + 3];

                                if bits & 0x80 != 0 {
                                    key_usage.push("digitalSignature".to_string());
                                }
                                if bits & 0x40 != 0 {
                                    key_usage.push("contentCommitment".to_string());
                                }
                                if bits & 0x20 != 0 {
                                    key_usage.push("keyEncipherment".to_string());
                                }
                                if bits & 0x10 != 0 {
                                    key_usage.push("dataEncipherment".to_string());
                                }
                                if bits & 0x08 != 0 {
                                    key_usage.push("keyAgreement".to_string());
                                }
                                if bits & 0x04 != 0 {
                                    key_usage.push("keyCertSign".to_string());
                                }
                                if bits & 0x02 != 0 {
                                    key_usage.push("cRLSign".to_string());
                                }

                                // Check second byte if present for last two bits
                                if i + 4 < ext_data.len() && ext_data[i + 1] > 1 {
                                    let bits2 = ext_data[i + 4];
                                    if bits2 & 0x80 != 0 {
                                        key_usage.push("encipherOnly".to_string());
                                    }
                                    if bits2 & 0x40 != 0 {
                                        key_usage.push("decipherOnly".to_string());
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Extract validity times from TBS certificate
    let validity = &cert.tbs_certificate.validity;

    // Convert x509-cert Time to SystemTime
    let not_before = validity.not_before.to_system_time();
    let not_after = validity.not_after.to_system_time();

    Ok((
        san_dns_names,
        san_ip_addresses,
        is_ca,
        key_usage,
        not_before,
        not_after,
    ))
}

/// Parse certificate from `X509Certificate` struct to extract actual certificate information
pub fn parse_x509_certificate_from_der_internal(cert: &X509CertCert) -> Result<ParsedCertificate, TlsError> {
    // Extract subject DN using x509-cert API
    let mut subject = HashMap::new();
    extract_name_attributes(&cert.tbs_certificate.subject, &mut subject);

    // Extract issuer DN using x509-cert API
    let mut issuer = HashMap::new();
    extract_name_attributes(&cert.tbs_certificate.issuer, &mut issuer);

    // Extract basic certificate info using x509-cert
    let (san_dns_names, san_ip_addresses, is_ca, key_usage, not_before, not_after) =
        extract_certificate_details(cert)?;

    // Extract OCSP and CRL URLs from certificate extensions
    let mut ocsp_urls = Vec::new();
    let mut crl_urls = Vec::new();

    // Iterate through all extensions to find Authority Information Access and CRL Distribution Points
    if let Some(extensions) = &cert.tbs_certificate.extensions {
        for ext in extensions {
            let oid_str = ext.extn_id.to_string();

            // Authority Information Access extension (1.3.6.1.5.5.7.1.1)
            if oid_str == "1.3.6.1.5.5.7.1.1" {
                // Parse Authority Information Access with proper ASN.1 parsing
                match parse_authority_info_access_extension(ext.extn_value.as_bytes()) {
                    Ok(urls) => {
                        for url in urls {
                            if !ocsp_urls.contains(&url) {
                                ocsp_urls.push(url);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse Authority Information Access extension: {}", e);
                        // Fall back to the old method for compatibility
                        let ext_bytes = ext.extn_value.as_bytes();
                        for i in 0..ext_bytes.len().saturating_sub(4) {
                            if &ext_bytes[i..i + 4] == b"http" {
                                let mut url_bytes = Vec::new();
                                for &byte in &ext_bytes[i..] {
                                    if (0x20..=0x7E).contains(&byte) {
                                        url_bytes.push(byte);
                                    } else {
                                        break;
                                    }
                                }
                                if let Ok(url) = String::from_utf8(url_bytes)
                                    && url.starts_with("http") && !ocsp_urls.contains(&url) {
                                        ocsp_urls.push(url);
                                    }
                            }
                        }
                    }
                }
            }

            // CRL Distribution Points extension (2.5.29.31)
            if oid_str == "2.5.29.31" {
                // Parse CRL Distribution Points with proper ASN.1 parsing
                match parse_crl_distribution_points_extension(ext.extn_value.as_bytes()) {
                    Ok(urls) => {
                        for url in urls {
                            if !crl_urls.contains(&url) {
                                crl_urls.push(url);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse CRL Distribution Points extension: {}", e);
                        // Fall back to the old method for compatibility
                        let ext_bytes = ext.extn_value.as_bytes();
                        for i in 0..ext_bytes.len().saturating_sub(4) {
                            if &ext_bytes[i..i + 4] == b"http" {
                                let mut url_bytes = Vec::new();
                                for &byte in &ext_bytes[i..] {
                                    if (0x20..=0x7E).contains(&byte) {
                                        url_bytes.push(byte);
                                    } else {
                                        break;
                                    }
                                }
                                if let Ok(url) = String::from_utf8(url_bytes)
                                    && url.starts_with("http") && !crl_urls.contains(&url) {
                                        crl_urls.push(url);
                                    }
                            }
                        }
                    }
                }
            }
        }
    }

    // Get raw DER bytes for OCSP validation
    let subject_der = cert.tbs_certificate.subject.to_der()
        .map_err(|e| TlsError::CertificateParsing(format!("Failed to encode subject: {e}")))?;
    
    let public_key_der = cert.tbs_certificate.subject_public_key_info.to_der()
        .map_err(|e| TlsError::CertificateParsing(format!("Failed to encode public key: {e}")))?;

    // Extract serial number
    let serial_number = cert.tbs_certificate.serial_number.as_bytes().to_vec();

    // Extract key algorithm and size information
    let (key_algorithm, key_size) = extract_key_info_from_cert(cert)
        .unwrap_or_else(|| ("Unknown".to_string(), None));

    Ok(ParsedCertificate {
        subject,
        issuer,
        san_dns_names,
        san_ip_addresses,
        is_ca,
        key_usage,
        not_before,
        not_after,
        serial_number,
        ocsp_urls,
        crl_urls,
        subject_der,
        public_key_der,
        key_algorithm,
        key_size,
    })
}

/// Extract key algorithm name and size from X.509 certificate
pub fn extract_key_info_from_cert(cert: &X509CertCert) -> Option<(String, Option<u32>)> {
    // Define common algorithm OIDs manually since some are not available in const_oid 0.9
    const RSA_ENCRYPTION_OID: &str = "1.2.840.113549.1.1.1";
    const DSA_OID: &str = "1.2.840.10040.4.1";
    const DH_OID: &str = "1.2.840.10046.2.1";

    let algorithm = &cert.tbs_certificate.subject_public_key_info.algorithm;
    let algorithm_oid = &algorithm.oid;

    let algorithm_oid_str = algorithm_oid.to_string();
    let algorithm_name = if algorithm_oid_str == RSA_ENCRYPTION_OID {
        "RSA".to_string()
    } else if algorithm_oid_str == DSA_OID {
        "DSA".to_string()
    } else if algorithm_oid_str == DH_OID {
        "DH".to_string()
    } else if algorithm_oid_str == ID_EC_PUBLIC_KEY.to_string() {
        "ECDSA".to_string()
    } else if algorithm_oid_str == ID_X_25519.to_string() {
        "X25519".to_string()
    } else if algorithm_oid_str == ID_X_448.to_string() {
        "X448".to_string()
    } else if algorithm_oid_str == ID_ED_25519.to_string() {
        "Ed25519".to_string()
    } else if algorithm_oid_str == ID_ED_448.to_string() {
        "Ed448".to_string()
    } else {
        "Unknown".to_string()
    };

    // Convert types for compatibility with extract_key_size_from_algorithm_and_key
    let parameters_bytes = algorithm.parameters.as_ref().and_then(|any| any.to_der().ok());
    let parameters_ref = parameters_bytes.as_ref().and_then(|bytes| der::AnyRef::from_der(bytes.as_slice()).ok());
    
    let algorithm_ref = spki::AlgorithmIdentifier {
        oid: algorithm.oid,
        parameters: parameters_ref,
    };
    let public_key_ref = der::asn1::BitStringRef::from_bytes(cert.tbs_certificate.subject_public_key_info.subject_public_key.as_bytes()?).ok()?;
    let key_size = extract_key_size_from_algorithm_and_key(
        &algorithm_ref,
        &public_key_ref
    );

    Some((algorithm_name, key_size))
}

/// Extract key size from algorithm and public key data
fn extract_key_size_from_algorithm_and_key(
    algorithm: &AlgorithmIdentifier<AnyRef>,
    public_key: &der::asn1::BitStringRef,
) -> Option<u32> {
    // Define common algorithm OIDs locally
    const RSA_ENCRYPTION_OID: &str = "1.2.840.113549.1.1.1";
    const DSA_OID: &str = "1.2.840.10040.4.1";
    const DH_OID: &str = "1.2.840.10046.2.1";
    
    let oid_str = algorithm.oid.to_string();
    if oid_str == RSA_ENCRYPTION_OID {
        extract_rsa_key_size(public_key)
    } else if oid_str == DSA_OID {
        extract_dh_like_key_size(algorithm.parameters.as_ref())
    } else if oid_str == DH_OID {
        extract_dh_like_key_size(algorithm.parameters.as_ref())
    } else if oid_str == ID_EC_PUBLIC_KEY.to_string() {
        extract_ec_key_size(algorithm.parameters.as_ref())
    } else if oid_str == ID_X_25519.to_string() {
        Some(256)
    } else if oid_str == ID_X_448.to_string() {
        Some(448)
    } else if oid_str == ID_ED_25519.to_string() {
        Some(256)
    } else if oid_str == ID_ED_448.to_string() {
        Some(448)
    } else {
        None
    }
}

/// Compute the bit length of a big-endian byte slice representing a positive integer
fn compute_bit_length(bytes: &[u8]) -> Option<u32> {
    let start = bytes.iter().position(|&b| b != 0)?;
    let effective = &bytes[start..];
    if effective.is_empty() {
        return None;
    }
    let high_byte = effective[0];
    let high_bits = 8u32 - high_byte.leading_zeros();
    let rest_bits = u32::try_from((effective.len() - 1) * 8).unwrap_or(u32::MAX);
    Some(high_bits + rest_bits)
}

/// Skip a single ASN.1 element using a `SliceReader`
fn skip_element(reader: &mut der::SliceReader) -> Option<()> {
    let header = reader.peek_header().ok()?;
    let header_len: usize = header.encoded_len().ok()?.try_into().ok()?;
    let content_len: usize = header.length.try_into().ok()?;
    let total_len = header_len + content_len;
    reader.read_slice(der::Length::try_from(total_len).ok()?).ok()?;
    Some(())
}

/// Extract RSA modulus size in bits from RSA public key
fn extract_rsa_key_size(public_key: &der::asn1::BitStringRef) -> Option<u32> {
    use der::{Length, Tag};

    let key_bytes = public_key.as_bytes()?;
    let mut reader = der::SliceReader::new(key_bytes).ok()?;

    let sequence_header = reader.peek_header().ok()?;
    if sequence_header.tag != Tag::Sequence {
        return None;
    }
    let sequence_len = sequence_header.encoded_len().ok()?;
    let length = Length::try_from(sequence_len).ok()?;
    reader.read_slice(length).ok()?;

    let modulus_header = reader.peek_header().ok()?;
    if modulus_header.tag != Tag::Integer {
        return None;
    }
    let modulus_len = modulus_header.encoded_len().ok()?;
    reader.read_slice(Length::try_from(modulus_len).ok()?).ok()?;

    let modulus_bytes = reader.read_slice(modulus_header.length).ok()?;
    compute_bit_length(modulus_bytes)
}

/// Extract key size for DH-like algorithms (DSA, DH) from parameters
fn extract_dh_like_key_size(parameters_opt: Option<&AnyRef>) -> Option<u32> {
    let parameters = parameters_opt?;
    let bytes = parameters.value();
    let mut reader = der::SliceReader::new(bytes).ok()?;

    let sequence_header = reader.peek_header().ok()?;
    if sequence_header.tag != der::Tag::Sequence {
        return None;
    }
    let sequence_len = sequence_header.encoded_len().ok()?;
    let length = der::Length::try_from(sequence_len).ok()?;
    reader.read_slice(length).ok()?;

    let p_header = reader.peek_header().ok()?;
    if p_header.tag != der::Tag::Integer {
        return None;
    }
    let p_len = p_header.encoded_len().ok()?;
    reader.read_slice(der::Length::try_from(p_len).ok()?).ok()?;

    let p_bytes = reader.read_slice(p_header.length).ok()?;
    compute_bit_length(p_bytes)
}

/// Extract EC key size from curve parameters
fn extract_ec_key_size(parameters_opt: Option<&AnyRef>) -> Option<u32> {
    let parameters = parameters_opt?;
    let bytes = parameters.value();
    let mut reader = SliceReader::new(bytes).ok()?;

    let header = reader.peek_header().ok()?;
    match header.tag {
        Tag::ObjectIdentifier => {
            let header_len = header.encoded_len().ok()?;
            let length = Length::try_from(header_len).ok()?;
            reader.read_slice(length).ok()?;
            // Read the OID bytes and create ObjectIdentifier
            let oid_bytes = reader.read_slice(header.length).ok()?;
            let curve_oid = const_oid::ObjectIdentifier::from_bytes(oid_bytes).ok()?;
            match curve_oid {
                SECP_224_R_1 => Some(224),
                SECP_256_R_1 => Some(256),
                SECP_384_R_1 => Some(384),
                SECP_521_R_1 => Some(521),
                _ => {
                    // Handle other curves by OID string matching
                    let oid_str = curve_oid.to_string();
                    match oid_str.as_str() {
                        "1.2.840.10045.3.1.1" => Some(192), // SECP192R1
                        "1.3.132.0.31" => Some(192),        // SECP192K1
                        "1.3.132.0.32" => Some(224),        // SECP224K1
                        "1.3.132.0.10" => Some(256),        // SECP256K1
                        _ => None,
                    }
                }
            }
        }
        Tag::Null => None, // implicitCurve
        Tag::Sequence => {
            // specifiedCurve: ECParameters
            let header_len = header.encoded_len().ok()?;
            reader.read_slice(Length::try_from(header_len).ok()?).ok()?;
            // Skip version INTEGER
            skip_element(&mut reader)?;
            // Skip fieldID SEQUENCE
            skip_element(&mut reader)?;
            // Skip curve SEQUENCE
            skip_element(&mut reader)?;
            // Skip base OCTET STRING
            skip_element(&mut reader)?;
            // Now order INTEGER
            let order_header = reader.peek_header().ok()?;
            if order_header.tag != Tag::Integer {
                return None;
            }
            let order_len = order_header.encoded_len().ok()?;
            reader.read_slice(Length::try_from(order_len).ok()?).ok()?;
            let order_bytes = reader.read_slice(order_header.length).ok()?;
            compute_bit_length(order_bytes)
        }
        _ => None,
    }
}

/// Parse certificate from PEM data to extract actual certificate information
pub fn parse_certificate_from_pem(pem_data: &str) -> Result<ParsedCertificate, TlsError> {
    // Parse PEM to get DER bytes using rustls-pemfile
    let mut cursor = std::io::Cursor::new(pem_data.as_bytes());
    let cert_der = rustls_pemfile::certs(&mut cursor)
        .next()
        .ok_or_else(|| TlsError::CertificateParsing("No certificate in PEM data".to_string()))?
        .map_err(|e| TlsError::CertificateParsing(format!("Failed to parse PEM: {e}")))?;

    // Parse X.509 certificate using x509-cert
    let cert = X509CertCert::from_der(&cert_der)
        .map_err(|e| TlsError::CertificateParsing(format!("X.509 parsing failed: {e}")))?;

    // Delegate to the DER function to avoid code duplication
    parse_x509_certificate_from_der_internal(&cert)
}

/// Parse Authority Information Access extension with proper ASN.1 parsing
fn parse_authority_info_access_extension(extension_bytes: &[u8]) -> Result<Vec<String>, TlsError> {
    use der::{Decode, asn1::ObjectIdentifier};
    
    // OCSP access method OID (1.3.6.1.5.5.7.48.1)
    const ID_AD_OCSP: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.6.1.5.5.7.48.1");
    
    // AccessDescription structure from x509-cert
    #[derive(der::Sequence)]
    struct AccessDescription {
        access_method: ObjectIdentifier,
        access_location: der::Any,
    }
    
    // AuthorityInfoAccessSyntax is a SEQUENCE OF AccessDescription
    let access_descriptions: Vec<AccessDescription> = Vec::<AccessDescription>::from_der(extension_bytes)
        .map_err(|e| TlsError::CertificateParsing(format!("Failed to parse Authority Information Access: {e}")))?;
    
    let mut ocsp_urls = Vec::new();
    
    for access_desc in access_descriptions {
        // Only process OCSP access method
        if access_desc.access_method == ID_AD_OCSP {
            // Parse GeneralName - we're looking for uniformResourceIdentifier (tag [6])
            let access_location_bytes = access_desc.access_location.value();
            
            // Check if it's a uniformResourceIdentifier (context tag 6)
            if !access_location_bytes.is_empty() && access_location_bytes[0] == 0x86 {
                // Skip the tag byte and get the length
                let mut offset = 1;
                if offset >= access_location_bytes.len() {
                    continue;
                }
                
                // Parse length (simplified - assumes length < 128)
                let url_len = access_location_bytes[offset] as usize;
                offset += 1;
                
                if offset + url_len <= access_location_bytes.len()
                    && let Ok(url) = String::from_utf8(access_location_bytes[offset..offset + url_len].to_vec())
                        && (url.starts_with("http://") || url.starts_with("https://")) {
                            ocsp_urls.push(url);
                        }
            }
        }
    }
    
    Ok(ocsp_urls)
}

/// Parse CRL Distribution Points extension with proper ASN.1 parsing
fn parse_crl_distribution_points_extension(extension_bytes: &[u8]) -> Result<Vec<String>, TlsError> {
    use der::Decode;
    
    // DistributionPoint structure
    #[derive(der::Sequence)]
    struct DistributionPoint {
        #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
        distribution_point: Option<der::Any>,
        
        #[asn1(context_specific = "1", tag_mode = "IMPLICIT", optional = "true")]
        reasons: Option<der::Any>,
        
        #[asn1(context_specific = "2", tag_mode = "IMPLICIT", optional = "true")]
        crl_issuer: Option<der::Any>,
    }
    
    // CRLDistributionPoints is a SEQUENCE OF DistributionPoint
    let distribution_points: Vec<DistributionPoint> = Vec::<DistributionPoint>::from_der(extension_bytes)
        .map_err(|e| TlsError::CertificateParsing(format!("Failed to parse CRL Distribution Points: {e}")))?;
    
    let mut crl_urls = Vec::new();
    
    for dp in distribution_points {
        if let Some(dp_name) = dp.distribution_point {
            // Parse DistributionPointName - we want fullName [0] GeneralNames
            let dp_bytes = dp_name.value();
            
            // Check if it's fullName (context tag 0)
            if !dp_bytes.is_empty() && dp_bytes[0] == 0xA0 {
                // Parse GeneralNames - look for uniformResourceIdentifier (tag [6])
                let mut offset = 1;
                if offset >= dp_bytes.len() {
                    continue;
                }
                
                // Skip length byte (simplified)
                let _len = dp_bytes[offset] as usize;
                offset += 1;
                
                // Look for uniformResourceIdentifier entries
                while offset < dp_bytes.len() {
                    if dp_bytes[offset] == 0x86 { // uniformResourceIdentifier tag
                        offset += 1;
                        if offset >= dp_bytes.len() {
                            break;
                        }
                        
                        let url_len = dp_bytes[offset] as usize;
                        offset += 1;
                        
                        if offset + url_len <= dp_bytes.len()
                            && let Ok(url) = String::from_utf8(dp_bytes[offset..offset + url_len].to_vec())
                                && (url.starts_with("http://") || url.starts_with("https://")) {
                                    crl_urls.push(url);
                                }
                        offset += url_len;
                    } else {
                        offset += 1;
                    }
                }
            }
        }
    }
    
    Ok(crl_urls)
}

/// Parse certificate from DER data to extract actual certificate information
pub fn parse_certificate_from_der(der_bytes: &[u8]) -> Result<ParsedCertificate, TlsError> {
    // Parse X.509 certificate using x509-cert
    let cert = X509CertCert::from_der(der_bytes)
        .map_err(|e| TlsError::CertificateParsing(format!("X.509 parsing failed: {e}")))?;

    // Delegate to the internal function
    parse_x509_certificate_from_der_internal(&cert)
}