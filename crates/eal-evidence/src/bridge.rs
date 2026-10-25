//! Admit qualified MCP capabilities and acquire exact measurements through persistent sessions.
use crate::{
    config::{ConnectionConfig, EvidenceConfig, ObservationTime, TransportConfig},
    discovery::{self, DiscoveryBudgetUsage, DiscoveryGap, DiscoveryReport, InterfaceCandidate},
    normalization,
};
use eal_collector_contracts::{
    AcquisitionRequest, ObservationEnvelope,
    capability::{
        CAPABILITY_CATALOGUE_SCHEMA, CapabilityCatalogue, CapabilityDescriptor, CoverageGuarantee,
        TransportKind, canonical_identity, schema_identity,
    },
};
use eal_mcp::{
    client::{ClientTarget, StdioTarget},
    persistent_mcp::{McpError, PersistentMcp, ToolDiscovery, ToolListLimits, ToolSchemaPins},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeErrorKind {
    Configuration,
    SchemaDrift,
    Acquisition,
    Budget,
    Protocol,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeError {
    pub kind: BridgeErrorKind,
    pub message: String,
}
impl BridgeError {
    pub(crate) fn new(kind: BridgeErrorKind, message: &str) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}
impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for BridgeError {}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallMetrics {
    /// JSON-RPC requests plus approved-catalogue GET requests; excludes notifications.
    pub requests: u64,
    /// Logical JSON argument/catalogue/result bytes, excluding HTTP/TLS framing.
    pub request_bytes: u64,
    pub response_bytes: u64,
    pub elapsed_ms: u64,
    pub server_starts: u64,
}
impl CallMetrics {
    fn add(&mut self, other: &Self) {
        self.requests = self.requests.saturating_add(other.requests);
        self.request_bytes = self.request_bytes.saturating_add(other.request_bytes);
        self.response_bytes = self.response_bytes.saturating_add(other.response_bytes);
        self.elapsed_ms = self.elapsed_ms.saturating_add(other.elapsed_ms);
        self.server_starts = self.server_starts.saturating_add(other.server_starts);
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeIdentity {
    pub provider_id: String,
    pub capability_id: String,
    pub capability_version: String,
    pub input_schema_sha256: String,
    pub output_schema_sha256: String,
    pub configuration_sha256: String,
    pub authentication_scope_sha256: String,
    pub qualified_contract_sha256: String,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub scope: Value,
}
#[derive(Clone, Debug)]
pub struct BridgeBinding {
    pub identity: BridgeIdentity,
    pub arguments: Value,
    pub descriptor: CapabilityDescriptor,
    pub admission_metrics: CallMetrics,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeObservation {
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub value: Value,
    pub observed_at: String,
    pub source_identity: String,
    pub coverage: Vec<CoverageGuarantee>,
    pub metrics: CallMetrics,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub provenance: Value,
}
/// Persistent connections and per-capability concurrency/rate budgets are shared
/// by clones of this bridge. Separate bridge instances/processes have separate budgets.
#[derive(Clone)]
pub struct EvidenceBridge {
    inner: Arc<BridgeState>,
}
struct BridgeState {
    config: EvidenceConfig,
    environment: BTreeMap<String, String>,
    sessions: BTreeMap<String, Mutex<Option<Arc<PersistentMcp>>>>,
    catalogues: Mutex<Option<LoadedCatalogues>>,
    gates: Mutex<BTreeMap<String, Arc<gate::CapabilityGate>>>,
}
#[derive(Clone)]
struct LoadedCatalogues {
    descriptors: Vec<CapabilityDescriptor>,
    gaps: Vec<DiscoveryGap>,
    metrics: CallMetrics,
    budget_usage: DiscoveryBudgetUsage,
    incomplete: bool,
}
fn error(kind: BridgeErrorKind, message: &str) -> BridgeError {
    BridgeError::new(kind, message)
}
fn lock<T>(value: &Mutex<T>) -> Result<MutexGuard<'_, T>, BridgeError> {
    value.lock().map_err(|_| {
        error(
            BridgeErrorKind::Protocol,
            "Evidence bridge state is unavailable",
        )
    })
}
fn elapsed(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}
fn protocol_error(error_value: McpError) -> BridgeError {
    match error_value {
        McpError::Rejected(message) if message.contains("schema identity changed") => error(
            BridgeErrorKind::SchemaDrift,
            "MCP tool schema changed after qualification",
        ),
        McpError::Rejected(message) if message.contains("budget") || message.contains("limit") => {
            error(
                BridgeErrorKind::Budget,
                "MCP discovery or request budget exhausted",
            )
        }
        McpError::Rejected(message) if message.starts_with("MCP result") => error(
            BridgeErrorKind::Protocol,
            "MCP result differs from the declared server output contract",
        ),
        McpError::Rejected(_) => error(
            BridgeErrorKind::Configuration,
            "MCP interface rejected the selected capability or arguments",
        ),
        McpError::Terminal(_) => error(
            BridgeErrorKind::Acquisition,
            "MCP connection failed or timed out; no request was retried",
        ),
    }
}
mod acquisition;
mod connection;
mod discovery_policy;
mod gate;

impl EvidenceBridge {
    pub fn new(
        config: EvidenceConfig,
        environment: BTreeMap<String, String>,
    ) -> Result<Self, String> {
        config.validate()?;
        discovery::combine(config.local_catalogues.clone())?;
        let sessions = config
            .connections
            .iter()
            .map(|connection| (connection.id.clone(), Mutex::new(None)))
            .collect();
        Ok(Self {
            inner: Arc::new(BridgeState {
                config,
                environment,
                sessions,
                catalogues: Mutex::new(None),
                gates: Mutex::new(BTreeMap::new()),
            }),
        })
    }
    pub fn config(&self) -> &EvidenceConfig {
        &self.inner.config
    }
    pub fn call(
        &self,
        capability_id: &str,
        arguments: Value,
        scope: Value,
        deadline: Instant,
    ) -> Result<BridgeObservation, BridgeError> {
        let binding = self.bind_acquisition_until(capability_id, arguments, scope, deadline)?;
        self.execute_bound(&binding, deadline)
    }
    /// Command-collector facade preserving exact EAL request/context correspondence.
    pub fn observe(
        &self,
        capability_id: &str,
        request: &AcquisitionRequest,
        scope: Value,
        deadline: Instant,
    ) -> Result<ObservationEnvelope, BridgeError> {
        request.validate().map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Invalid EAL acquisition request",
            )
        })?;
        let observation = self.call(capability_id, request.input.clone(), scope, deadline)?;
        Ok(ObservationEnvelope {
            value: observation.value,
            observed_at: Some(observation.observed_at),
            context: Some(request.context.clone()),
            request: Some(request.acquisition()),
            details: Some(
                json!({"source_identity":observation.source_identity,"coverage":observation.coverage,"provenance":observation.provenance,"metrics":observation.metrics}),
            ),
        })
    }
    pub fn diagnostics(&self) -> Value {
        json!(
            self.inner
                .sessions
                .iter()
                .filter_map(|(name, cell)| cell.lock().ok().and_then(|session| session
                    .as_ref()
                    .map(|session| (name.clone(), session.diagnostics()))))
                .collect::<BTreeMap<_, _>>()
        )
    }
}
