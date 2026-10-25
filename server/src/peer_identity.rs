use std::{
    collections::HashSet,
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use axum::{
    extract::{ConnectInfo, FromRequestParts, connect_info::MockConnectInfo},
    http::{HeaderMap, request::Parts},
};

const MAX_TRUSTED_PROXIES: usize = 32;
const MAX_TRUSTED_PROXY_CONFIG_BYTES: usize = 2_048;
const MAX_FORWARDED_FOR_BYTES: usize = 64;
const X_FORWARDED_FOR: &str = "x-forwarded-for";

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) enum PeerKey {
    Ipv4(Ipv4Addr),
    Ipv6Prefix([u8; 8]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PeerIdentity {
    key: PeerKey,
}

pub(crate) struct TransportPeer(pub(crate) Option<SocketAddr>);

impl<S> FromRequestParts<S> for TransportPeer
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let address = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|connect| connect.0)
            .or_else(|| {
                parts
                    .extensions
                    .get::<MockConnectInfo<SocketAddr>>()
                    .map(|connect| connect.0)
            });
        Ok(Self(address))
    }
}

impl PeerIdentity {
    #[must_use]
    pub(crate) fn key(self) -> PeerKey {
        self.key
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TrustedProxyIps {
    addresses: HashSet<IpAddr>,
}

impl TrustedProxyIps {
    pub(crate) fn parse(value: Option<&str>) -> Result<Self, TrustedProxyConfigError> {
        let value = value.unwrap_or_default();
        if value.is_empty() {
            return Ok(Self::default());
        }
        if value.len() > MAX_TRUSTED_PROXY_CONFIG_BYTES {
            return Err(TrustedProxyConfigError);
        }

        let literals: Vec<_> = value.split(',').collect();
        if literals.len() > MAX_TRUSTED_PROXIES {
            return Err(TrustedProxyConfigError);
        }
        let mut addresses = HashSet::with_capacity(literals.len());
        for literal in literals {
            if literal.is_empty() || literal != literal.trim() {
                return Err(TrustedProxyConfigError);
            }
            let address = literal
                .parse::<IpAddr>()
                .map_err(|_| TrustedProxyConfigError)?;
            addresses.insert(normalize_ip(address));
        }
        Ok(Self { addresses })
    }

    fn contains(&self, address: IpAddr) -> bool {
        self.addresses.contains(&normalize_ip(address))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TrustedProxyConfigError;

impl fmt::Display for TrustedProxyConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("trusted proxy configuration is invalid")
    }
}

impl std::error::Error for TrustedProxyConfigError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PeerIdentityError {
    Unavailable,
    InvalidForwardedFor,
}

pub(crate) fn resolve_peer_identity(
    transport_peer: Option<SocketAddr>,
    headers: &HeaderMap,
    trusted_proxies: &TrustedProxyIps,
) -> Result<PeerIdentity, PeerIdentityError> {
    let transport_ip = normalize_ip(transport_peer.ok_or(PeerIdentityError::Unavailable)?.ip());
    let resolved_ip = if trusted_proxies.contains(transport_ip) {
        parse_forwarded_for(headers)?
    } else {
        transport_ip
    };
    Ok(PeerIdentity {
        key: peer_key(resolved_ip),
    })
}

fn parse_forwarded_for(headers: &HeaderMap) -> Result<IpAddr, PeerIdentityError> {
    let mut values = headers.get_all(X_FORWARDED_FOR).iter();
    let value = values
        .next()
        .ok_or(PeerIdentityError::InvalidForwardedFor)?;
    if values.next().is_some() {
        return Err(PeerIdentityError::InvalidForwardedFor);
    }
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_FORWARDED_FOR_BYTES {
        return Err(PeerIdentityError::InvalidForwardedFor);
    }
    let literal = value
        .to_str()
        .map_err(|_| PeerIdentityError::InvalidForwardedFor)?;
    if literal != literal.trim() || literal.contains(',') {
        return Err(PeerIdentityError::InvalidForwardedFor);
    }
    literal
        .parse::<IpAddr>()
        .map(normalize_ip)
        .map_err(|_| PeerIdentityError::InvalidForwardedFor)
}

fn normalize_ip(address: IpAddr) -> IpAddr {
    match address {
        IpAddr::V6(address) => address
            .to_ipv4_mapped()
            .map_or(IpAddr::V6(address), IpAddr::V4),
        address => address,
    }
}

fn peer_key(address: IpAddr) -> PeerKey {
    match normalize_ip(address) {
        IpAddr::V4(address) => PeerKey::Ipv4(address),
        IpAddr::V6(address) => {
            let octets = address.octets();
            PeerKey::Ipv6Prefix(
                octets[..8]
                    .try_into()
                    .expect("IPv6 prefix has eight octets"),
            )
        }
    }
}

#[cfg(test)]
#[path = "peer_identity_tests.rs"]
mod tests;
