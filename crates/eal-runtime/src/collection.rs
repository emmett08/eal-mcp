//! Durable acquisition under an explicitly selected operator registry.
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
    let tool = program
        .tools
        .get(&evidence.tool)
        .ok_or("Undeclared evidence tool")?;
    canonical::canonical_digest(context, &program.limits)?;
    let context = evaluation::environment_context(&evidence.environment, context)?;
    Ok(
        json!({"tool":tool.name,"tool_version":tool.version,"input":evidence.input,"context":context}),
    )
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
        if names.len() > program.limits.collection_evidence {
            return Err(format!(
                "Collection exceeds {} evidence requests",
                program.limits.collection_evidence
            ));
        }
        let mut output_allowance = 0usize;
        for name in &names {
            let declaration = &program.evidence[name];
            let version = &program.tools[&declaration.tool].version;
            let acquisition = acquisition_request(program, name, context)?;
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
            let Ok(binding) = self.registry.binding_for(&declaration.tool, version) else {
                continue;
            };
            if !(1..=MAX_COLLECTION_OUTPUT_BYTES).contains(&binding.max_output_bytes) {
                return Err(format!(
                    "Tool {} has an invalid output allowance",
                    quoted(&declaration.tool, program)?
                ));
            }
            output_allowance = output_allowance.saturating_add(binding.max_output_bytes);
            if output_allowance > MAX_COLLECTION_OUTPUT_BYTES {
                return Err(format!(
                    "Collection output allowance exceeds {MAX_COLLECTION_OUTPUT_BYTES} bytes"
                ));
            }
        }
        Ok(names)
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
        let records = self.scheduler.run(
            &names,
            |name| {
                let declaration = &program.evidence[name];
                self.registry
                    .binding_for(&declaration.tool, &program.tools[&declaration.tool].version)
                    .is_ok_and(|binding| binding.parallel_safe)
            },
            |name| {
                // Each acquisition owns a separate strong reference. Its supervisor
                // receives only an explicitly inherited descriptor from that lease.
                let task_lease = lease.clone();
                self.collect_one(program, name, context, task_lease.as_deref())
            },
        )?;
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
        lease: Option<&AcquisitionLease>,
    ) -> Result<Value, String> {
        let declaration = &program.evidence[name];
        let context = evaluation::environment_context(&declaration.environment, context)?;
        let declared_tool = &program.tools[&declaration.tool];
        let run_id = Uuid::new_v4().to_string();
        let started = utc_now();
        let mut record = json!({"schema":OBSERVATION_SCHEMA,"evidence_id":name,"source_digest":program.source_digest,
            "tool":declaration.tool,"tool_version":declared_tool.version,"evidence_kind":declaration.kind,
            "environment":declaration.environment,
            "environment_fingerprint":evaluation::environment_fingerprint(&declaration.environment,&context,&program.limits)?,
            "collected_at":started,"started_at":started,"run_id":run_id,
            "input_digest":canonical::canonical_digest(&declaration.input,&program.limits)?,"input":declaration.input,
            "context":context,"status":"error"});
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let acquired = (|| -> Result<(), AcquisitionError> {
            let binding = self
                .registry
                .binding_for(&declaration.tool, &declared_tool.version)?;
            let before = binding.binding_digest_with_limits(
                self.store.binding_key(),
                Some(&self.workspace),
                &program.limits,
            )?;
            record["tool_binding_digest"] = json!(before);
            let acquisition = acquisition_request(program, name, &context)?;
            let request = full_request(name, &declaration.environment, &acquisition);
            record["request_digest"] =
                json!(canonical::canonical_digest(&request, &program.limits)?);
            record["acquisition_request"] = acquisition.clone();
            record["acquisition_request_digest"] =
                json!(canonical::canonical_digest(&acquisition, &program.limits)?);
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
        }
        store_observation(&self.store, &mut record, &stdout, &stderr)?;
        Ok(record)
    }

    pub fn current_binding_digests(&self, program: &Program) -> Value {
        let mut current = json!({});
        let mut resolved: BTreeMap<(&str, &str), Option<String>> = BTreeMap::new();
        for (name, evidence) in &program.evidence {
            let digest = program.tools.get(&evidence.tool).and_then(|tool| {
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
            });
            current[name] = json!(digest);
        }
        current
    }
    pub fn current_execution_digests(&self, program: &Program) -> Value {
        let mut current = json!({});
        for (name, evidence) in &program.evidence {
            let digest = program.tools.get(&evidence.tool).and_then(|tool| {
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
fn full_request(name: &str, environment: &str, acquisition: &Value) -> Value {
    json!({"evidence_id":name,"environment":environment,"tool":acquisition["tool"],
        "tool_version":acquisition["tool_version"],"input":acquisition["input"],"context":acquisition["context"]})
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
