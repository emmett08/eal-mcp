//! Bounded approved-catalogue retrieval and explicit interface/admission reporting.
use crate::{CallMetrics, DiscoveryLimits, EvidenceConfig, config::approved_http};
use eal_collector_contracts::capability::{CapabilityCatalogue, CapabilityDescriptor};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Instant};

pub const DISCOVERY_SCHEMA: &str = "eal.evidence-discovery/1";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryGap {
    pub capability_id: Option<String>,
    pub connection_id: Option<String>,
    pub kind: String,
    pub message: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceCandidate {
    pub connection_id: String,
    pub tool: String,
    pub input_schema_sha256: String,
    pub output_schema_sha256: Option<String>,
    pub qualified: bool,
}
/// Conservative allocations used to bound later phases. Failed/unmetered
/// attempts consume their complete reserved allowances; successful attempts use
/// actual JSON-RPC/list-page and logical-payload counts. These are not wire bytes.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryBudgetUsage {
    pub requests_upper_bound: u64,
    pub bytes_upper_bound: u64,
    pub pages_by_connection: BTreeMap<String, u64>,
}
impl DiscoveryBudgetUsage {
    pub(crate) fn add(&mut self, other: &Self) {
        self.requests_upper_bound = self
            .requests_upper_bound
            .saturating_add(other.requests_upper_bound);
        self.bytes_upper_bound = self
            .bytes_upper_bound
            .saturating_add(other.bytes_upper_bound);
        for (connection, pages) in &other.pages_by_connection {
            let entry = self
                .pages_by_connection
                .entry(connection.clone())
                .or_default();
            *entry = entry.saturating_add(*pages);
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryReport {
    pub schema: String,
    /// Effective discovery bounds, including any operator-bounded reductions.
    pub limits: DiscoveryLimits,
    /// Includes qualified candidates needing configuration; inspect admitted IDs/gaps.
    pub catalogue: CapabilityCatalogue,
    pub admitted_capabilities: Vec<String>,
    pub interfaces: Vec<InterfaceCandidate>,
    pub gaps: Vec<DiscoveryGap>,
    pub incomplete: bool,
    pub metrics: CallMetrics,
    pub budget_usage: DiscoveryBudgetUsage,
}

impl DiscoveryReport {
    /// Retain selected candidates without opening a source, native artifact or registry.
    pub fn exhausted(config: &EvidenceConfig, limits: DiscoveryLimits) -> Result<Self, String> {
        config.validate()?;
        if limits.requests > config.discovery.requests
            || limits.bytes > config.discovery.bytes
            || limits.candidates > config.discovery.candidates
            || limits.pages_per_connection > config.discovery.pages_per_connection
            || limits.timeout_seconds > config.discovery.timeout_seconds
        {
            return Err("Discovery reductions exceed operator-configured bounds".into());
        }
        let native = CapabilityCatalogue {
            schema: eal_collector_contracts::capability::CAPABILITY_CATALOGUE_SCHEMA.into(),
            capabilities: config
                .native_providers
                .iter()
                .map(|provider| provider.capability.clone())
                .collect(),
        };
        let descriptors = combine(
            config
                .local_catalogues
                .iter()
                .cloned()
                .chain(std::iter::once(native)),
        )?;
        if descriptors.len() > limits.candidates {
            return Err("Selected capability catalogue exceeds the candidate budget".into());
        }
        Ok(Self {
            schema: DISCOVERY_SCHEMA.into(),
            limits,
            catalogue: CapabilityCatalogue {
                schema: eal_collector_contracts::capability::CAPABILITY_CATALOGUE_SCHEMA.into(),
                capabilities: descriptors,
            },
            admitted_capabilities: vec![],
            interfaces: vec![],
            gaps: vec![DiscoveryGap {
                capability_id: None,
                connection_id: None,
                kind: "discovery_budget".into(),
                message: "The remaining discovery budget cannot admit another bounded request"
                    .into(),
            }],
            incomplete: true,
            metrics: CallMetrics::default(),
            budget_usage: DiscoveryBudgetUsage::default(),
        })
    }
}

pub(crate) fn fetch_catalogue(
    url: &str,
    max_bytes: usize,
    deadline: Instant,
) -> Result<(CapabilityCatalogue, u64), String> {
    approved_http(url)?;
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err("Approved catalogue discovery deadline exhausted".into());
    }
    let url = url.to_owned();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| "Cannot start approved catalogue retrieval")?;
        runtime.block_on(async move {
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(remaining)
                .build()
                .map_err(|_| "Cannot configure approved catalogue retrieval")?;
            let mut response = client
                .get(url)
                .send()
                .await
                .map_err(|_| "Approved catalogue request failed")?;
            if !response.status().is_success() {
                return Err("Approved catalogue returned a failed HTTP status".into());
            }
            if response
                .content_length()
                .is_some_and(|size| size > max_bytes as u64)
            {
                return Err("Approved catalogue exceeds discovery byte budget".into());
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| "Approved catalogue read failed")?
            {
                if chunk.len() > max_bytes.saturating_sub(bytes.len()) {
                    return Err("Approved catalogue exceeds discovery byte budget".into());
                }
                bytes.extend_from_slice(&chunk);
            }
            let value = eal_collector_contracts::strict_json_exact(
                &bytes,
                eal_collector_contracts::JsonLimits {
                    max_bytes,
                    ..Default::default()
                },
            )?;
            let catalogue: CapabilityCatalogue = serde_json::from_value(value)
                .map_err(|_| "Approved catalogue has an invalid evidence capability format")?;
            catalogue.validate()?;
            Ok((catalogue, bytes.len() as u64))
        })
    })
    .join()
    .map_err(|_| "Approved catalogue retrieval failed".to_owned())?
}

pub(crate) fn combine(
    catalogues: impl IntoIterator<Item = CapabilityCatalogue>,
) -> Result<Vec<CapabilityDescriptor>, String> {
    let mut found = std::collections::BTreeMap::new();
    for catalogue in catalogues {
        catalogue.validate()?;
        for descriptor in catalogue.capabilities {
            if let Some(old) = found.get(&descriptor.id) {
                if eal_collector_contracts::capability::canonical_identity(old)?
                    != eal_collector_contracts::capability::canonical_identity(&descriptor)?
                {
                    return Err("Conflicting evidence capability IDs in selected catalogues".into());
                }
            } else {
                found.insert(descriptor.id.clone(), descriptor);
            }
        }
    }
    Ok(found.into_values().collect())
}
