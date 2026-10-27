//! Resolve operator-selected interfaces into inspectable pins without acquiring evidence.
use super::*;
use crate::acquisition::{self, ToolKind};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, time::Instant};

const MAX_PREPARATION_TOOLS: usize = 128;
const MAX_PREPARATION_SECONDS: u64 = 120;

impl ReasoningService {
    pub fn prepare(
        &self,
        source: &str,
        context: &Value,
        claim: Option<&str>,
    ) -> Result<Value, String> {
        self.prepare_with_lease(source, context, claim, self.acquisition_lease()?)
    }

    /// Discovery establishes the configured interface, never the truth of a claim.
    /// Generated pins are ordinary TOML fields; no receipt is needed to execute.
    pub fn prepare_with_lease(
        &self,
        source: &str,
        context: &Value,
        claim: Option<&str>,
        lease: Arc<crate::lease::AcquisitionLease>,
    ) -> Result<Value, String> {
        let deadline = Instant::now() + Duration::from_secs(MAX_PREPARATION_SECONDS);
        let source_program = self.parse(source)?;
        let selected = claim
            .map(|claim| {
                EvidencePlanner::new_with_registry(&source_program, &self.methods)?
                    .plan(claim)
                    .map(|plan| plan.evidence_ids())
            })
            .transpose()?;
        let (registry, manifest_digest) = match &self.registry_path {
            Some(path) => {
                let (registry, digest) = acquisition::ToolRegistry::load_snapshot(path)
                    .map_err(|error| error.to_string())?;
                (registry, Some(digest))
            }
            None => (self.registry.clone(), None),
        };
        let runtime = EvidenceRuntime::new(
            self.workspace.clone(),
            registry,
            self.store.clone(),
            self.methods.clone(),
            self.scheduler.clone(),
            self.supervisor.clone(),
        )?;
        let plan = runtime.admit_plan(&source_program, context, selected.as_deref())?;
        let names = plan.evidence_ids;
        let tool_names: BTreeSet<_> = names
            .iter()
            .filter_map(|name| source_program.evidence[name].tool.as_deref())
            .collect();
        if tool_names.len() > MAX_PREPARATION_TOOLS {
            return Err(format!(
                "Preparation exceeds {MAX_PREPARATION_TOOLS} distinct tools"
            ));
        }
        // Admit every binding and credential before starting the first server.
        let bindings = tool_names
            .into_iter()
            .map(|name| {
                let binding = runtime
                    .registry()
                    .binding_for(name, &source_program.tools[name].version)
                    .map_err(|error| error.to_string())?;
                let digest = plan
                    .binding_digests
                    .get(name)
                    .ok_or("Selected binding was not admitted")?
                    .clone();
                Ok((binding.clone(), digest))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut tools = serde_json::Map::new();
        for (mut binding, digest) in bindings {
            let mut receipt = json!({"kind":binding.kind.as_str(),"version":binding.version,
                "binding_digest":digest,"operation_calls":0});
            if matches!(binding.kind, ToolKind::McpStdio | ToolKind::McpHttp) {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err("Preparation deadline exhausted".into());
                }
                binding.timeout_seconds = binding.timeout_seconds.min(remaining.as_secs_f64());
                let discovered = acquisition::inspect_mcp(
                    &binding,
                    &self.workspace,
                    self.store.binding_key(),
                    &self.supervisor,
                    Some(lease.as_ref()),
                    &self.limits,
                )
                .map_err(|error| error.to_string())?;
                receipt["interface"] = discovered["interface"].clone();
                receipt["discovery_requests"] = discovered["discovery_requests"].clone();
                receipt["initialization_requests"] = discovered["initialization_requests"].clone();
                receipt["discovery_bytes"] = discovered["discovery_bytes"].clone();
                receipt["process_environment_digest"] =
                    discovered["process_environment_digest"].clone();
                receipt["pins"] = json!({
                    "input_schema_sha256":discovered["interface"]["input_schema_sha256"],
                    "output_schema_sha256":discovered["interface"]["output_schema_sha256"],
                    "server_identity":discovered["interface"]["server"],
                });
            }
            tools.insert(binding.name, receipt);
        }
        if self.manifest_digest()? != manifest_digest {
            return Err("Tool manifest changed during preparation".into());
        }
        let mut receipt = json!({"schema":"EAL/preparation/1","source_digest":source_program.source_digest,
            "executable_digest":semantic_format::executable_digest(&source_program, &self.methods)
                .map_err(|error| error.to_string())?,
            "manifest_sha256":manifest_digest,"method_registry_fingerprint":self.methods.fingerprint(),
            "context_digest":eal_core::canonical::canonical_digest(context, &self.limits)?,
            "evidence_ids":names,
            "claim":claim,"prepared_at":crate::store::utc_now(),"tools":tools,
            "operation_calls":0,"observations_created":0});
        eal_core::canonical::canonical_json_bytes(&receipt, &self.limits)?;
        let id = self.store.put("preparation", &receipt, None)?;
        receipt["preparation_id"] = json!(id);
        Ok(receipt)
    }

    fn manifest_digest(&self) -> Result<Option<String>, String> {
        self.registry_path
            .as_ref()
            .map(|path| {
                acquisition::read_regular(path, 1_048_576, "Tool manifest must be a regular file")
                    .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
                    .map_err(|error| error.to_string())
            })
            .transpose()
    }
}
