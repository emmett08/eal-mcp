//! Validated provider-specific operator selections and shared resource allowances.
mod cli;
use std::{collections::BTreeSet, path::PathBuf, time::Duration};

pub const HELP: &str = "eal-collector describe [--catalogue-version 1|2]\n\
eal-collector kubernetes|github|tempo [OPTIONS]\n\
Shared --contract-version 1|2 --timeout-seconds 30 --max-bytes 1048576\n\
Commands --executable ABSOLUTE --supervisor ABSOLUTE [--inherit-env NAME ...]\n\
Kubernetes --inventory ABSOLUTE --resource RESOURCE --kind KIND --name NAME\n\
           [--scope namespaced --namespace NAME | --scope cluster]\n\
GitHub --repository OWNER/REPO --commit EXACT_40_HEX_SHA\n\
Tempo --inventory ABSOLUTE --url https://HOST/api/search --kind KIND --name NAME\n\
      [--cluster-attribute resource.k8s.cluster.name] [--kind-attribute span.dpe.rollout_kind]\n\
      [--name-attribute span.dpe.rollout_name] [--token-env NAME] [--tenant-env NAME]\n\
      [--max-window-seconds 86400] [--limit 20]\n\
Version 1 is the compatibility default. Version 2 explicitly describes Kubernetes scope.\n\
Credentials are read only from operator-named environment variables.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Kubernetes,
    Github,
    Tempo,
}
impl Provider {
    pub fn name(self) -> &'static str {
        match self {
            Self::Kubernetes => "kubernetes",
            Self::Github => "github",
            Self::Tempo => "tempo",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractVersion {
    V1,
    V2,
}
impl ContractVersion {
    pub fn name(self) -> &'static str {
        match self {
            Self::V1 => "1",
            Self::V2 => "2",
        }
    }
}
#[derive(Clone, Debug)]
pub struct CollectorLimits {
    timeout: Duration,
    max_bytes: usize,
}
impl CollectorLimits {
    pub fn new(timeout: Duration, max_bytes: usize) -> Result<Self, String> {
        if timeout < Duration::from_secs(1)
            || timeout > Duration::from_secs(3600)
            || !(256..=16_777_216).contains(&max_bytes)
        {
            return Err("Invalid collector process allowances".into());
        }
        Ok(Self { timeout, max_bytes })
    }
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }
}
impl Default for CollectorLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_bytes: 1_048_576,
        }
    }
}
#[derive(Clone, Debug)]
pub struct CommandConfig {
    pub executable: PathBuf,
    pub supervisor: PathBuf,
    pub inherit_env: Vec<String>,
}
impl CommandConfig {
    fn validate(&self) -> Result<(), String> {
        absolute(&self.executable)?;
        absolute(&self.supervisor)?;
        if self.inherit_env.len() > 64
            || self.inherit_env.iter().any(|n| !environment_name(n))
            || self.inherit_env.iter().collect::<BTreeSet<_>>().len() != self.inherit_env.len()
        {
            return Err("Invalid or repeated command environment name".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceScope {
    Namespaced(String),
    Cluster,
}
#[derive(Clone, Debug)]
pub struct KubernetesConfig {
    pub command: CommandConfig,
    pub inventory: PathBuf,
    pub resource: String,
    pub kind: String,
    pub name: String,
    pub scope: ResourceScope,
}
impl KubernetesConfig {
    fn validate(&self) -> Result<(), String> {
        self.command.validate()?;
        absolute(&self.inventory)?;
        for s in [&self.resource, &self.kind, &self.name] {
            selector(s)?;
        }
        if let ResourceScope::Namespaced(namespace) = &self.scope {
            selector(namespace)?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct GithubConfig {
    pub command: CommandConfig,
    pub repository: String,
    pub commit: String,
}
impl GithubConfig {
    fn validate(&self) -> Result<(), String> {
        self.command.validate()?;
        if self.repository.split('/').count() != 2
            || self
                .repository
                .split('/')
                .any(|s| selector(s).is_err() || s == "." || s == "..")
        {
            return Err("Repository must be OWNER/REPO".into());
        }
        if self.commit.len() != 40 || !self.commit.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("GitHub requires an exact 40-character commit SHA".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct TempoConfig {
    pub inventory: PathBuf,
    pub url: url::Url,
    pub kind: String,
    pub name: String,
    pub cluster_attribute: String,
    pub kind_attribute: String,
    pub name_attribute: String,
    pub token_env: Option<String>,
    pub tenant_env: Option<String>,
    pub max_window: Duration,
    pub limit: usize,
}
impl TempoConfig {
    fn validate(&self) -> Result<(), String> {
        absolute(&self.inventory)?;
        selector(&self.kind)?;
        selector(&self.name)?;
        if !["https", "http"].contains(&self.url.scheme())
            || self.url.host_str().is_none()
            || !self.url.username().is_empty()
            || self.url.password().is_some()
            || self.url.query().is_some()
            || self.url.fragment().is_some()
        {
            return Err("Tempo URL requires an HTTP origin and literal search path without credentials or query".into());
        }
        if self.url.scheme() == "http"
            && !self.url.host_str().is_some_and(|h| {
                h == "localhost"
                    || h.parse::<std::net::IpAddr>()
                        .is_ok_and(|ip| ip.is_loopback())
            })
        {
            return Err("Plain HTTP Tempo is permitted only on loopback".into());
        }
        for a in [
            &self.cluster_attribute,
            &self.kind_attribute,
            &self.name_attribute,
        ] {
            attribute(a)?;
        }
        if [&self.token_env, &self.tenant_env]
            .into_iter()
            .flatten()
            .any(|n| !environment_name(n))
        {
            return Err("Invalid credential environment name".into());
        }
        if self.max_window < Duration::from_secs(1)
            || self.max_window > Duration::from_secs(2_592_000)
            || !(1..=1000).contains(&self.limit)
        {
            return Err("Invalid Tempo query allowances".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub enum ProviderConfig {
    Kubernetes(KubernetesConfig),
    Github(GithubConfig),
    Tempo(TempoConfig),
}
impl ProviderConfig {
    pub fn provider(&self) -> Provider {
        match self {
            Self::Kubernetes(_) => Provider::Kubernetes,
            Self::Github(_) => Provider::Github,
            Self::Tempo(_) => Provider::Tempo,
        }
    }
    fn validate(&self) -> Result<(), String> {
        match self {
            Self::Kubernetes(c) => c.validate(),
            Self::Github(c) => c.validate(),
            Self::Tempo(c) => c.validate(),
        }
    }
    pub fn command(&self) -> Option<&CommandConfig> {
        match self {
            Self::Kubernetes(c) => Some(&c.command),
            Self::Github(c) => Some(&c.command),
            Self::Tempo(_) => None,
        }
    }
    pub fn inventory(&self) -> Option<&std::path::Path> {
        match self {
            Self::Kubernetes(c) => Some(&c.inventory),
            Self::Tempo(c) => Some(&c.inventory),
            Self::Github(_) => None,
        }
    }
}
/// Immutable admitted configuration. Programmatic records pass the same invariants as CLI input.
#[derive(Clone, Debug)]
pub struct Config {
    selected: ProviderConfig,
    limits: CollectorLimits,
    version: ContractVersion,
}
impl Config {
    pub fn new(
        selected: ProviderConfig,
        limits: CollectorLimits,
        version: ContractVersion,
    ) -> Result<Self, String> {
        selected.validate()?;
        if version == ContractVersion::V1
            && matches!(
                &selected,
                ProviderConfig::Kubernetes(KubernetesConfig {
                    scope: ResourceScope::Cluster,
                    ..
                })
            )
        {
            return Err("Kubernetes contract 1 requires an explicit namespace".into());
        }
        Ok(Self {
            selected,
            limits,
            version,
        })
    }
    pub fn provider(&self) -> Provider {
        self.selected.provider()
    }
    pub fn selected(&self) -> &ProviderConfig {
        &self.selected
    }
    pub fn limits(&self) -> &CollectorLimits {
        &self.limits
    }
    pub fn version(&self) -> ContractVersion {
        self.version
    }
    pub fn with_limits(&self, limits: CollectorLimits) -> Self {
        Self {
            limits,
            ..self.clone()
        }
    }
    pub fn parse(arguments: Vec<String>) -> Result<Self, String> {
        cli::parse(arguments)
    }
}
fn absolute(path: &std::path::Path) -> Result<(), String> {
    if !path.is_absolute() || path.as_os_str().len() > 8192 || path.to_str().is_none() {
        Err("Operator path requires a bounded absolute UTF-8 path".into())
    } else {
        Ok(())
    }
}
pub(crate) fn selector(value: &str) -> Result<(), String> {
    if value.len() > 253
        || value.is_empty()
        || value.starts_with('-')
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
    {
        Err("Invalid fixed resource selector".into())
    } else {
        Ok(())
    }
}
pub(crate) fn attribute(value: &str) -> Result<(), String> {
    let path = value
        .strip_prefix("span.")
        .or_else(|| value.strip_prefix("resource."));
    if value.len() > 253
        || path.is_none_or(|path| path.split('.').any(str::is_empty))
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._".contains(&c))
    {
        Err("TraceQL attributes require a scoped dotted identifier".into())
    } else {
        Ok(())
    }
}
pub(crate) fn environment_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    value.len() <= 1024
        && bytes
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
}
