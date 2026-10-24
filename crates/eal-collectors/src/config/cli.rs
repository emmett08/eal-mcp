//! Conversion of raw CLI options into validated provider records.
use super::*;
use std::collections::BTreeMap;

pub(super) fn parse(arguments: Vec<String>) -> Result<Config, String> {
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
    let take = |options: &mut BTreeMap<String, String>, key: &str| -> Result<String, String> {
        options
            .remove(key)
            .filter(|v| !v.is_empty() && v.len() <= 8192)
            .ok_or_else(|| format!("Required operator option {key} is missing or oversized"))
    };
    let limits = CollectorLimits::new(
        Duration::from_secs(number(
            options.remove("--timeout-seconds").as_deref(),
            30,
            1,
            3600,
        )?),
        number(
            options.remove("--max-bytes").as_deref(),
            1_048_576,
            256,
            16_777_216,
        )? as usize,
    )?;
    let version = match options
        .remove("--contract-version")
        .as_deref()
        .unwrap_or("1")
    {
        "1" => ContractVersion::V1,
        "2" => ContractVersion::V2,
        _ => return Err("Unsupported native collector contract version".into()),
    };
    let mut command = || -> Result<CommandConfig, String> {
        Ok(CommandConfig {
            executable: take(&mut options, "--executable")?.into(),
            supervisor: take(&mut options, "--supervisor")?.into(),
            inherit_env: inherit_env.clone(),
        })
    };
    let selected = match provider {
        Provider::Kubernetes => {
            let command = command()?;
            let scope = match options.remove("--scope").as_deref().unwrap_or("namespaced") {
                "namespaced" => ResourceScope::Namespaced(take(&mut options, "--namespace")?),
                "cluster" if version == ContractVersion::V2 => ResourceScope::Cluster,
                _ => {
                    return Err(
                        "Unsupported Kubernetes resource scope for the selected contract".into(),
                    );
                }
            };
            ProviderConfig::Kubernetes(KubernetesConfig {
                command,
                inventory: take(&mut options, "--inventory")?.into(),
                resource: take(&mut options, "--resource")?,
                kind: take(&mut options, "--kind")?,
                name: take(&mut options, "--name")?,
                scope,
            })
        }
        Provider::Github => ProviderConfig::Github(GithubConfig {
            command: command()?,
            repository: take(&mut options, "--repository")?,
            commit: take(&mut options, "--commit")?,
        }),
        Provider::Tempo => {
            // Retain the previously accepted CLI flag for source compatibility.
            // HTTP credentials are selected exclusively by token_env and tenant_env.
            if inherit_env.len() > 64 {
                return Err("Too many inherited environment selections".into());
            }
            ProviderConfig::Tempo(TempoConfig {
                inventory: take(&mut options, "--inventory")?.into(),
                url: url::Url::parse(&take(&mut options, "--url")?)
                    .map_err(|_| "Invalid Tempo URL")?,
                kind: take(&mut options, "--kind")?,
                name: take(&mut options, "--name")?,
                cluster_attribute: options
                    .remove("--cluster-attribute")
                    .unwrap_or_else(|| "resource.k8s.cluster.name".into()),
                kind_attribute: options
                    .remove("--kind-attribute")
                    .unwrap_or_else(|| "span.dpe.rollout_kind".into()),
                name_attribute: options
                    .remove("--name-attribute")
                    .unwrap_or_else(|| "span.dpe.rollout_name".into()),
                token_env: options.remove("--token-env"),
                tenant_env: options.remove("--tenant-env"),
                max_window: Duration::from_secs(number(
                    options.remove("--max-window-seconds").as_deref(),
                    86400,
                    1,
                    2_592_000,
                )?),
                limit: number(options.remove("--limit").as_deref(), 20, 1, 1000)? as usize,
            })
        }
    };
    if !options.is_empty() {
        return Err("Unknown operator option for the selected collector".into());
    }
    Config::new(selected, limits, version)
}

fn number(value: Option<&str>, default: u64, min: u64, max: u64) -> Result<u64, String> {
    let value = value
        .map(str::parse)
        .transpose()
        .map_err(|_| "Invalid integer operator allowance")?
        .unwrap_or(default);
    if !(min..=max).contains(&value) {
        Err("Operator allowance is outside its supported range".into())
    } else {
        Ok(value)
    }
}
