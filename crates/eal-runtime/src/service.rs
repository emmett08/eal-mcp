//! Shared, host-configured operations for durable collection and assessment.
mod execution;
mod preparation;
use crate::{
    acquisition::ToolRegistry, collection::EvidenceRuntime, identity::CollectionIdentityValidator,
    lease::WorkspaceAcquisition, methods::MethodExecutor, process::Supervisor,
    scheduler::CollectionScheduler, store::RunStore,
};
use eal_core::{
    ExecutionLimits, composition, evaluation, methods::MethodRegistry, planning::EvidencePlanner,
    semantic::Program, semantic_format, validation,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

/// Only the operator constructs this configuration; source cannot select code,
/// storage paths, process environments or execution limits.
pub struct ServiceConfig {
    pub workspace: PathBuf,
    pub registry_path: Option<PathBuf>,
    pub database_path: Option<PathBuf>,
    pub method_registry: MethodRegistry,
    pub method_executor: Option<MethodExecutor>,
    pub scheduler: CollectionScheduler,
    pub supervisor: Supervisor,
    pub limits: ExecutionLimits,
    pub lease_timeout: Duration,
}

pub struct ReasoningService {
    workspace: PathBuf,
    registry_path: Option<PathBuf>,
    registry: ToolRegistry,
    store: RunStore,
    methods: MethodRegistry,
    method_executor: Option<MethodExecutor>,
    scheduler: CollectionScheduler,
    supervisor: Supervisor,
    limits: ExecutionLimits,
    acquisition: WorkspaceAcquisition,
    lease_timeout: Duration,
}

impl ReasoningService {
    pub fn new(config: ServiceConfig) -> Result<Self, String> {
        config.limits.validate()?;
        let methods = if let Some(executor) = &config.method_executor {
            executor.supervise_registry(&config.method_registry)?
        } else {
            for (id, descriptor) in config
                .method_registry
                .describe()
                .as_object()
                .ok_or("Invalid method registry description")?
            {
                if eal_core::methods::default_registry().get(id).is_none()
                    && descriptor.get("executor_identity").is_none()
                {
                    return Err("Runtime optional methods require a host-installed executor".into());
                }
            }
            config.method_registry.clone()
        };
        let workspace = config.workspace.canonicalize().map_err(|e| e.to_string())?;
        if !workspace.is_dir() {
            return Err("Workspace must be a directory".into());
        }
        let registry_path = config
            .registry_path
            .map(|path| path.canonicalize().map_err(|e| e.to_string()))
            .transpose()?;
        let registry = match &registry_path {
            Some(path) => ToolRegistry::load(path).map_err(|e| e.to_string())?,
            None => ToolRegistry::default(),
        };
        let store = RunStore::new(
            config
                .database_path
                .unwrap_or_else(|| workspace.join(".eal/runs.sqlite3")),
        )?;
        let acquisition = WorkspaceAcquisition::new(&workspace)?;
        Ok(Self {
            workspace,
            registry_path,
            registry,
            store,
            methods,
            method_executor: config.method_executor,
            scheduler: config.scheduler,
            supervisor: config.supervisor,
            limits: config.limits,
            acquisition,
            lease_timeout: config.lease_timeout,
        })
    }

    pub fn workspace(&self) -> &Path {
        &self.workspace
    }
    pub fn store(&self) -> &RunStore {
        &self.store
    }
    pub fn method_registry(&self) -> &MethodRegistry {
        &self.methods
    }
    pub fn limits(&self) -> &ExecutionLimits {
        &self.limits
    }

    pub fn describe(&self) -> Result<Value, String> {
        let mut description = crate::discovery::describe_language(&self.methods)?;
        description["execution_limits"] = self.limits.describe();
        Ok(description)
    }

    pub fn packet(&self, assessment_id: &str, claim: Option<&str>) -> Result<Value, String> {
        self.packet_with_limits(
            assessment_id,
            claim,
            crate::packets::PacketLimits::default(),
        )
    }

    /// Project the immutable assessment using caller-selected presentation limits.
    /// These limits never change evaluation or the retained observations.
    pub fn packet_with_limits(
        &self,
        assessment_id: &str,
        claim: Option<&str>,
        limits: crate::packets::PacketLimits,
    ) -> Result<Value, String> {
        limits.validate()?;
        let assessment = self.explain(assessment_id, None)?;
        if assessment["method_registry_fingerprint"] != self.methods.fingerprint() {
            return Err(
                "Stored assessment method registry differs from the current packet contract".into(),
            );
        }
        let collection = match assessment.get("collection_id").and_then(Value::as_str) {
            Some(id) => {
                let mut collection = self.store.get(id, Some("collection"))?;
                collection["collection_id"] = json!(id);
                Some(collection)
            }
            None => None,
        };
        let claims = claim.map(|claim| vec![claim.to_owned()]);
        crate::packets::AssessmentPacketBuilder::new(limits, Some(self.methods.clone()))?.build(
            &assessment,
            claims.as_deref(),
            collection.as_ref(),
        )
    }

    /// Bounded source-only imports remain inside the selected workspace.
    pub fn parse_resolver(&self, relative: &str) -> Result<String, String> {
        let root = self
            .registry_path
            .as_ref()
            .and_then(|path| path.parent())
            .unwrap_or(&self.workspace);
        let path = root
            .join(relative)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !path.starts_with(root)
            || path.extension() != Some(std::ffi::OsStr::new("eal"))
            || !path.is_file()
        {
            return Err("An import must name a workspace .eal file".into());
        }
        let bytes = crate::acquisition::read_regular(
            &path,
            self.limits.source_bytes,
            "An import must name a workspace .eal file",
        )
        .map_err(|e| e.to_string())?;
        if bytes.len() > self.limits.source_bytes {
            return Err("Imported source exceeds the host byte budget".into());
        }
        String::from_utf8(bytes).map_err(|e| e.to_string())
    }

    pub fn parse(&self, source: &str) -> Result<Program, String> {
        let resolver = |relative: &str| self.parse_resolver(relative);
        composition::compile(source, Some(&resolver), "", self.limits.clone())
            .map_err(|e| e.to_string())
    }

    pub fn validate(&self, source: &str) -> Value {
        match self.parse(source) {
            Err(message) => {
                json!({"valid":false,"diagnostics":[{"code":"syntax","message":message}]})
            }
            Ok(program) => {
                let diagnostics = validation::validate(&program, &self.methods);
                json!({"valid":diagnostics.is_empty(),"source_digest":program.source_digest,
                    "method_registry_fingerprint":self.methods.fingerprint(),"diagnostics":diagnostics})
            }
        }
    }

    pub fn format(&self, source: &str) -> Result<Value, String> {
        let formatted = semantic_format::format_program(&self.parse(source)?, &self.methods)
            .map_err(|e| e.to_string())?;
        Ok(
            json!({"source_digest":self.parse(&formatted)?.source_digest,
            "new_collection_required":formatted != source,"source":formatted}),
        )
    }

    pub fn plan(&self, source: &str, claim: &str) -> Result<Value, String> {
        let program = self.parse(source)?;
        let plan = EvidencePlanner::new_with_registry(&program, &self.methods)?.plan(claim)?;
        let closure = &plan.closure;
        Ok(json!({"claim":claim,"source_digest":program.source_digest,
            "method_registry_fingerprint":self.methods.fingerprint(),
            "evidence_ids":plan.evidence_ids(),"estimated_calls":plan.estimated_calls(),"calls":plan.calls,
            "dependencies":{
                "claims":program.claims.keys().filter(|n|closure.claims.contains(*n)).collect::<Vec<_>>(),
                "arguments":program.arguments.keys().filter(|n|closure.arguments.contains(*n)).collect::<Vec<_>>(),
                "reasoning":program.reasoning.keys().filter(|n|closure.reasoning.contains(*n)).collect::<Vec<_>>(),
                "assumptions":program.assumptions.keys().filter(|n|closure.assumptions.contains(*n)).collect::<Vec<_>>(),
                "objections":program.objections.keys().filter(|n|closure.objections.contains(*n)).collect::<Vec<_>>()}}))
    }

    pub(crate) fn runtime(&self) -> Result<EvidenceRuntime, String> {
        let registry = match &self.registry_path {
            Some(path) => ToolRegistry::load(path).map_err(|e| e.to_string())?,
            None => self.registry.clone(),
        };
        EvidenceRuntime::new(
            self.workspace.clone(),
            registry,
            self.store.clone(),
            self.methods.clone(),
            self.scheduler.clone(),
            self.supervisor.clone(),
        )
    }

    pub fn collect(
        &self,
        source: &str,
        context: &Value,
        evidence_ids: Option<&[String]>,
    ) -> Result<Value, String> {
        self.collect_with_lease(source, context, evidence_ids, self.acquisition_lease()?)
    }

    pub fn acquisition_lease(&self) -> Result<Arc<crate::lease::AcquisitionLease>, String> {
        Ok(Arc::new(self.acquisition.hold(self.lease_timeout)?))
    }

    pub fn collect_with_lease(
        &self,
        source: &str,
        context: &Value,
        evidence_ids: Option<&[String]>,
        lease: Arc<crate::lease::AcquisitionLease>,
    ) -> Result<Value, String> {
        let program = self.parse(source)?;
        self.runtime()?
            .collect_with_lease(&program, context, evidence_ids, Some(lease))
    }

    pub fn collect_claim(
        &self,
        source: &str,
        context: &Value,
        claim: &str,
    ) -> Result<Value, String> {
        self.collect_claim_with_lease(source, context, claim, self.acquisition_lease()?)
    }

    pub fn collect_claim_with_lease(
        &self,
        source: &str,
        context: &Value,
        claim: &str,
        lease: Arc<crate::lease::AcquisitionLease>,
    ) -> Result<Value, String> {
        let plan = self.plan(source, claim)?;
        let evidence_ids = plan["evidence_ids"]
            .as_array()
            .ok_or("Invalid evidence plan")?
            .iter()
            .map(|id| {
                id.as_str()
                    .map(str::to_owned)
                    .ok_or("Invalid evidence identifier")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut collection =
            self.collect_with_lease(source, context, Some(&evidence_ids), lease)?;
        collection["plan"] = plan;
        Ok(collection)
    }

    pub fn current_binding_digests(&self, program: &Program) -> Result<Value, String> {
        Ok(self.runtime()?.current_binding_digests(program))
    }
    pub fn current_execution_digests(&self, program: &Program) -> Result<Value, String> {
        Ok(self.runtime()?.current_execution_digests(program))
    }

    fn records(
        &self,
        program: &Program,
        context: &Value,
        collection_id: Option<&str>,
    ) -> Result<Value, String> {
        validation::check_json_resources(context, &self.limits)?;
        if !context.is_object() {
            return Err("Context must be a JSON object".into());
        }
        match collection_id {
            None => Ok(json!({})),
            Some(id) if id.trim().is_empty() => {
                Err("collection_id must be a nonempty string or null".into())
            }
            Some(id) => CollectionIdentityValidator.validate(
                &self.store.get(id, Some("collection"))?,
                &program.source_digest,
                context,
                &self.limits,
            ),
        }
    }

    pub fn reason(
        &self,
        source: &str,
        context: &Value,
        collection_id: Option<&str>,
        now: Option<&str>,
    ) -> Result<Value, String> {
        let program = self.parse(source)?;
        self.reason_program(&program, context, collection_id, now)
    }

    fn reason_program(
        &self,
        program: &Program,
        context: &Value,
        collection_id: Option<&str>,
        now: Option<&str>,
    ) -> Result<Value, String> {
        let records = self.records(program, context, collection_id)?;
        let now = now.map(str::to_owned).unwrap_or_else(utc_now);
        let bindings = self.current_binding_digests(program)?;
        let mut assessment = evaluation::evaluate(
            program,
            &records,
            &now,
            context,
            &self.methods,
            Some(&bindings),
        )?;
        assessment["collection_id"] = json!(collection_id);
        assessment["method_registry_fingerprint"] = json!(self.methods.fingerprint());
        let id = self.store.put("assessment", &assessment, None)?;
        assessment["assessment_id"] = json!(id);
        Ok(assessment)
    }

    pub fn compile_aspic(
        &self,
        source: &str,
        context: &Value,
        collection_id: &str,
        options: FormalOptions<'_>,
    ) -> Result<Value, String> {
        let executor = self
            .method_executor
            .as_ref()
            .ok_or("Runtime ASPIC compilation requires a supervised method executor")?;
        let program = self.parse(source)?;
        let records = self.records(&program, context, Some(collection_id))?;
        let now = options.now.map(str::to_owned).unwrap_or_else(utc_now);
        let bindings = self.current_binding_digests(&program)?;
        let resolver = |relative: &str| self.parse_resolver(relative);
        let mut compile = eal_aspic::CompileOptions::new(options.goal, &now, context);
        compile.registry = Some(&self.methods);
        compile.binding_digests = Some(&bindings);
        compile.limits = self.limits.clone();
        compile.resolver = Some(&resolver);
        compile.semantics = options.semantics;
        compile.query_mode = options.query_mode;
        compile.preference = options.preference;
        let snapshot =
            eal_aspic::compile_eal_aspic(source, &records, compile).map_err(|e| e.to_string())?;
        let contract = eal_aspic::registry::aspic_contract();
        let run = |payload: &Value, limits: &ExecutionLimits| {
            executor
                .execute(contract, payload, limits)
                .map_err(|error| match error {
                    eal_core::methods::MethodError::Invalid(message) => {
                        eal_aspic::AspicError::Invalid(message)
                    }
                    eal_core::methods::MethodError::Incomplete(message) => {
                        eal_aspic::AspicError::Incomplete(message)
                    }
                })
        };
        let mut result = snapshot
            .to_dict_with_executor(&run)
            .map_err(|e| e.to_string())?;
        result["collection_id"] = json!(collection_id);
        Ok(result)
    }

    pub fn explain(&self, assessment_id: &str, claim: Option<&str>) -> Result<Value, String> {
        let mut assessment = self.store.get(assessment_id, Some("assessment"))?;
        let Some(claim) = claim else {
            assessment["assessment_id"] = json!(assessment_id);
            return Ok(assessment);
        };
        let result = assessment
            .get("claims")
            .and_then(|claims| claims.get(claim))
            .ok_or_else(|| format!("Unknown claim {claim:?}"))?;
        let mut selected = json!({"assessment_id":assessment_id,"claim":claim,"result":result});
        for key in [
            "assessed_at",
            "source_digest",
            "method_registry_fingerprint",
            "arguments",
            "evidence",
            "assumptions",
            "reasoning",
            "objections",
        ] {
            selected[key] = assessment.get(key).cloned().unwrap_or(Value::Null);
        }
        Ok(selected)
    }
}

pub struct FormalOptions<'a> {
    pub goal: &'a str,
    pub now: Option<&'a str>,
    pub semantics: &'a str,
    pub query_mode: &'a str,
    pub preference: Option<&'a str>,
}
impl<'a> FormalOptions<'a> {
    pub fn new(goal: &'a str) -> Self {
        Self {
            goal,
            now: None,
            semantics: "grounded",
            query_mode: "sceptical",
            preference: None,
        }
    }
}

pub fn utc_now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false)
}
