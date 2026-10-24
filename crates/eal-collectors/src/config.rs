//! Operator-owned selectors and resource allowances for collector processes.
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

pub const HELP: &str = "eal-collector describe\n\
eal-collector kubernetes|github|tempo [OPTIONS]\n\
Shared --timeout-seconds 30 --max-bytes 1048576\n\
Commands --executable ABSOLUTE --supervisor ABSOLUTE [--inherit-env NAME ...]\n\
Kubernetes --inventory ABSOLUTE --resource applications.argoproj.io --kind Application --namespace NAME --name NAME\n\
GitHub --repository OWNER/REPO --commit EXACT_40_HEX_SHA\n\
Tempo --inventory ABSOLUTE --url https://HOST/api/search --kind Application --name NAME\n\
      [--cluster-attribute resource.k8s.cluster.name] [--kind-attribute span.dpe.rollout_kind]\n\
      [--name-attribute span.dpe.rollout_name] [--token-env NAME] [--tenant-env NAME]\n\
      [--max-window-seconds 86400] [--limit 20]\n\
Input is one EAL JSON request. Tempo input requires integer start_unix and end_unix.\n\
Kubernetes input is {resource,namespace,name}; GitHub input is {repository,commit}.\n\
All source parameters must match operator selectors. Credentials are read only from named environment variables.";

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

#[derive(Clone, Debug)]
pub struct Config {
    pub provider: Provider,
    pub options: BTreeMap<String, String>,
    pub inherit_env: Vec<String>,
    pub timeout: Duration,
    pub max_bytes: usize,
}
impl Config {
    pub fn parse(arguments: Vec<String>) -> Result<Self, String> {
        let mut args = arguments.into_iter();
        let provider = match args.next().as_deref() {
            Some("kubernetes") => Provider::Kubernetes,
            Some("github") => Provider::Github,
            Some("tempo") => Provider::Tempo,
            _ => return Err("Select kubernetes, github or tempo".into()),
        };
        let mut options = BTreeMap::new();
        let mut inherit_env = Vec::new();
        while let Some(key) = args.next() {
            let value = args
                .next()
                .ok_or("Every operator option requires a value")?;
            if key == "--inherit-env" {
                if !environment_name(&value) || inherit_env.contains(&value) {
                    return Err("Invalid or repeated environment name".into());
                }
                inherit_env.push(value);
            } else if options.insert(key, value).is_some() {
                return Err("Repeated operator option".into());
            }
        }
        let timeout = number(options.get("--timeout-seconds"), 30, 1, 3600)?;
        let max_bytes = number(options.get("--max-bytes"), 1_048_576, 256, 16_777_216)? as usize;
        let config = Self {
            provider,
            options,
            inherit_env,
            timeout: Duration::from_secs(timeout),
            max_bytes,
        };
        config.validate()?;
        Ok(config)
    }
    pub fn get(&self, name: &str) -> Result<&str, String> {
        self.options
            .get(name)
            .map(String::as_str)
            .filter(|s| !s.is_empty() && s.len() <= 8192)
            .ok_or_else(|| format!("Required operator option {name} is missing or oversized"))
    }
    pub fn optional<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
        self.options
            .get(name)
            .map(String::as_str)
            .unwrap_or(default)
    }
    pub fn path(&self, name: &str) -> Result<PathBuf, String> {
        let path = PathBuf::from(self.get(name)?);
        if !path.is_absolute() {
            return Err(format!("Operator option {name} requires an absolute path"));
        }
        Ok(path)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.timeout < Duration::from_secs(1)
            || self.timeout > Duration::from_secs(3600)
            || !(256..=16_777_216).contains(&self.max_bytes)
            || self.inherit_env.iter().any(|name| !environment_name(name))
            || self
                .inherit_env
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.inherit_env.len()
        {
            return Err("Invalid collector process allowances or environment selection".into());
        }
        let shared = ["--timeout-seconds", "--max-bytes"];
        let selected: &[&str] = match self.provider {
            Provider::Kubernetes => &[
                "--executable",
                "--supervisor",
                "--inventory",
                "--resource",
                "--kind",
                "--namespace",
                "--name",
            ],
            Provider::Github => &["--executable", "--supervisor", "--repository", "--commit"],
            Provider::Tempo => &[
                "--inventory",
                "--url",
                "--kind",
                "--name",
                "--cluster-attribute",
                "--kind-attribute",
                "--name-attribute",
                "--token-env",
                "--tenant-env",
                "--max-window-seconds",
                "--limit",
            ],
        };
        if self
            .options
            .keys()
            .any(|k| !shared.contains(&k.as_str()) && !selected.contains(&k.as_str()))
        {
            return Err("Unknown operator option for the selected collector".into());
        }
        if self.provider != Provider::Tempo {
            self.path("--executable")?;
            self.path("--supervisor")?;
        }
        if self.provider != Provider::Github {
            self.path("--inventory")?;
            for key in ["--kind", "--name"] {
                selector(self.get(key)?)?;
            }
        }
        match self.provider {
            Provider::Kubernetes => {
                for key in ["--resource", "--namespace"] {
                    selector(self.get(key)?)?;
                }
            }
            Provider::Github => {
                let repository = self.get("--repository")?;
                if repository.split('/').count() != 2
                    || repository
                        .split('/')
                        .any(|s| selector(s).is_err() || s == "." || s == "..")
                {
                    return Err("Repository must be OWNER/REPO".into());
                }
                let commit = self.get("--commit")?;
                if commit.len() != 40 || !commit.bytes().all(|c| c.is_ascii_hexdigit()) {
                    return Err("GitHub requires an exact 40-character commit SHA".into());
                }
            }
            Provider::Tempo => {
                let url = url::Url::parse(self.get("--url")?).map_err(|_| "Invalid Tempo URL")?;
                if !["https", "http"].contains(&url.scheme())
                    || url.host_str().is_none()
                    || !url.username().is_empty()
                    || url.password().is_some()
                    || url.query().is_some()
                    || url.fragment().is_some()
                {
                    return Err("Tempo URL requires an HTTP origin and literal search path without credentials or query".into());
                }
                if url.scheme() == "http"
                    && !url.host_str().is_some_and(|h| {
                        h == "localhost"
                            || h.parse::<std::net::IpAddr>()
                                .is_ok_and(|ip| ip.is_loopback())
                    })
                {
                    return Err("Plain HTTP Tempo is permitted only on loopback".into());
                }
                for (key, default) in [
                    ("--cluster-attribute", "resource.k8s.cluster.name"),
                    ("--kind-attribute", "span.dpe.rollout_kind"),
                    ("--name-attribute", "span.dpe.rollout_name"),
                ] {
                    attribute(self.optional(key, default))?;
                }
                for key in ["--token-env", "--tenant-env"] {
                    if let Some(value) = self.options.get(key)
                        && !environment_name(value)
                    {
                        return Err("Invalid credential environment name".into());
                    }
                }
                number(
                    self.options.get("--max-window-seconds"),
                    86400,
                    1,
                    2_592_000,
                )?;
                number(self.options.get("--limit"), 20, 1, 1000)?;
            }
        }
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
        return Err("Invalid fixed resource selector".into());
    }
    Ok(())
}
pub(crate) fn attribute(value: &str) -> Result<(), String> {
    if !(value.starts_with("span.") || value.starts_with("resource."))
        || value.len() > 253
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._".contains(&c))
    {
        return Err("TraceQL attributes require a scoped dotted identifier".into());
    }
    Ok(())
}
pub(crate) fn number(
    value: Option<&String>,
    default: u64,
    min: u64,
    max: u64,
) -> Result<u64, String> {
    let value = match value {
        Some(v) => v
            .parse()
            .map_err(|_| "Invalid integer operator allowance")?,
        None => default,
    };
    if !(min..=max).contains(&value) {
        return Err("Operator allowance is outside its supported range".into());
    }
    Ok(value)
}
pub(crate) fn environment_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
}
