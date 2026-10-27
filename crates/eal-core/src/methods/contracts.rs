//! Own exact method identifiers, detached schemas and immutable registration.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

use super::schema::validate_schema;

/// Closed method-result states shared by evaluation and transport projections.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MethodResultStatus {
    Supported,
    Unsupported,
    NotRun,
    Incomplete,
}
impl MethodResultStatus {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "supported" => Some(Self::Supported),
            "unsupported" => Some(Self::Unsupported),
            "not_run" => Some(Self::NotRun),
            "incomplete" => Some(Self::Incomplete),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
            Self::NotRun => "not_run",
            Self::Incomplete => "incomplete",
        }
    }
}

/// Failure of a pure native computation; exhaustion is distinct from a negative result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MethodError {
    Invalid(String),
    Incomplete(String),
}

impl std::fmt::Display for MethodError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) | Self::Incomplete(message) => formatter.write_str(message),
        }
    }
}
impl std::error::Error for MethodError {}
impl From<String> for MethodError {
    fn from(message: String) -> Self {
        Self::Invalid(message)
    }
}
impl From<&str> for MethodError {
    fn from(message: &str) -> Self {
        Self::Invalid(message.to_owned())
    }
}

/// Trusted callbacks receive captured host budgets and return declared output objects.
/// Callbacks report exhaustion explicitly. Process supervision belongs to the runtime.
type MethodCallback =
    dyn Fn(&Value, &crate::ExecutionLimits) -> Result<Value, MethodError> + Send + Sync;
#[derive(Clone)]
pub struct MethodImplementation(Arc<MethodCallback>);
impl MethodImplementation {
    pub fn new(
        callback: impl Fn(&Value, &crate::ExecutionLimits) -> Result<Value, MethodError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self(Arc::new(callback))
    }
    pub fn call(
        &self,
        value: &Value,
        limits: &crate::ExecutionLimits,
    ) -> Result<Value, MethodError> {
        (self.0)(value, limits)
    }
}
impl std::fmt::Debug for MethodImplementation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("MethodImplementation(..)")
    }
}

/// A host declaration binding a native callback to its retained build artefact.
/// The core checks identity shape; verifying loaded artefacts belongs to the host.
#[derive(Clone, Debug)]
pub struct NativeMethodIdentity {
    pub version: String,
    pub artifact_sha256: String,
}

#[derive(Clone, Debug)]
pub struct MethodContract {
    pub identifier: String,
    pub evidence_kind: Option<String>,
    pub input_schema: Value,
    pub query_schema: Value,
    pub output_schema: Value,
    pub outputs: BTreeMap<String, String>,
    pub quantities: Vec<String>,
    pub exact_unit: bool,
    pub query_fields: Vec<String>,
    pub implementation_version: String,
    pub implementation_artifact_digest: Option<String>,
    pub timeout_seconds: f64,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub max_memory_bytes: usize,
    pub implementation: Option<MethodImplementation>,
    pub(crate) builtin_mode: Option<String>,
    executor_identity: Option<NativeMethodIdentity>,
}

impl MethodContract {
    pub(super) fn from_descriptor(identifier: &str, spec: &Value) -> Self {
        let optional = identifier.starts_with("engineering/");
        Self {
            identifier: identifier.to_owned(),
            evidence_kind: spec["evidence_kind"].as_str().map(str::to_owned),
            input_schema: spec["input_schema"].clone(),
            query_schema: spec["query_schema"].clone(),
            output_schema: spec["output_schema"].clone(),
            outputs: spec["outputs"]
                .as_object()
                .expect("output map")
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.as_str().expect("output interpretation").to_owned(),
                    )
                })
                .collect(),
            quantities: spec["quantities"]
                .as_array()
                .expect("quantities")
                .iter()
                .map(|v| v.as_str().expect("quantity").to_owned())
                .collect(),
            exact_unit: spec["exact_unit"].as_bool().expect("exact_unit"),
            query_fields: spec["query_schema"]["required"]
                .as_array()
                .expect("query required fields")
                .iter()
                .map(|v| v.as_str().expect("query field").to_owned())
                .collect(),
            implementation_version: if optional {
                "eal-rust-optional-0.1.0"
            } else {
                "eal-rust-builtin-0.1.0"
            }
            .into(),
            implementation_artifact_digest: None,
            timeout_seconds: spec["limits"]["timeout_seconds"]
                .as_f64()
                .expect("method timeout"),
            max_input_bytes: spec["limits"]["max_input_bytes"]
                .as_u64()
                .expect("input bytes") as usize,
            max_output_bytes: spec["limits"]["max_output_bytes"]
                .as_u64()
                .expect("output bytes") as usize,
            max_memory_bytes: spec["limits"]["max_memory_bytes"]
                .as_u64()
                .expect("memory bytes") as usize,
            implementation: None,
            executor_identity: None,
            builtin_mode: Some(
                identifier
                    .rsplit_once('/')
                    .expect("versioned identifier")
                    .0
                    .to_owned(),
            ),
        }
    }
    /// Construct a host-owned native contract. It is checked at registration.
    pub fn native(
        identifier: impl Into<String>,
        evidence_kind: impl Into<String>,
        input_schema: Value,
        query_schema: Value,
        output_schema: Value,
        identity: NativeMethodIdentity,
        implementation: impl Fn(&Value, &crate::ExecutionLimits) -> Result<Value, MethodError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        let query_fields = query_schema["required"]
            .as_array()
            .map(|p| {
                p.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        Self {
            identifier: identifier.into(),
            evidence_kind: Some(evidence_kind.into()),
            input_schema,
            query_schema,
            output_schema,
            outputs: BTreeMap::new(),
            quantities: Vec::new(),
            exact_unit: false,
            query_fields,
            implementation_version: identity.version,
            implementation_artifact_digest: Some(identity.artifact_sha256),
            timeout_seconds: 1.0,
            max_input_bytes: 1024 * 1024,
            max_output_bytes: 1024 * 1024,
            max_memory_bytes: 256 * 1024 * 1024,
            implementation: Some(MethodImplementation::new(implementation)),
            builtin_mode: None,
            executor_identity: None,
        }
    }

    pub fn output_type(&self, path: &str) -> Option<&str> {
        self.outputs.get(path).map(String::as_str).or_else(|| {
            let (parent, child) = path.rsplit_once('.')?;
            if !super::validation::identifier(child) {
                return None;
            }
            self.outputs.get(&format!("{parent}.*")).map(String::as_str)
        })
    }

    pub fn describe(&self) -> Value {
        let mut description = json!({"identifier":self.identifier,"evidence_kind":self.evidence_kind,"input_schema":self.input_schema,
            "query_schema":self.query_schema,"output_schema":self.output_schema,"outputs":self.outputs,"quantities":self.quantities,
            "exact_unit":self.exact_unit,"quantity_interpretation":"input measurement basis; each output separately declares its unit interpretation",
            "implementation_version":self.implementation_version,"implementation_source_digest":if self.builtin_mode.is_some(){Some(source_fingerprint())}else{None},
            "implementation_code_digest":Value::Null,"implementation_artifact_digest":self.implementation_artifact_digest,"identity_basis":if self.builtin_mode.is_some(){"rust-source-and-package-version; binary attestation is separate"}else{"host-declared native artefact SHA-256; runtime attestation is separate"},
            "limits":{"timeout_seconds":self.timeout_seconds,"max_input_bytes":self.max_input_bytes,"max_output_bytes":self.max_output_bytes,
                "max_memory_bytes":self.max_memory_bytes,"startup_timeout_seconds":5.0},
            "execution":if self.executor_identity.is_some(){"host_registered_executor"}else if self.builtin_mode.as_ref().is_some_and(|m|m.starts_with("engineering/")){"bounded_native_optional"}else if self.builtin_mode.is_some(){"bounded_native_builtin"}else{"trusted_native_callback"}});
        if let Some(identity) = &self.executor_identity {
            description["executor_identity"] =
                json!({"version":identity.version,"artifact_sha256":identity.artifact_sha256});
        }
        description
    }
}

/// Source identity is explicit; it is not an attestation of linked machine code.
fn source_fingerprint() -> &'static str {
    static DIGEST: OnceLock<String> = OnceLock::new();
    DIGEST.get_or_init(|| {
        let mut digest = Sha256::new();
        for source in [
            include_str!("../methods.rs"),
            include_str!("contracts.rs"),
            include_str!("dispatch.rs"),
            include_str!("logical.rs"),
            include_str!("numerical.rs"),
            include_str!("probabilistic.rs"),
            include_str!("query.rs"),
            include_str!("schema.rs"),
            include_str!("structural.rs"),
            include_str!("validation.rs"),
            include_str!("work.rs"),
            include_str!("builtin-contracts.json"),
            include_str!("../expressions/numbers.rs"),
            include_str!("../canonical.rs"),
            include_str!("../canonical/footprint.rs"),
            include_str!("../limits.rs"),
            include_str!("optional.rs"),
            include_str!("optional/api_load.rs"),
            include_str!("optional/reachability.rs"),
            include_str!("optional/rms.rs"),
            include_str!("optional/sampled_negative.rs"),
            include_str!("optional-contracts.json"),
            env!("CARGO_PKG_VERSION"),
        ] {
            digest.update((source.len() as u64).to_be_bytes());
            digest.update(source.as_bytes());
        }
        format!("{:x}", digest.finalize())
    })
}

#[derive(Clone, Debug, Default)]
pub struct MethodRegistry {
    contracts: BTreeMap<String, MethodContract>,
}

impl MethodRegistry {
    /// Bind an explicitly installed non-builtin method to an immutable host executor.
    /// The core records the host identity; process guarantees belong to that executor.
    pub fn with_executor(
        &self,
        identifier: &str,
        identity: NativeMethodIdentity,
        implementation: impl Fn(&Value, &crate::ExecutionLimits) -> Result<Value, MethodError>
        + Send
        + Sync
        + 'static,
    ) -> Result<Self, String> {
        if default_identifiers().contains(identifier) {
            return Err("Original built-in methods retain their native execution contracts".into());
        }
        let mut contract = self
            .get(identifier)
            .ok_or("Cannot bind an unknown method executor")?
            .clone();
        if contract.executor_identity.is_some() {
            return Err("Method executor is already bound".into());
        }
        contract.executor_identity = Some(identity);
        contract.implementation = Some(MethodImplementation::new(implementation));
        let mut registry = self.clone();
        validate_contract(&contract)?;
        registry.contracts.insert(identifier.to_owned(), contract);
        Ok(registry)
    }
    pub fn new(contracts: impl IntoIterator<Item = MethodContract>) -> Result<Self, String> {
        let mut registry = Self::default();
        for contract in contracts {
            validate_contract(&contract)?;
            if registry
                .contracts
                .insert(contract.identifier.clone(), contract)
                .is_some()
            {
                return Err("Method identifier is already registered".into());
            }
        }
        Ok(registry)
    }
    /// Return a detached registry; existing registered contracts never change.
    pub fn with_method(&self, contract: MethodContract) -> Result<Self, String> {
        if contract.builtin_mode.is_some() {
            return Err("Extensions cannot define built-in execution contracts".into());
        }
        if compiled_identifiers().contains(contract.identifier.as_str()) {
            return Err("Built-in method identities are reserved".into());
        }
        Self::new(
            self.contracts
                .values()
                .cloned()
                .chain(std::iter::once(contract)),
        )
    }
    pub(super) fn with_compiled_methods(
        &self,
        methods: impl IntoIterator<Item = MethodContract>,
    ) -> Result<Self, String> {
        Self::new(self.contracts.values().cloned().chain(methods))
    }
    pub fn get(&self, identifier: &str) -> Option<&MethodContract> {
        self.contracts.get(identifier)
    }
    pub fn describe(&self) -> Value {
        self.contracts
            .iter()
            .map(|(key, c)| (key.clone(), c.describe()))
            .collect::<serde_json::Map<_, _>>()
            .into()
    }
    pub fn fingerprint(&self) -> String {
        let limits = crate::ExecutionLimits {
            json_depth: 128,
            json_nodes: usize::MAX,
            record_bytes: usize::MAX,
            ..Default::default()
        };
        format!(
            "{:x}",
            Sha256::digest(
                crate::canonical::canonical_json_bytes(&self.describe(), &limits)
                    .expect("registered schemas have bounded depth and finite metadata")
            )
        )
    }
}

fn default_identifiers() -> BTreeSet<&'static str> {
    [
        "structured/1",
        "deductive/1",
        "inductive/1",
        "abductive/1",
        "causal/1",
        "counterfactual/1",
        "analogical/1",
        "temporal/1",
    ]
    .into_iter()
    .collect()
}
fn compiled_identifiers() -> BTreeSet<&'static str> {
    default_identifiers()
        .into_iter()
        .chain(super::optional::IDENTIFIERS)
        .collect()
}

pub fn is_method_identifier(value: &str) -> bool {
    let Some((name, version)) = value.rsplit_once('/') else {
        return false;
    };
    if version.is_empty()
        || version.starts_with('0')
        || !version.bytes().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    name.split('/').all(|part| {
        let mut chars = part.bytes();
        chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
    })
}

fn validate_contract(contract: &MethodContract) -> Result<(), String> {
    if let Some(identity) = &contract.executor_identity {
        if identity.version.trim().is_empty()
            || identity.artifact_sha256.len() != 64
            || !identity
                .artifact_sha256
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err("Method executor requires a version and lowercase SHA-256 identity".into());
        }
        if default_identifiers().contains(contract.identifier.as_str()) {
            return Err("Original built-in executors are reserved".into());
        }
    }
    if !is_method_identifier(&contract.identifier) {
        return Err("Method identifier requires a name and positive integer version".into());
    }
    if contract.implementation_version.trim().is_empty() {
        return Err("Method requires an explicit implementation version".into());
    }
    if let Some(mode) = &contract.builtin_mode {
        if !compiled_identifiers().contains(contract.identifier.as_str())
            || contract.identifier != format!("{mode}/1")
            || (contract.implementation.is_some() != contract.executor_identity.is_some())
        {
            return Err("Built-in method identities are reserved".into());
        }
        let descriptor = if contract.identifier.starts_with("engineering/") {
            &optional_specs()[&contract.identifier]
        } else {
            &builtin_specs()[&contract.identifier]
        };
        for (field, actual) in [
            ("evidence_kind", json!(contract.evidence_kind)),
            ("input_schema", contract.input_schema.clone()),
            ("query_schema", contract.query_schema.clone()),
            ("output_schema", contract.output_schema.clone()),
            ("outputs", json!(contract.outputs)),
            ("quantities", json!(contract.quantities)),
            ("exact_unit", json!(contract.exact_unit)),
        ] {
            if descriptor[field] != actual {
                return Err("Built-in method contracts cannot be replaced in place".into());
            }
        }
    } else if compiled_identifiers().contains(contract.identifier.as_str())
        || contract.implementation.is_none()
    {
        return Err(
            "Native methods require a distinct versioned identity and implementation".into(),
        );
    }
    if contract.builtin_mode.is_none()
        && !contract
            .implementation_artifact_digest
            .as_deref()
            .is_some_and(|s| {
                s.len() == 64
                    && s.bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            })
    {
        return Err(
            "Native methods require an explicit lowercase SHA-256 artefact identity".into(),
        );
    }
    if let Some(kind) = &contract.evidence_kind {
        if kind.is_empty()
            || !kind.bytes().enumerate().all(|(i, c)| {
                if i == 0 {
                    c.is_ascii_alphabetic() || c == b'_'
                } else {
                    c.is_ascii_alphanumeric() || c == b'_'
                }
            })
        {
            return Err("Method evidence kind must be an identifier".into());
        }
    } else if contract.builtin_mode.as_deref() != Some("structured") {
        return Err("Custom methods require a computational evidence kind".into());
    }
    for schema in [
        &contract.input_schema,
        &contract.query_schema,
        &contract.output_schema,
    ] {
        validate_schema(schema)?;
        if schema["type"] != "object" {
            return Err("Input, query and output contracts require object schemas".into());
        }
    }
    let query = &contract.query_schema;
    let properties = query["properties"].as_object().cloned().unwrap_or_default();
    if contract.query_fields
        != query["required"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    {
        return Err("Method query_fields must match its declared query schema".into());
    }
    let required = query["required"].as_array().cloned().unwrap_or_default();
    let required: BTreeSet<_> = required.iter().filter_map(Value::as_str).collect();
    if query["additionalProperties"] != false
        || required != properties.keys().map(String::as_str).collect()
    {
        return Err(
            "Every query property must be required and additional query properties forbidden"
                .into(),
        );
    }
    for (key, schema) in &properties {
        if contract.input_schema["properties"].get(key) != Some(schema)
            || !contract.input_schema["required"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x == key))
        {
            return Err(format!(
                "Each query field must use the identical required input schema: {key:?}"
            ));
        }
    }
    let known = [
        "pressure",
        "time",
        "length",
        "mass",
        "temperature_difference",
        "velocity",
        "volumetric_flow",
        "dimensionless",
        "probability",
        "proposition",
    ];
    if contract
        .quantities
        .iter()
        .any(|q| !known.contains(&q.as_str()))
    {
        return Err("Method requires known quantities".into());
    }
    if !(contract.timeout_seconds.is_finite()
        && 0.0 < contract.timeout_seconds
        && contract.timeout_seconds <= 60.0)
    {
        return Err("Method timeout must be in (0, 60] seconds".into());
    }
    if [contract.max_input_bytes, contract.max_output_bytes]
        .iter()
        .any(|v| !(1024..=16 * 1024 * 1024).contains(v))
    {
        return Err("Method input/output limits must be 1 KiB..16 MiB".into());
    }
    if !(64 * 1024 * 1024..=4 * 1024usize.pow(3)).contains(&contract.max_memory_bytes) {
        return Err("Method memory limit must be 64 MiB..4 GiB".into());
    }
    for (path, kind) in &contract.outputs {
        if !["boolean", "basis", "dimensionless"].contains(&kind.as_str()) {
            return Err(
                "Typed outputs require boolean, basis or dimensionless interpretation".into(),
            );
        }
        let mut schema = &contract.output_schema;
        for part in path.split('.') {
            schema = if part == "*" {
                &schema["additionalProperties"]
            } else {
                &schema["properties"][part]
            };
        }
        if if kind == "boolean" {
            schema["type"] != "boolean"
        } else {
            schema["type"] != "number" && schema["type"] != "integer"
        } {
            return Err(format!(
                "Typed output {path:?} does not match its output schema"
            ));
        }
    }
    Ok(())
}

pub fn default_registry() -> &'static MethodRegistry {
    static REGISTRY: OnceLock<MethodRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let descriptors = builtin_specs();
        let contracts = descriptors
            .as_object()
            .expect("contract map")
            .iter()
            .map(|(identifier, spec)| MethodContract::from_descriptor(identifier, spec));
        MethodRegistry::new(contracts).expect("valid frozen builtin contracts")
    })
}

fn builtin_specs() -> &'static Value {
    static SPECS: OnceLock<Value> = OnceLock::new();
    SPECS.get_or_init(|| {
        serde_json::from_str(include_str!("builtin-contracts.json"))
            .expect("frozen method contracts")
    })
}
pub(super) fn optional_specs() -> &'static Value {
    static SPECS: OnceLock<Value> = OnceLock::new();
    SPECS.get_or_init(|| {
        serde_json::from_str(include_str!("optional-contracts.json"))
            .expect("frozen optional contracts")
    })
}
