//! Opaque bearer authentication and explicit HTTP Host/Origin admission.
use crate::settings::ServerSettings;
use sha2::{
    Digest, Sha256,
    digest::{CtOutput, Output},
};
use std::collections::{BTreeMap, BTreeSet};

pub use eal_mcp_client::authentication::{read_bearer_token, validate_bearer_token};
#[derive(Clone)]
pub struct BearerTokenVerifier {
    digest: Output<Sha256>,
}
impl std::fmt::Debug for BearerTokenVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BearerTokenVerifier(..)")
    }
}
impl BearerTokenVerifier {
    pub fn new(token: &str) -> Result<Self, String> {
        validate_bearer_token(token)?;
        Ok(Self {
            digest: Sha256::digest(token.as_bytes()),
        })
    }
    pub fn verify(&self, token: &str) -> bool {
        CtOutput::<Sha256>::from(self.digest)
            == CtOutput::<Sha256>::from(Sha256::digest(token.as_bytes()))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthError {
    pub status: u16,
    pub message: &'static str,
}
#[derive(Clone, Debug)]
pub struct HttpPolicy {
    verifier: Option<BearerTokenVerifier>,
    hosts: BTreeSet<String>,
    origins: BTreeSet<String>,
}
fn normalise_host(host: &str) -> String {
    host.trim().to_ascii_lowercase()
}
fn hostname(value: &str) -> Option<String> {
    if value.chars().any(|c| c.is_whitespace() || c.is_control())
        || value.contains(['@', '/', '?', '#', '\\'])
    {
        return None;
    }
    let host = crate::settings::authority_hostname(value)?;
    crate::settings::validate_host(host).ok()?;
    Some(normalise_host(host))
}
fn origin_parts(value: &str) -> Option<(&str, &str, Option<u16>, &str)> {
    let (scheme, rest) = value.split_once("://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    let host = crate::settings::authority_hostname(authority)?;
    let suffix = if authority.starts_with('[') {
        authority.split_once(']')?.1
    } else {
        authority.strip_prefix(host)?
    };
    let port = suffix.strip_prefix(':').map(str::parse).transpose().ok()?;
    Some((scheme, host, port, &rest[end..]))
}
fn normalise_origin(value: &str) -> String {
    let original = value.trim().trim_end_matches('/');
    let (without_fragment, fragment) = original
        .split_once('#')
        .map(|(a, b)| (a, Some(b)))
        .unwrap_or((original, None));
    let (value, query) = without_fragment
        .split_once('?')
        .map(|(a, b)| (a, Some(b)))
        .unwrap_or((without_fragment, None));
    if query.is_some_and(|s| !s.is_empty()) || fragment.is_some_and(|s| !s.is_empty()) {
        return original.to_ascii_lowercase();
    }
    let Some((scheme, host, port, path)) = origin_parts(value) else {
        return value.to_ascii_lowercase();
    };
    if !path.is_empty() {
        return value.to_ascii_lowercase();
    }
    let scheme = scheme.to_ascii_lowercase();
    let host = normalise_host(host);
    let host = if host.contains(':') {
        format!("[{host}]")
    } else {
        host
    };
    let port = port.or(match scheme.as_str() {
        "http" => Some(80),
        "https" => Some(443),
        _ => None,
    });
    if let Some(port) = port {
        format!("{scheme}://{host}:{port}")
    } else {
        format!("{scheme}://{host}")
    }
}
fn origin_allowed(origin: &str, raw_host: &str, host: &str, origins: &BTreeSet<String>) -> bool {
    if origins.contains(&normalise_origin(origin)) {
        return true;
    }
    let origin_host = origin_parts(origin).map(|(_, h, _, _)| h);
    if origin_host.is_some_and(crate::settings::loopback_host)
        && crate::settings::loopback_host(host)
    {
        return true;
    }
    normalise_origin(origin) == normalise_origin(&format!("http://{raw_host}"))
}
impl HttpPolicy {
    pub fn from_settings(
        settings: &ServerSettings,
        environ: &BTreeMap<String, String>,
    ) -> Result<Self, String> {
        settings.validate()?;
        let verifier = if settings.transport == "http" {
            read_bearer_token(settings.token_env.as_deref(), environ)?
                .as_deref()
                .map(BearerTokenVerifier::new)
                .transpose()?
        } else {
            None
        };
        let hosts = ["127.0.0.1", "localhost", "::1"]
            .into_iter()
            .chain(std::iter::once(settings.host.as_str()))
            .chain(settings.allowed_hosts.iter().map(String::as_str))
            .map(normalise_host)
            .collect();
        let origins = settings
            .allowed_origins
            .iter()
            .map(|origin| normalise_origin(origin))
            .collect();
        Ok(Self {
            verifier,
            hosts,
            origins,
        })
    }
    pub fn authorize(
        &self,
        host: Option<&str>,
        origin: Option<&str>,
        authorization: Option<&str>,
    ) -> Result<(), AuthError> {
        let raw_host = host.unwrap_or("");
        let host = host.and_then(hostname).ok_or(AuthError {
            status: 421,
            message: "Invalid Host header",
        })?;
        if !self.hosts.contains(&host) {
            return Err(AuthError {
                status: 421,
                message: "Untrusted Host header",
            });
        }
        if let Some(origin) = origin
            && !origin.is_empty()
            && !origin_allowed(origin, raw_host, &host, &self.origins)
        {
            return Err(AuthError {
                status: 403,
                message: "Untrusted Origin header",
            });
        }
        if let Some(verifier) = &self.verifier {
            let token = authorization
                .and_then(|s| s.split_once(' '))
                .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("Bearer"))
                .map(|(_, token)| token);
            if !token.is_some_and(|token| verifier.verify(token)) {
                return Err(AuthError {
                    status: 401,
                    message: "Bearer authentication required",
                });
            }
        }
        Ok(())
    }
}
