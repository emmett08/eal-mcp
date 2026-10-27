//! Durable acquisition under an explicitly selected operator registry.
mod dependencies;
mod size;

use crate::{
    acquisition::{self, AcquisitionError, ToolKind, ToolRegistry},
    lease::AcquisitionLease,
    process::Supervisor,
    scheduler::CollectionScheduler,
    store::{RunStore, utc_now},
};
use eal_core::{canonical, evaluation, methods::MethodRegistry, semantic::Program, validation};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use uuid::Uuid;

pub const OBSERVATION_SCHEMA: &str = "EAL/observation-record/1";
pub const MAX_COLLECTION_CONTEXT_BYTES: usize = 16 * 1024;
pub const MAX_COLLECTION_EVIDENCE: usize = 128;
pub const MAX_COLLECTION_OUTPUT_BYTES: usize = 128 * 1024 * 1024;
pub const MAX_COLLECTION_BYTES: usize = 32 * 1024 * 1024;

pub(crate) fn stored_collection_size(
    value: &Value,
    limits: &eal_core::ExecutionLimits,
) -> Result<usize, String> {
    size::collection_size(value, limits)
}

/// Identity independent of local declaration names. Collection additionally
/// binds exact source, evidence identity, environment and the private binding.
pub fn acquisition_request(
    program: &Program,
    evidence_id: &str,
    context: &Value,
) -> Result<Value, String> {
    let evidence = program
        .evidence
        .get(evidence_id)
        .ok_or("Undeclared evidence identifier")?;
    if !evidence.bindings.is_empty() {
        return Err("Bound acquisition request requires parent observations".into());
    }
    request_for_input(program, evidence_id, context, &evidence.input)
}

fn request_for_input(
    program: &Program,
    evidence_id: &str,
    context: &Value,
    input: &Value,
) -> Result<Value, String> {
    let evidence = program
        .evidence
        .get(evidence_id)
        .ok_or("Undeclared evidence identifier")?;
    let tool = evidence
        .tool
        .as_ref()
        .map(|name| program.tools.get(name).ok_or("Undeclared evidence tool"))
        .transpose()?;
    let tool_name = tool.map(|tool| &tool.name);
    let version = tool.map(|tool| &tool.version);
    canonical::canonical_digest(context, &program.limits)?;
    let context = evaluation::environment_context(&evidence.environment, context)?;
    Ok(json!({"tool":tool_name,"tool_version":version,"input":input,"context":context}))
}

/// One admitted selection shared by collection and interface preparation.
pub(crate) struct AdmittedPlan {
    pub evidence_ids: Vec<String>,
    pub binding_digests: BTreeMap<String, String>,
}

#[derive(Clone)]
pub struct EvidenceRuntime {
    workspace: PathBuf,
    registry: ToolRegistry,
    store: RunStore,
    method_registry: MethodRegistry,
    scheduler: CollectionScheduler,
    supervisor: Supervisor,
}
impl EvidenceRuntime {
    pub fn new(
        workspace: PathBuf,
        registry: ToolRegistry,
        store: RunStore,
        method_registry: MethodRegistry,
        scheduler: CollectionScheduler,
        supervisor: Supervisor,
    ) -> Result<Self, String> {
        let workspace = std::fs::canonicalize(workspace).map_err(|error| error.to_string())?;
        if !workspace.is_dir() {
            return Err("Workspace must be a directory".into());
        }
        Ok(Self {
            workspace,
            registry,
            store,
            method_registry,
            scheduler,
            supervisor,
        })
    }
    pub fn workspace(&self) -> &Path {
        &self.workspace
    }
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }
    pub fn store(&self) -> &RunStore {
        &self.store
    }

    /// Check the entire selected plan before the first command or observation
    /// write. Missing bindings remain eligible for durable error observations.
    pub fn preflight(
        &self,
        program: &Program,
        context: &Value,
        evidence_ids: Option<&[String]>,
    ) -> Result<Vec<String>, String> {
        self.admit_plan(program, context, evidence_ids)
            .map(|plan| plan.evidence_ids)
    }

    pub(crate) fn admit_plan(
        &self,
        program: &Program,
        context: &Value,
        evidence_ids: Option<&[String]>,
    ) -> Result<AdmittedPlan, String> {
        let diagnostics = validation::validate(program, &self.method_registry);
        if !diagnostics.is_empty() {
            return Err(format!(
                "Cannot collect evidence for an invalid program: {}",
                diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        if !context.is_object() {
            return Err("context must be a JSON object".into());
        }
        canonical::canonical_digest(context, &program.limits)?;
        if size::json_size(context, true, &program.limits)? > MAX_COLLECTION_CONTEXT_BYTES {
            return Err(format!(
                "Collection context exceeds {MAX_COLLECTION_CONTEXT_BYTES} bytes"
            ));
        }
        let names = evidence_ids.map_or_else(
            || program.evidence.keys().cloned().collect(),
            <[String]>::to_vec,
        );
        if names
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != names.len()
            || names
                .iter()
                .any(|name| !program.evidence.contains_key(name))
        {
            return Err("evidence_ids must be unique declared evidence identifiers".into());
        }
        let names = eal_core::acquisition::dependency_order(program, &names)?;
        if names.len() > program.limits.collection_evidence {
            return Err(format!(
                "Collection exceeds {} evidence requests",
                program.limits.collection_evidence
            ));
        }
        let mut output_allowance = 0usize;
        let mut binding_digests = BTreeMap::new();
        for name in &names {
            let declaration = &program.evidence[name];
            let acquisition = request_for_input(program, name, context, &declaration.input)?;
            let request = full_request(name, &declaration.environment, &acquisition);
            if size::json_size(&request, true, &program.limits)?.saturating_add(1)
                > acquisition::MAX_REQUEST_BYTES
            {
                return Err(format!(
                    "Tool request for {} exceeds {} bytes",
                    quoted(name, program)?,
                    acquisition::MAX_REQUEST_BYTES
                ));
            }
            let Some(tool_name) = &declaration.tool else {
                continue;
            };
            let version = &program.tools[tool_name].version;
            // An absent source is unavailable evidence. A present but invalid
            // operator binding is a configuration error for the complete plan.
            if !self.registry.bindings.contains_key(tool_name) {
                continue;
            }
            if let std::collections::btree_map::Entry::Vacant(entry) =
                binding_digests.entry(tool_name.clone())
            {
                let digest = self
                    .registry
                    .admit_binding(
                        tool_name,
                        version,
                        &self.workspace,
                        self.store.binding_key(),
                        &program.limits,
                    )
                    .map_err(|error| error.to_string())?;
                entry.insert(digest);
            }
            let binding = self
                .registry
                .binding_for(tool_name, version)
                .map_err(|error| error.to_string())?;
            if declaration.bindings.is_empty() {
                binding
                    .preflight_input(&request, &self.workspace)
                    .map_err(|error| error.to_string())?;
            }
            if !(1..=MAX_COLLECTION_OUTPUT_BYTES).contains(&binding.max_output_bytes) {
                return Err(format!(
                    "Tool {} has an invalid output allowance",
                    quoted(tool_name, program)?
                ));
            }
            output_allowance = output_allowance.saturating_add(binding.max_output_bytes);
            if output_allowance > MAX_COLLECTION_OUTPUT_BYTES {
                return Err(format!(
                    "Collection output allowance exceeds {MAX_COLLECTION_OUTPUT_BYTES} bytes"
                ));
            }
        }
        Ok(AdmittedPlan {
            evidence_ids: names,
            binding_digests,
        })
    }

    pub fn collect(
        &self,
        program: &Program,
        context: &Value,
        evidence_ids: Option<&[String]>,
    ) -> Result<Value, String> {
        self.collect_with_lease(program, context, evidence_ids, None)
    }
    pub fn collect_with_lease(
        &self,
        program: &Program,
        context: &Value,
        evidence_ids: Option<&[String]>,
        lease: Option<Arc<AcquisitionLease>>,
    ) -> Result<Value, String> {
        let names = self.preflight(program, context, evidence_ids)?;
        let records = self.collect_dependencies(program, context, &names, lease)?;
        let collection =
            json!({"source_digest":program.source_digest,"context":context,"records":records});
        if size::collection_size(&collection, &program.limits)? > MAX_COLLECTION_BYTES {
            return Err(format!(
                "Collection exceeds {MAX_COLLECTION_BYTES} stored bytes"
            ));
        }
        let id = self.store.put("collection", &collection, None)?;
        let mut result = collection;
        result["collection_id"] = json!(id);
        Ok(result)
    }

    fn collect_one(
        &self,
        program: &Program,
        name: &str,
        context: &Value,
        parents: &Value,
        lease: Option<&AcquisitionLease>,
    ) -> Result<Value, String> {
        let declaration = &program.evidence[name];
        let context = evaluation::environment_context(&declaration.environment, context)?;
        let declared_tool = declaration.tool.as_ref().map(|name| &program.tools[name]);
        let tool_version = declared_tool.map(|tool| &tool.version);
        let resolved = self.resolve_dependencies(program, name, parents, &context);
        let input = resolved
            .as_ref()
            .map(|value| &value.input)
            .unwrap_or(&declaration.input);
        let run_id = Uuid::new_v4().to_string();
        let started = utc_now();
        let mut record = json!({"schema":OBSERVATION_SCHEMA,"evidence_id":name,"source_digest":program.source_digest,
            "tool":declaration.tool,"tool_version":tool_version,"evidence_kind":declaration.kind,
            "environment":declaration.environment,
            "environment_fingerprint":evaluation::environment_fingerprint(&declaration.environment,&context,&program.limits)?,
            "collected_at":started,"started_at":started,"run_id":run_id,
            "input_digest":canonical::canonical_digest(input,&program.limits)?,"input":input,
            "context":context,"status":"error"});
        if let Ok(resolved) = &resolved
            && !declaration.bindings.is_empty()
        {
            record["dependencies"] = resolved.dependencies.clone();
        }
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let acquired = (|| -> Result<(), AcquisitionError> {
            let resolved = resolved
                .as_ref()
                .map_err(|error| AcquisitionError::invalid(error.clone()))?;
            let acquisition = request_for_input(program, name, &context, &resolved.input)?;
            let request = full_request(name, &declaration.environment, &acquisition);
            if size::json_size(&request, true, &program.limits)?.saturating_add(1)
                > acquisition::MAX_REQUEST_BYTES
            {
                return Err("Resolved acquisition request exceeds the byte limit".into());
            }
            record["request_digest"] =
                json!(canonical::canonical_digest(&request, &program.limits)?);
            record["acquisition_request"] = acquisition.clone();
            record["acquisition_request_digest"] =
                json!(canonical::canonical_digest(&acquisition, &program.limits)?);
            if let Some(expression) = &declaration.derivation {
                record["tool_binding_digest"] =
                    json!(eal_core::acquisition::derivation_digest(program, name)?);
                record["execution_kind"] = json!("derivation");
                let outcome = eal_core::expressions::evaluate_expression_with_limits(
                    expression,
                    &resolved.input,
                    &program.limits,
                );
                let value = outcome
                    .value
                    .ok_or_else(|| AcquisitionError::invalid(outcome.reason))?;
                if !value.is_object() {
                    return Err("Derived observation must produce a JSON object".into());
                }
                record["data_digest"] =
                    json!(canonical::canonical_digest(&value, &program.limits)?);
                record["value"] = value;
                record["collected_at"] = json!(utc_now());
                record["status"] = json!("ok");
                return Ok(());
            }
            let declared_tool = declared_tool.ok_or("Evidence requires a tool or derivation")?;
            let binding = self
                .registry
                .binding_for(&declared_tool.name, &declared_tool.version)?;
            let before = binding.binding_digest_with_limits(
                self.store.binding_key(),
                Some(&self.workspace),
                &program.limits,
            )?;
            record["tool_binding_digest"] = json!(before);
            let result = acquisition::acquire_with_limits(
                binding,
                &request,
                &self.workspace,
                self.store.binding_key(),
                &self.supervisor,
                lease,
                &program.limits,
            )?;
            stdout = result.stdout;
            stderr = result.stderr;
            if let Some(metadata) = result.metadata.as_object() {
                record
                    .as_object_mut()
                    .expect("record object")
                    .extend(metadata.clone());
            }
            if binding.binding_digest_with_limits(
                self.store.binding_key(),
                Some(&self.workspace),
                &program.limits,
            )? != before
            {
                return Err("Collector binding identity changed during acquisition".into());
            }
            if let Some(error) = result.error {
                return Err(error);
            }
            // Acquisition admits the operator-granted output allowance.
            // Assessment applies its own record byte budget to the stored
            // observation later; collecting does not refresh or assess it.
            let mut envelope_limits = program.limits.clone();
            envelope_limits.record_bytes = binding.max_output_bytes;
            let envelope = acquisition::validate_envelope_with_limits(
                &stdout,
                binding.kind == ToolKind::JsonFile,
                &context,
                &acquisition,
                &envelope_limits,
            )?;
            record["collected_at"] = if let Some(observed) = envelope.get("observed_at") {
                let observed = observed
                    .as_str()
                    .ok_or("observed_at must be an ISO-8601 timestamp with a timezone")?;
                validation::parse_time(observed)?;
                json!(observed)
            } else {
                json!(utc_now())
            };
            record["value"] = envelope["value"].clone();
            record["data_digest"] = json!(canonical::canonical_digest(
                &envelope["value"],
                &program.limits
            )?);
            if let Some(details) = envelope.get("details") {
                record["details"] = details.clone();
            }
            record["status"] = json!("ok");
            Ok(())
        })();
        if let Err(error) = acquired {
            record["error"] = error.as_record();
        } else {
            dependencies::retain_measurement_time(&mut record, parents, declaration)?;
        }
        store_observation(&self.store, &mut record, &stdout, &stderr)?;
        Ok(record)
    }

    pub fn current_binding_digests(&self, program: &Program) -> Value {
        let mut current = json!({});
        let mut resolved: BTreeMap<(&str, &str), Option<String>> = BTreeMap::new();
        for (name, evidence) in &program.evidence {
            let digest = if evidence.derivation.is_some() {
                eal_core::acquisition::derivation_digest(program, name).ok()
            } else {
                evidence
                    .tool
                    .as_ref()
                    .and_then(|name| program.tools.get(name))
                    .and_then(|tool| {
                        resolved
                            .entry((&tool.name, &tool.version))
                            .or_insert_with(|| {
                                self.registry
                                    .binding_for(&tool.name, &tool.version)
                                    .ok()
                                    .and_then(|binding| {
                                        binding
                                            .binding_digest_with_limits(
                                                self.store.binding_key(),
                                                Some(&self.workspace),
                                                &program.limits,
                                            )
                                            .ok()
                                    })
                            })
                            .clone()
                    })
            };
            current[name] = json!(digest);
        }
        current
    }
    pub fn current_execution_digests(&self, program: &Program) -> Value {
        let mut current = json!({});
        for (name, evidence) in &program.evidence {
            let digest = evidence
                .tool
                .as_ref()
                .and_then(|name| program.tools.get(name))
                .and_then(|tool| {
                    self.registry
                        .binding_for(&tool.name, &tool.version)
                        .ok()
                        .and_then(|binding| {
                            binding
                                .process_environment_digest_with_limits(
                                    self.store.binding_key(),
                                    None,
                                    &program.limits,
                                )
                                .ok()
                                .flatten()
                        })
                });
            current[name] = json!(digest);
        }
        current
    }
}

fn quoted(name: &str, program: &Program) -> Result<String, String> {
    eal_core::expressions::diagnostic_repr(&json!(name), &program.limits)
}
pub(crate) fn full_request(name: &str, environment: &str, acquisition: &Value) -> Value {
    json!({"evidence_id":name, "environment":environment,
        "tool":acquisition["tool"], "tool_version":acquisition["tool_version"],
        "input":acquisition["input"], "context":acquisition["context"]})
}

pub fn store_observation(
    store: &RunStore,
    record: &mut Value,
    stdout: &[u8],
    stderr: &[u8],
) -> Result<(), String> {
    record["ingested_at"] = json!(utc_now());
    record["stdout_digest"] = json!(format!("{:x}", Sha256::digest(stdout)));
    record["stderr_digest"] = json!(format!("{:x}", Sha256::digest(stderr)));
    record["stdout_bytes"] = json!(stdout.len());
    record["stderr_bytes"] = json!(stderr.len());
    let run_id = record["run_id"]
        .as_str()
        .ok_or("Observation requires a run_id")?;
    store.put("observation", record, Some(run_id))?;
    Ok(())
}
