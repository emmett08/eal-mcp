//! Origin-bound JSON acquisition with one deadline and immutable credential inspection.
use crate::Error;
use eal_collector_contracts::{JsonLimits, SensitiveJsonLimits, contains_sensitive_json};
use reqwest::{Method, Url, header};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

/// Captured credentials are inspected again after the complete observation is assembled.
pub struct JsonClient {
    client: reqwest::Client,
    origin: Url,
    headers: header::HeaderMap,
    credentials: CredentialGuard,
    started: Instant,
    timeout: Duration,
    max_bytes: usize,
    spent: usize,
    status: Option<u16>,
}

pub fn validate_origin(value: &str) -> Result<Url, Error> {
    let url = Url::parse(value).map_err(|_| invalid("Invalid HTTP origin"))?;
    let loopback = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host
                .trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid(
            "HTTP acquisition requires an HTTPS origin or loopback HTTP without credentials, path or query",
        ));
    }
    Ok(url)
}

pub fn validate_headers(names: &BTreeMap<String, String>) -> Result<(), Error> {
    if names.len() > 16 {
        return Err(invalid("Too many HTTP credential headers"));
    }
    let mut unique = std::collections::BTreeSet::new();
    for (name, environment) in names {
        let name = header::HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| invalid("Invalid HTTP header name"))?;
        if !unique.insert(name.as_str().to_owned())
            || matches!(
                name.as_str(),
                "host"
                    | "content-length"
                    | "transfer-encoding"
                    | "connection"
                    | "content-type"
                    | "accept"
            )
            || eal_collector_contracts::EnvironmentName::parse(environment.clone()).is_err()
        {
            return Err(invalid("Invalid or repeated HTTP credential selection"));
        }
    }
    Ok(())
}

impl JsonClient {
    pub fn new(
        origin: &str,
        headers_env: &BTreeMap<String, String>,
        bearer_env: Option<&str>,
        started: Instant,
        timeout: Duration,
        max_bytes: usize,
    ) -> Result<Self, Error> {
        let origin = validate_origin(origin)?;
        validate_headers(headers_env)?;
        if timeout.is_zero()
            || timeout > Duration::from_secs(3600)
            || !(256..=16_777_216).contains(&max_bytes)
        {
            return Err(invalid("Invalid HTTP acquisition allowances"));
        }
        let mut headers = header::HeaderMap::new();
        let mut secrets = Vec::new();
        for (name, environment) in headers_env {
            let value = credential(environment)?;
            headers.insert(
                header::HeaderName::from_bytes(name.as_bytes())
                    .map_err(|_| invalid("Invalid HTTP header name"))?,
                header::HeaderValue::from_str(&value)
                    .map_err(|_| invalid("Invalid HTTP credential value"))?,
            );
            secrets.push(value);
        }
        if let Some(name) = bearer_env {
            if headers.contains_key(header::AUTHORIZATION) {
                return Err(invalid("Repeated HTTP authorization selection"));
            }
            let value = credential(name)?;
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {value}"))
                    .map_err(|_| invalid("Invalid HTTP bearer value"))?,
            );
            secrets.push(value);
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .timeout(timeout)
            .build()
            .map_err(|_| invalid("Cannot construct HTTP acquisition client"))?;
        Ok(Self {
            client,
            origin,
            headers,
            credentials: CredentialGuard { secrets },
            started,
            timeout,
            max_bytes,
            spent: 0,
            status: None,
        })
    }

    pub fn status(&self) -> Option<u16> {
        self.status
    }

    pub fn url(&self, path: &str) -> Result<Url, Error> {
        if !path.starts_with('/')
            || path.starts_with("//")
            || path.contains(['?', '#', '\\'])
            || path.split('/').any(|part| matches!(part, "." | ".."))
        {
            return Err(invalid("HTTP operation requires a literal absolute path"));
        }
        let mut url = self.origin.clone();
        url.set_path(path);
        Ok(url)
    }

    pub fn inspect(&self, value: &Value) -> Result<(), Error> {
        self.credentials.inspect(value)
    }
    pub fn into_credentials(self) -> CredentialGuard {
        self.credentials
    }

    pub async fn request(
        &mut self,
        method: Method,
        url: Url,
        body: Option<&Value>,
    ) -> Result<Value, Error> {
        if url.origin() != self.origin.origin()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid("HTTP request differs from its operator origin"));
        }
        if !matches!(method, Method::GET | Method::POST) {
            return Err(invalid("Unsupported HTTP evidence operation"));
        }
        let remaining = self
            .timeout
            .checked_sub(self.started.elapsed())
            .filter(|time| !time.is_zero())
            .ok_or_else(|| invalid("HTTP acquisition deadline exhausted"))?;
        let mut request = self
            .client
            .request(method, url)
            .headers(self.headers.clone())
            .header(header::ACCEPT, "application/json")
            .timeout(remaining);
        if let Some(body) = body {
            request = request.json(body);
        }
        let mut response = request.send().await.map_err(|_| {
            Error::Provider("HTTP acquisition failed or reached its deadline".into())
        })?;
        if !response.status().is_success() {
            return Err(Error::Provider(format!(
                "HTTP acquisition returned status {}",
                response.status().as_u16()
            )));
        }
        self.status = Some(response.status().as_u16());
        let available = self.max_bytes.saturating_sub(self.spent);
        if response
            .content_length()
            .is_some_and(|length| length > available as u64)
        {
            return Err(invalid("HTTP response exceeds aggregate byte allowance"));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| Error::Provider("HTTP response transfer failed".into()))?
        {
            if chunk.len() > available.saturating_sub(bytes.len()) {
                return Err(invalid("HTTP response exceeds aggregate byte allowance"));
            }
            bytes.extend_from_slice(&chunk);
        }
        self.spent += bytes.len();
        let value = eal_collector_contracts::strict_json(
            &bytes,
            JsonLimits {
                max_bytes: self.max_bytes,
                ..Default::default()
            },
        )
        .map_err(|_| invalid("HTTP response must be bounded strict JSON"))?;
        self.inspect(&value)?;
        if self.started.elapsed() >= self.timeout {
            return Err(invalid("HTTP acquisition deadline exhausted"));
        }
        Ok(value)
    }
}

fn invalid(message: &str) -> Error {
    Error::InvalidInput(message.into())
}
fn credential(name: &str) -> Result<String, Error> {
    eal_collector_contracts::EnvironmentName::parse(name.to_owned())
        .map_err(|_| invalid("Invalid HTTP credential environment name"))?;
    let value =
        std::env::var(name).map_err(|_| invalid("Configured HTTP credential is unavailable"))?;
    if value.is_empty() || value.len() > 8192 || value.chars().any(char::is_control) {
        return Err(invalid("Configured HTTP credential is malformed"));
    }
    Ok(value)
}

/// Immutable captured values used to inspect the final observation.
pub struct CredentialGuard {
    secrets: Vec<String>,
}
impl CredentialGuard {
    pub fn inspect(&self, value: &Value) -> Result<(), Error> {
        let secrets = self.secrets.iter().map(String::as_str).collect::<Vec<_>>();
        if contains_sensitive_json(value, &secrets, SensitiveJsonLimits::default())
            .map_err(|_| invalid("HTTP observation failed credential inspection"))?
        {
            return Err(invalid(
                "HTTP observation contains configured credential material",
            ));
        }
        Ok(())
    }
}
