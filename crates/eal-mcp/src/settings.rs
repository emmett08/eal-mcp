//! Resolve trusted CLI, environment and TOML settings before creating application state.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::IpAddr,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerSettings {
    pub workspace: PathBuf,
    pub registry: Option<PathBuf>,
    pub database: Option<PathBuf>,
    pub methods: String,
    pub limits: Option<PathBuf>,
    pub known_entries: Vec<String>,
    pub exposure: String,
    pub transport: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub max_in_flight: usize,
    pub token_env: Option<String>,
    pub allowed_hosts: Vec<String>,
    pub allowed_origins: Vec<String>,
}
impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            workspace: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            registry: None,
            database: None,
            methods: "builtin".into(),
            limits: None,
            known_entries: vec![],
            exposure: "operator".into(),
            transport: "stdio".into(),
            host: "127.0.0.1".into(),
            port: 8000,
            path: "/mcp".into(),
            max_in_flight: 1,
            token_env: None,
            allowed_hosts: vec![],
            allowed_origins: vec![],
        }
    }
}
pub fn loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host.trim_end_matches('.').eq_ignore_ascii_case("localhost")
        || host.parse::<IpAddr>().is_ok_and(|p| p.is_loopback())
}
pub fn environment_name(value: &str) -> bool {
    let mut chars = value.bytes();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == b'_')
}
fn text<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .filter(|s| !s.is_empty() && s.trim() == *s)
        .ok_or_else(|| format!("{name} must be a non-empty string without surrounding whitespace"))
}
pub fn validate_host(value: &str) -> Result<(), String> {
    if value.is_empty() || value.trim() != value {
        return Err("host must be a non-empty string without surrounding whitespace".into());
    }
    if value.parse::<IpAddr>().is_ok() {
        return Ok(());
    }
    if value.trim_end_matches('.').split('.').all(|part| {
        !part.is_empty()
            && part.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
            && part.as_bytes()[0].is_ascii_alphanumeric()
            && part.as_bytes()[part.len() - 1].is_ascii_alphanumeric()
    }) {
        Ok(())
    } else {
        Err("host must be an IP address or hostname without a URL, port or wildcard".into())
    }
}
pub(crate) fn authority_hostname(authority: &str) -> Option<&str> {
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    if let Some(rest) = authority.strip_prefix('[') {
        let (host, suffix) = rest.split_once(']')?;
        if !suffix.is_empty()
            && !suffix.strip_prefix(':').is_some_and(|port| {
                !port.is_empty()
                    && port.bytes().all(|c| c.is_ascii_digit())
                    && port.parse::<u16>().is_ok_and(|p| p > 0)
            })
        {
            return None;
        }
        host.parse::<std::net::Ipv6Addr>().ok()?;
        Some(host)
    } else {
        let (host, port) = authority
            .split_once(':')
            .map(|(h, p)| (h, Some(p)))
            .unwrap_or((authority, None));
        if port.is_some_and(|p| {
            p.is_empty()
                || !p.bytes().all(|c| c.is_ascii_digit())
                || !p.parse::<u16>().is_ok_and(|p| p > 0)
        }) {
            return None;
        }
        Some(host)
    }
}
pub(crate) fn origin_hostname(value: &str) -> Option<&str> {
    let (_, rest) = value.split_once("://")?;
    authority_hostname(rest.split(['?', '#']).next()?)
}
pub fn validate_origin(value: &str) -> Result<(), String> {
    let invalid =
        || "allowed_origins must contain HTTP(S) origins without paths or wildcards".to_owned();
    if value.chars().any(|c| c.is_whitespace() || c < ' ') {
        return Err(invalid());
    }
    let url = url::Url::parse(value).map_err(|_| invalid())?;
    let host = origin_hostname(value).ok_or_else(invalid)?;
    if !["http", "https"].contains(&url.scheme())
        || url.host_str().is_none()
        || url.query().is_some_and(|s| !s.is_empty())
        || url.fragment().is_some_and(|s| !s.is_empty())
    {
        return Err(invalid());
    }
    validate_host(host).map_err(|_| invalid())
}
impl ServerSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !["builtin", "engineering", "aspic", "all"].contains(&self.methods.as_str()) {
            return Err("methods must be builtin, engineering, aspic or all; Python factories are not loaded".into());
        }
        if !["stdio", "http"].contains(&self.transport.as_str()) {
            return Err("transport must be stdio or http".into());
        }
        crate::operations::Exposure::resolve(Some(&self.exposure), &self.known_entries)?;
        if self.port == 0 {
            return Err("port must be an integer from 1 through 65535".into());
        }
        if !(1..=1024).contains(&self.max_in_flight) {
            return Err("max_in_flight must be an integer from 1 through 1024".into());
        }
        validate_host(&self.host)?;
        for (name, items) in [
            ("allowed_hosts", &self.allowed_hosts),
            ("allowed_origins", &self.allowed_origins),
        ] {
            if items.len() > 128 || items.iter().collect::<BTreeSet<_>>().len() != items.len() {
                return Err(format!("{name} must contain up to 128 unique entries"));
            }
            for item in items {
                if name == "allowed_hosts" {
                    validate_host(item)?;
                } else {
                    validate_origin(item)?;
                }
            }
        }
        if !self.path.starts_with('/')
            || self.path.starts_with("//")
            || self
                .path
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || "?#\\{}".contains(c))
            || self.path.split('/').any(|p| p == "." || p == "..")
        {
            return Err(
                "path must be an absolute HTTP path without query, fragment or traversal".into(),
            );
        }
        if self
            .token_env
            .as_deref()
            .is_some_and(|s| !environment_name(s))
        {
            return Err("token_env must name an environment variable".into());
        }
        if self.transport == "http" && self.token_env.is_none() {
            if !loopback_host(&self.host) {
                return Err("HTTP on a non-loopback host requires token_env authentication".into());
            }
            if self.allowed_hosts.iter().any(|h| !loopback_host(h))
                || self
                    .allowed_origins
                    .iter()
                    .any(|s| origin_hostname(s).is_none_or(|host| !loopback_host(host)))
            {
                return Err("HTTP with non-loopback allowed hosts or origins requires token_env authentication".into());
            }
        }
        for path in [&self.workspace]
            .into_iter()
            .chain(self.registry.iter())
            .chain(self.database.iter())
            .chain(self.limits.iter())
        {
            if !path.is_absolute() || path.to_str().is_none() {
                return Err("Server paths must be absolute UTF-8 filesystem paths".into());
            }
        }
        Ok(())
    }
}
const SERVICE: &[&str] = &[
    "workspace",
    "registry",
    "database",
    "methods",
    "limits",
    "known_entries",
];
const SERVER: &[&str] = &["transport", "exposure", "max_in_flight", "http"];
const HTTP: &[&str] = &[
    "host",
    "port",
    "path",
    "token_env",
    "allowed_hosts",
    "allowed_origins",
];
const PATHS: &[&str] = &["workspace", "registry", "database", "limits"];
fn string_list(value: &Value, name: &str) -> Result<Vec<String>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{name} must be an array"))?
        .iter()
        .map(|s| {
            s.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{name} must contain strings"))
        })
        .collect()
}
fn config(path: &Path) -> Result<BTreeMap<String, Value>, String> {
    const MAX_CONFIG_BYTES: usize = 1_048_576;
    let selected = path
        .canonicalize()
        .map_err(|_| "Cannot resolve server configuration")?;
    if std::fs::metadata(&selected)
        .map_err(|_| "Cannot inspect server configuration")?
        .len()
        > MAX_CONFIG_BYTES as u64
    {
        return Err("Server configuration exceeds 1 MiB".into());
    }
    let bytes = eal_runtime::acquisition::read_regular(
        &selected,
        MAX_CONFIG_BYTES,
        "Server configuration requires a regular file",
    )
    .map_err(|_| "Cannot read server configuration as an identified regular file")?;
    if bytes.len() > MAX_CONFIG_BYTES {
        return Err("Server configuration exceeds 1 MiB".into());
    }
    let source = String::from_utf8(bytes).map_err(|_| "Server configuration must be UTF-8")?;
    let root: toml::Value = toml::from_str(&source).map_err(|error| {
        eal_runtime::bootstrap::toml_diagnostic(
            eal_runtime::bootstrap::TomlDocumentKind::ServerConfiguration,
            &source,
            &error,
        )
    })?;
    fn table<'a>(
        v: &'a toml::Value,
        name: &str,
        allowed: &[&str],
    ) -> Result<&'a toml::map::Map<String, toml::Value>, String> {
        let t = v
            .as_table()
            .ok_or_else(|| format!("{name} must be a TOML table"))?;
        if t.keys().any(|s| !allowed.contains(&s.as_str())) {
            return Err(format!("Unknown {name} settings"));
        }
        Ok(t)
    }
    let root = table(&root, "root", &["service", "server"])?;
    let empty = toml::Value::Table(Default::default());
    let service = table(root.get("service").unwrap_or(&empty), "service", SERVICE)?;
    let server = table(root.get("server").unwrap_or(&empty), "server", SERVER)?;
    let http = table(server.get("http").unwrap_or(&empty), "server.http", HTTP)?;
    let mut result = BTreeMap::new();
    for (k, v) in service
        .iter()
        .chain(server.iter().filter(|(k, _)| k.as_str() != "http"))
        .chain(http.iter())
    {
        result.insert(k.clone(), toml_value(v));
    }
    Ok(result)
}
// TOML datetimes remain a non-string invalid setting instead of silently becoming text.
fn toml_value(value: &toml::Value) -> Value {
    match value {
        toml::Value::Datetime(_) => Value::Null,
        toml::Value::Array(values) => Value::Array(values.iter().map(toml_value).collect()),
        toml::Value::Table(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), toml_value(value)))
                .collect(),
        ),
        other => serde_json::to_value(other).expect("Scalar TOML values serialize as JSON"),
    }
}
fn resolve(
    base: &Path,
    value: &str,
    environ: &BTreeMap<String, String>,
) -> Result<PathBuf, String> {
    let expanded = if value == "~" || value.starts_with("~/") {
        let home = environ
            .get("HOME")
            .ok_or("Cannot expand home directory without HOME")?;
        PathBuf::from(home).join(value.strip_prefix("~/").unwrap_or(""))
    } else {
        PathBuf::from(value)
    };
    let path = if expanded.is_absolute() {
        expanded
    } else {
        base.join(expanded)
    };
    let mut resolved = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            other => {
                resolved.push(other.as_os_str());
                if let Ok(canonical) = resolved.canonicalize() {
                    resolved = canonical;
                }
            }
        }
    }
    if resolved.to_str().is_none() {
        return Err("Server paths must be UTF-8".into());
    }
    Ok(resolved)
}
pub fn parse_server_settings(
    argv: &[String],
    environ: &BTreeMap<String, String>,
    cwd: &Path,
) -> Result<ServerSettings, String> {
    let mut cli = BTreeMap::<String, Value>::new();
    let mut index = 0;
    while index < argv.len() {
        let argument = &argv[index];
        let (flag, inline) = argument
            .split_once('=')
            .map(|(a, b)| (a, Some(b)))
            .unwrap_or((argument.as_str(), None));
        let key = match flag {
            "--known-entry" => "known_entries",
            "--allowed-host" => "allowed_hosts",
            "--allowed-origin" => "allowed_origins",
            "--config" | "--workspace" | "--registry" | "--database" | "--methods" | "--limits"
            | "--transport" | "--exposure" | "--host" | "--port" | "--path" | "--max-in-flight"
            | "--token-env" => &flag[2..],
            _ => return Err("Invalid server configuration arguments".into()),
        }
        .replace('-', "_");
        if key != "config"
            && !SERVICE.contains(&key.as_str())
            && !SERVER.contains(&key.as_str())
            && !HTTP.contains(&key.as_str())
            || key == "http"
        {
            return Err(format!("Unknown server option {flag}"));
        }
        let raw = if let Some(raw) = inline {
            raw.to_owned()
        } else {
            index += 1;
            argv.get(index)
                .ok_or_else(|| format!("Missing value for {flag}"))?
                .clone()
        };
        let value = if ["known_entries", "allowed_hosts", "allowed_origins"].contains(&key.as_str())
        {
            let values = cli.entry(key.clone()).or_insert_with(|| json!([]));
            values.as_array_mut().unwrap().push(json!(raw));
            index += 1;
            continue;
        } else if ["port", "max_in_flight"].contains(&key.as_str()) {
            json!(
                raw.parse::<i64>()
                    .map_err(|_| format!("{key} must be an integer"))?
            )
        } else {
            json!(raw)
        };
        cli.insert(key, value);
        index += 1;
    }
    let selected = cli
        .remove("config")
        .or_else(|| environ.get("EAL_MCP_CONFIG").map(|s| json!(s)));
    let config_path = selected
        .as_ref()
        .map(|v| resolve(cwd, text(v, "config")?, environ))
        .transpose()?;
    let file = if let Some(path) = &config_path {
        config(path)?
    } else {
        BTreeMap::new()
    };
    let mut values = file.clone();
    let mut overridden = BTreeSet::new();
    for key in SERVICE
        .iter()
        .chain(SERVER.iter().filter(|s| **s != "http"))
        .chain(HTTP.iter())
    {
        if cli.contains_key(*key) {
            continue;
        }
        let env = format!("EAL_MCP_{}", key.to_uppercase());
        if let Some(raw) = environ.get(&env) {
            let value = if ["known_entries", "allowed_hosts", "allowed_origins"].contains(key) {
                eal_core::json::parse(raw, &eal_core::ExecutionLimits::default())
                    .map_err(|_| format!("{env} must be a JSON array"))?
            } else if ["port", "max_in_flight"].contains(key) {
                if raw.is_empty() || raw.trim() != raw {
                    return Err(format!("{env} must be an integer"));
                }
                json!(
                    raw.parse::<i64>()
                        .map_err(|_| format!("{env} must be an integer"))?
                )
            } else {
                json!(text(&json!(raw), &env)?)
            };
            values.insert((*key).into(), value);
            overridden.insert((*key).to_owned());
        }
    }
    for (k, v) in &cli {
        values.insert(k.clone(), v.clone());
        overridden.insert(k.clone());
    }
    let mut settings = ServerSettings {
        workspace: resolve(cwd, ".", environ)?,
        ..Default::default()
    };
    for key in PATHS {
        if let Some(v) = values.get(*key) {
            let base = if file.contains_key(*key) && !overridden.contains(*key) {
                config_path.as_ref().unwrap().parent().unwrap()
            } else {
                cwd
            };
            let path = resolve(base, text(v, key)?, environ)?;
            match *key {
                "workspace" => settings.workspace = path,
                "registry" => settings.registry = Some(path),
                "database" => settings.database = Some(path),
                "limits" => settings.limits = Some(path),
                _ => unreachable!(),
            }
        }
    }
    for (key, target) in [
        ("methods", &mut settings.methods),
        ("transport", &mut settings.transport),
        ("exposure", &mut settings.exposure),
        ("host", &mut settings.host),
        ("path", &mut settings.path),
    ] {
        if let Some(v) = values.get(key) {
            *target = text(v, key)?.to_owned();
        }
    }
    if let Some(v) = values.get("token_env") {
        settings.token_env = Some(text(v, "token_env")?.into());
    }
    if let Some(v) = values.get("port") {
        settings.port = v
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .filter(|n| *n > 0)
            .ok_or("port must be an integer from 1 through 65535")?;
    }
    if let Some(v) = values.get("max_in_flight") {
        settings.max_in_flight = v
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("max_in_flight must be an integer from 1 through 1024")?;
    }
    for (key, target) in [
        ("known_entries", &mut settings.known_entries),
        ("allowed_hosts", &mut settings.allowed_hosts),
        ("allowed_origins", &mut settings.allowed_origins),
    ] {
        if let Some(v) = values.get(key) {
            *target = string_list(v, key)?;
        }
    }
    if !values.contains_key("exposure") {
        settings.exposure = if settings.known_entries.is_empty() {
            "operator"
        } else {
            "registered"
        }
        .into();
    }
    settings.validate()?;
    Ok(settings)
}
