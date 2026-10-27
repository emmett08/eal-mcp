use std::collections::BTreeMap;
pub fn validate_bearer_token(token: &str) -> Result<(), String> {
    if token.is_empty() || !token.bytes().all(|c| (33..=126).contains(&c)) {
        Err("HTTP bearer token must contain only non-whitespace printable ASCII characters".into())
    } else {
        Ok(())
    }
}
pub fn read_bearer_token(
    name: Option<&str>,
    environ: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let Some(name) = name else {
        return Ok(None);
    };
    if !eal_collector_contracts::environment_name(name) {
        return Err("token_env must name an environment variable".into());
    }
    let token = environ.get(name).filter(|s| !s.is_empty()).ok_or_else(|| {
        format!("HTTP bearer token environment variable {name} is missing or empty")
    })?;
    validate_bearer_token(token).map_err(|_| {
        format!("HTTP bearer token environment variable {name} contains invalid characters")
    })?;
    Ok(Some(token.clone()))
}

pub fn loopback_host(host: &str) -> bool {
    host.trim_end_matches('.').eq_ignore_ascii_case("localhost")
        || host
            .trim_matches(['[', ']'])
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}
