use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::http::{HeaderMap, HeaderValue};

use super::*;

fn socket(address: &str) -> SocketAddr {
    SocketAddr::new(address.parse().expect("valid test address"), 1234)
}

#[test]
fn direct_identity_ignores_spoofed_forwarding_headers() {
    let mut headers = HeaderMap::new();
    headers.insert(X_FORWARDED_FOR, HeaderValue::from_static("198.51.100.9"));
    let identity = resolve_peer_identity(
        Some(socket("192.0.2.7")),
        &headers,
        &TrustedProxyIps::default(),
    )
    .expect("direct peer resolves");
    assert_eq!(identity.key(), PeerKey::Ipv4(Ipv4Addr::new(192, 0, 2, 7)));
}

#[test]
fn loopback_is_not_exempt_from_normal_identity_resolution() {
    let identity = resolve_peer_identity(
        Some(socket("127.0.0.1")),
        &HeaderMap::new(),
        &TrustedProxyIps::default(),
    )
    .expect("loopback resolves like any direct peer");
    assert_eq!(identity.key(), PeerKey::Ipv4(Ipv4Addr::LOCALHOST));
}

#[test]
fn trusted_proxy_uses_one_valid_literal_and_requires_it() {
    let trusted = TrustedProxyIps::parse(Some("192.0.2.7")).expect("valid proxy config");
    let mut headers = HeaderMap::new();
    assert_eq!(
        resolve_peer_identity(Some(socket("192.0.2.7")), &headers, &trusted),
        Err(PeerIdentityError::InvalidForwardedFor)
    );

    headers.insert(X_FORWARDED_FOR, HeaderValue::from_static("198.51.100.9"));
    let identity = resolve_peer_identity(Some(socket("192.0.2.7")), &headers, &trusted)
        .expect("forwarded peer resolves");
    assert_eq!(
        identity.key(),
        PeerKey::Ipv4(Ipv4Addr::new(198, 51, 100, 9))
    );
}

#[test]
fn duplicate_chained_and_non_literal_forwarding_are_rejected() {
    let trusted = TrustedProxyIps::parse(Some("192.0.2.7")).expect("valid proxy config");
    for invalid in [
        "198.51.100.9, 203.0.113.4",
        "198.51.100.9:8080",
        "fe80::1%eth0",
        "\"198.51.100.9\"",
        " 198.51.100.9",
    ] {
        let mut headers = HeaderMap::new();
        headers.insert(
            X_FORWARDED_FOR,
            invalid.parse().expect("valid header bytes"),
        );
        assert_eq!(
            resolve_peer_identity(Some(socket("192.0.2.7")), &headers, &trusted),
            Err(PeerIdentityError::InvalidForwardedFor),
            "accepted {invalid:?}"
        );
    }

    let mut headers = HeaderMap::new();
    headers.append(X_FORWARDED_FOR, HeaderValue::from_static("198.51.100.9"));
    headers.append(X_FORWARDED_FOR, HeaderValue::from_static("203.0.113.4"));
    assert_eq!(
        resolve_peer_identity(Some(socket("192.0.2.7")), &headers, &trusted),
        Err(PeerIdentityError::InvalidForwardedFor)
    );
}

#[test]
fn forwarded_header_rejects_empty_oversized_and_non_utf8_values() {
    let trusted = TrustedProxyIps::parse(Some("192.0.2.7")).expect("valid proxy config");
    let mut headers = HeaderMap::new();
    for invalid in [
        HeaderValue::from_static(""),
        HeaderValue::from_str(&"1".repeat(MAX_FORWARDED_FOR_BYTES + 1))
            .expect("visible ASCII header"),
        HeaderValue::from_bytes(&[0x80]).expect("opaque header byte"),
    ] {
        headers.insert(X_FORWARDED_FOR, invalid);
        assert_eq!(
            resolve_peer_identity(Some(socket("192.0.2.7")), &headers, &trusted),
            Err(PeerIdentityError::InvalidForwardedFor)
        );
    }
}

#[test]
fn mapped_ipv4_is_normalized_for_proxy_trust_and_peer_keys() {
    let trusted = TrustedProxyIps::parse(Some("192.0.2.7")).expect("valid proxy config");
    let mut headers = HeaderMap::new();
    headers.insert(
        X_FORWARDED_FOR,
        HeaderValue::from_static("::ffff:198.51.100.9"),
    );
    let identity = resolve_peer_identity(Some(socket("::ffff:192.0.2.7")), &headers, &trusted)
        .expect("mapped transport matches full proxy address");
    assert_eq!(
        identity.key(),
        PeerKey::Ipv4(Ipv4Addr::new(198, 51, 100, 9))
    );
}

#[test]
fn native_ipv6_groups_by_64_but_proxy_trust_uses_the_full_address() {
    let trusted = TrustedProxyIps::parse(Some("2001:db8:1:2::1")).expect("valid proxy config");
    let headers = HeaderMap::new();
    let first = resolve_peer_identity(Some(socket("2001:db8:1:2::2")), &headers, &trusted)
        .expect("untrusted same-prefix transport resolves directly");
    let second = resolve_peer_identity(Some(socket("2001:db8:1:2::ffff")), &headers, &trusted)
        .expect("same prefix resolves");
    assert_eq!(first.key(), second.key());

    assert_eq!(
        resolve_peer_identity(Some(socket("2001:db8:1:2::1")), &headers, &trusted),
        Err(PeerIdentityError::InvalidForwardedFor)
    );
}

#[test]
fn missing_transport_metadata_is_not_an_unknown_shared_identity() {
    assert_eq!(
        resolve_peer_identity(None, &HeaderMap::new(), &TrustedProxyIps::default()),
        Err(PeerIdentityError::Unavailable)
    );
}

#[test]
fn proxy_configuration_is_bounded_and_accepts_only_ip_literals() {
    assert!(TrustedProxyIps::parse(None).is_ok());
    assert!(TrustedProxyIps::parse(Some("")).is_ok());
    assert!(TrustedProxyIps::parse(Some("127.0.0.1,::1")).is_ok());
    assert!(TrustedProxyIps::parse(Some("127.0.0.1, ::1")).is_err());
    assert!(TrustedProxyIps::parse(Some(" 127.0.0.1")).is_err());
    assert!(TrustedProxyIps::parse(Some("127.0.0.1 ")).is_err());
    assert!(TrustedProxyIps::parse(Some("   ")).is_err());
    assert!(TrustedProxyIps::parse(Some("localhost")).is_err());

    let too_many = (0..33)
        .map(|index| format!("192.0.2.{index}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(TrustedProxyIps::parse(Some(&too_many)).is_err());
    let oversized = "1".repeat(MAX_TRUSTED_PROXY_CONFIG_BYTES + 1);
    assert!(TrustedProxyIps::parse(Some(&oversized)).is_err());
    let oversized_outer_whitespace =
        format!("{}127.0.0.1", " ".repeat(MAX_TRUSTED_PROXY_CONFIG_BYTES));
    assert!(TrustedProxyIps::parse(Some(&oversized_outer_whitespace)).is_err());
}

#[test]
fn compressed_and_expanded_ipv6_literals_have_the_same_key() {
    let first: IpAddr = "2001:db8::1".parse().expect("valid IPv6");
    let second: IpAddr = "2001:0db8:0000:0000:ffff::1".parse().expect("valid IPv6");
    assert_eq!(peer_key(first), peer_key(second));
}
