//! Tests for redirect handling functionality
//!
//! Comprehensive tests for redirect policies, header handling,
//! and various redirect scenarios.

use http::{HeaderMap, HeaderValue, StatusCode};
use hyper::header::{ACCEPT, AUTHORIZATION, COOKIE};
use quyc_client::redirect::attempt::ActionKind;
use quyc_client::redirect::headers::remove_sensitive_headers;
use quyc_client::redirect::policy::Policy;
use quyc_client::Url;

#[test]
fn test_redirect_policy_limit() {
    let policy = Policy::default();
    let next = Url::parse("http://x.y/z").expect("test URL should parse");
    let mut previous = (0..=9)
        .map(|i| Url::parse(&format!("http://a.b/c/{i}")).expect("test URL should parse"))
        .collect::<Vec<_>>();

    match policy.check(StatusCode::FOUND, &next, &previous) {
        ActionKind::Follow => (),
        other => panic!("unexpected {other:?}"),
    }

    previous.push(Url::parse("http://a.b.d/e/33").expect("test URL should parse"));

    match policy.check(StatusCode::FOUND, &next, &previous) {
        ActionKind::Error(err) if err.to_string().contains("too many redirects") => (),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn test_redirect_policy_limit_to_0() {
    let policy = Policy::limited(0);
    let next = Url::parse("http://x.y/z").expect("test URL should parse");
    let previous = vec![Url::parse("http://a.b/c").expect("test URL should parse")];

    match policy.check(StatusCode::FOUND, &next, &previous) {
        ActionKind::Error(err) if err.to_string().contains("too many redirects") => (),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn test_redirect_policy_custom() {
    let policy = Policy::custom(|attempt| {
        if attempt.url().host_str() == Some("foo") {
            attempt.stop()
        } else {
            attempt.follow()
        }
    });

    let next = Url::parse("http://bar/baz").expect("test URL should parse");
    match policy.check(StatusCode::FOUND, &next, &[]) {
        ActionKind::Follow => (),
        other => panic!("unexpected {other:?}"),
    }

    let next = Url::parse("http://foo/baz").expect("test URL should parse");
    match policy.check(StatusCode::FOUND, &next, &[]) {
        ActionKind::Stop => (),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn test_remove_sensitive_headers() {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(AUTHORIZATION, HeaderValue::from_static("let me in"));
    headers.insert(COOKIE, HeaderValue::from_static("foo=bar"));

    let next = Url::parse("http://initial-domain.com/path").expect("test URL should parse");
    let mut prev =
        vec![Url::parse("http://initial-domain.com/new_path").expect("test URL should parse")];
    let mut filtered_headers = headers.clone();

    remove_sensitive_headers(&mut headers, &next, &prev);
    assert_eq!(headers, filtered_headers);

    prev.push(Url::parse("http://new-domain.com/path").expect("test URL should parse"));
    filtered_headers.remove(AUTHORIZATION);
    filtered_headers.remove(COOKIE);

    remove_sensitive_headers(&mut headers, &next, &prev);
    assert_eq!(headers, filtered_headers);
}