//! Resolve operator-owned native runtime configuration before creating state.
use crate::{
    methods::{MethodExecutor, WorkerCommand},
    process::Supervisor,
    scheduler::CollectionScheduler,
    service::{ReasoningService, ServiceConfig},
};
pub use eal_collector_contracts::{TomlDocumentKind, toml_diagnostic};
use eal_core::{ExecutionLimits, methods};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct RuntimeOptions {
    pub workspace: PathBuf,
    pub registry: Option<PathBuf>,
    pub database: Option<PathBuf>,
    pub limits: Option<PathBuf>,
    pub methods: String,
    pub workers: usize,
    pub supervisor: Option<PathBuf>,
    pub method_worker: Option<PathBuf>,
}
impl RuntimeOptions {
    pub fn new(workspace: PathBuf) -> Self {
        Self {
            workspace,
            registry: None,
            database: None,
            limits: None,
            methods: "builtin".into(),
            workers: 4,
            supervisor: None,
            method_worker: None,
        }
    }
}

pub fn sibling_executable(name: &str) -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(executable
        .parent()
        .ok_or("Cannot locate executable directory")?
        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX)))
}

/// Installed supervisor selection shared by runtime and coordination frontends.
pub fn supervisor_in(directory: &Path) -> PathBuf {
    eal_process::companion_in(directory)
}

pub fn load_limits(path: Option<&Path>) -> Result<ExecutionLimits, String> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Document {
        limits: ExecutionLimits,
    }
    let Some(path) = path else {
        return Ok(ExecutionLimits::default());
    };
    let path = path.canonicalize().map_err(|error| error.to_string())?;
    let bytes = crate::acquisition::read_regular(&path, 16 * 1024, "Limits must be a regular file")
        .map_err(|e| e.to_string())?;
    if bytes.len() > 16 * 1024 {
        return Err("Limits configuration exceeds 16 KiB".into());
    }
    let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
    let document: Document = toml::from_str(text)
        .map_err(|error| toml_diagnostic(TomlDocumentKind::Limits, text, &error))?;
    document.limits.validate()?;
    Ok(document.limits)
}

pub fn build_service(options: RuntimeOptions) -> Result<ReasoningService, String> {
    let limits = load_limits(options.limits.as_deref())?;
    build_service_with_limits(options, limits)
}

/// Construct the same runtime for frontends that already admitted bounded limits.
/// The supplied limits replace file loading and still undergo runtime validation.
pub fn build_service_with_limits(
    options: RuntimeOptions,
    limits: ExecutionLimits,
) -> Result<ReasoningService, String> {
    limits.validate()?;
    let workspace = options
        .workspace
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !workspace.is_dir() {
        return Err("Workspace must be a directory".into());
    }
    let scheduler = CollectionScheduler::new(options.workers)?;
    let registry = match options.methods.as_str() {
        "builtin" => methods::default_registry().clone(),
        "engineering" => methods::engineering_registry(),
        "aspic" => eal_aspic::registry::aspic_registry(),
        "all" => eal_aspic::registry::with_aspic(&methods::engineering_registry())?,
        _ => return Err("methods must be builtin, engineering, aspic or all".into()),
    };
    let supervisor = Supervisor::new(
        options
            .supervisor
            .unwrap_or(supervisor_in(
                sibling_executable("eal-process-supervisor")?
                    .parent()
                    .ok_or("Cannot locate supervisor directory")?,
            ))
            .canonicalize()
            .map_err(|e| format!("Cannot resolve supervisor executable: {e}"))?,
    );
    let worker = options
        .method_worker
        .unwrap_or(sibling_executable("eal-method-worker")?)
        .canonicalize()
        .map_err(|e| format!("Cannot resolve method worker executable: {e}"))?;
    // Bundled algorithms are closed pure native functions; their environment is
    // deliberately empty and identical across CLI, MCP and configured workers.
    let environment = BTreeMap::new();
    let worker = WorkerCommand::new(
        vec![
            worker
                .to_str()
                .ok_or("Worker path must be UTF-8")?
                .to_owned(),
        ],
        environment,
        workspace.clone(),
    )
    .map_err(|e| e.to_string())?;
    let executor = MethodExecutor::new(supervisor.clone(), worker).map_err(|e| e.to_string())?;
    ReasoningService::new(ServiceConfig {
        workspace,
        registry_path: options.registry,
        database_path: options.database,
        method_registry: registry,
        method_executor: Some(executor),
        scheduler,
        supervisor,
        limits,
        lease_timeout: Duration::from_secs(30),
    })
}
