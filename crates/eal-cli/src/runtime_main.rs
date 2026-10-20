//! Staged native runtime driver; production catalogues and transports follow.
use eal_cli::input;
mod runtime_options;
use eal_core::methods;
use eal_runtime::{
    methods::{MethodExecutor, WorkerCommand},
    process::Supervisor,
    scheduler::CollectionScheduler,
    service::{FormalOptions, ReasoningService, ServiceConfig},
};
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf, process::ExitCode, time::Duration};

fn sibling(name: &str) -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(executable
        .parent()
        .ok_or("Cannot locate executable directory")?
        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX)))
}

fn run(options: runtime_options::Options) -> Result<Value, String> {
    let workspace = input::Workspace::new(&options.workspace)?;
    let limits = workspace.limits(options.limits.as_deref())?;
    let supervisor_path = options
        .supervisor
        .unwrap_or(sibling("eal-command-supervisor")?)
        .canonicalize()
        .map_err(|e| format!("Cannot resolve supervisor executable: {e}"))?;
    let supervisor = Supervisor::new(supervisor_path);
    let worker = options
        .method_worker
        .unwrap_or(sibling("eal-method-worker")?)
        .canonicalize()
        .map_err(|e| format!("Cannot resolve method worker executable: {e}"))?;
    let environment = std::env::vars_os()
        .map(|(key, value)| {
            Ok((
                key.into_string()
                    .map_err(|_| "Environment names must be UTF-8")?,
                value
                    .into_string()
                    .map_err(|_| "Environment values must be UTF-8")?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let worker = WorkerCommand::new(
        vec![
            worker
                .to_str()
                .ok_or("Worker path must be UTF-8")?
                .to_owned(),
        ],
        environment,
        options
            .workspace
            .canonicalize()
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let executor = MethodExecutor::new(supervisor.clone(), worker).map_err(|e| e.to_string())?;
    let registry = match options.methods.as_str() {
        "builtin" => methods::default_registry().clone(),
        "engineering" => methods::engineering_registry().clone(),
        "aspic" => eal_aspic::registry::aspic_registry().clone(),
        "all" => eal_aspic::registry::with_aspic(&methods::engineering_registry())?,
        _ => unreachable!(),
    };
    let registry = executor.supervise_registry(&registry)?;
    let service = ReasoningService::new(ServiceConfig {
        workspace: options.workspace,
        registry_path: options.registry,
        database_path: options.database,
        method_registry: registry,
        method_executor: Some(executor),
        scheduler: CollectionScheduler::new(options.workers)?,
        supervisor,
        limits: limits.clone(),
        lease_timeout: Duration::from_secs(30),
    })?;
    if options.command == "explain" {
        return service.explain(&options.source, options.claim.as_deref());
    }
    let source = workspace.source(&options.source, limits.source_bytes)?;
    let context = workspace.context(options.context.as_deref(), &limits)?;
    match options.command.as_str() {
        "validate" => Ok(service.validate(&source)),
        "format" => service.format(&source),
        "plan" => service.plan(&source, options.claim.as_deref().unwrap()),
        "collect" => match options.claim.as_deref() {
            Some(claim) => service.collect_claim(&source, &context, claim),
            None => service.collect(
                &source,
                &context,
                (!options.evidence.is_empty()).then_some(options.evidence.as_slice()),
            ),
        },
        "reason" => service.reason(
            &source,
            &context,
            options.collection.as_deref(),
            options.now.as_deref(),
        ),
        "compile-aspic" => service.compile_aspic(
            &source,
            &context,
            options.collection.as_deref().unwrap(),
            FormalOptions {
                goal: options.goal.as_deref().unwrap(),
                now: options.now.as_deref(),
                semantics: &options.semantics,
                query_mode: &options.query_mode,
                preference: options.preference.as_deref(),
            },
        ),
        _ => unreachable!(),
    }
}

fn main() -> ExitCode {
    let result = runtime_options::parse().and_then(|options| options.map(run).transpose());
    match result {
        Ok(None) => ExitCode::SUCCESS,
        Ok(Some(value)) => {
            let failure = value.get("valid") == Some(&Value::Bool(false))
                || value.get("complete") == Some(&Value::Bool(false))
                || value
                    .get("records")
                    .and_then(Value::as_object)
                    .is_some_and(|records| {
                        records.values().any(|record| record["status"] == "error")
                    });
            println!(
                "{}",
                serde_json::to_string_pretty(&value).expect("JSON value")
            );
            if failure {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            println!("{}", serde_json::json!({"error":error}));
            ExitCode::from(2)
        }
    }
}
