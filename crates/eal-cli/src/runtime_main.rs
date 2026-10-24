//! Staged native runtime driver; production catalogues and transports follow.
use eal_cli::input;
mod runtime_options;
use eal_runtime::{
    bootstrap::{self, RuntimeOptions},
    service::FormalOptions,
};
use serde_json::Value;
use std::process::ExitCode;

fn run(options: runtime_options::Options) -> Result<Value, String> {
    let workspace = input::Workspace::new(&options.workspace)?;
    // Preserve this driver's bounded, workspace-relative limits input contract.
    let limits = workspace.limits(options.limits.as_deref())?;
    let service = bootstrap::build_service_with_limits(
        RuntimeOptions {
            workspace: options.workspace,
            registry: options.registry,
            database: options.database,
            limits: None,
            methods: options.methods,
            workers: options.workers,
            supervisor: options.supervisor,
            method_worker: options.method_worker,
        },
        limits,
    )?;
    let limits = service.limits();
    if options.command == "explain" {
        return service.explain(&options.source, options.claim.as_deref());
    }
    let source = workspace.source(&options.source, limits.source_bytes)?;
    let context = workspace.context(options.context.as_deref(), limits)?;
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
