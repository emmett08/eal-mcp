//! Exercise native pure semantics with explicit source, observations and time.
mod core_options;
use core_options::Options;
use eal_cli::input;
mod aspic;

use eal_core::{
    composition, dialectic, evaluation, methods, planning::EvidencePlanner, propositions,
    semantic_format, validation,
};
use serde_json::{Value, json};
use std::io::{self, Write};
use std::process::ExitCode;

enum Output {
    Json(Value),
    Source(String),
}

fn execute(options: Options) -> Result<Output, String> {
    let workspace = input::Workspace::new(&options.workspace)?;
    let limits = workspace.limits(options.limits.as_deref())?;
    let selected_registry = match options.methods.as_str() {
        "builtin" => methods::default_registry().clone(),
        "engineering" => methods::engineering_registry(),
        "aspic" => eal_aspic::aspic_registry(),
        "all" => eal_aspic::with_aspic(&methods::engineering_registry())?,
        _ => unreachable!("method profile checked above"),
    };
    let registry = &selected_registry;
    if options.command == "describe" {
        return Ok(Output::Json(
            json!({"language":"EAL/4","methods":registry.describe(),"typed_bindings":propositions::describe_bindings(registry),"limits":limits,"scope":"pure semantic core; supplied observations; no collection or persistence"}),
        ));
    }
    let source_name = options.source.as_deref().expect("source checked above");
    if matches!(options.command.as_str(), "aspic" | "export-aspic") {
        return aspic::json_command(&workspace, source_name, &options.command, &limits)
            .map(Output::Json);
    }
    if options.command == "grounded" {
        let graph = eal_core::json::parse(
            &workspace.source(source_name, limits.record_bytes)?,
            &limits,
        )?;
        let object = graph
            .as_object()
            .ok_or("Graph must contain exactly arguments and attacks")?;
        if object.len() != 2 || !object.contains_key("arguments") || !object.contains_key("attacks")
        {
            return Err("Graph must contain exactly arguments and attacks".into());
        }
        return dialectic::solve_grounded(&graph["arguments"], &graph["attacks"], &limits)
            .map(Output::Json);
    }
    let source = workspace.source(source_name, limits.source_bytes)?;
    let origin = workspace.origin(source_name)?;
    let resolver = |path: &str| workspace.imported_source(path, limits.source_bytes);
    if options.command == "compile-aspic" {
        let context = workspace.context(options.context.as_deref(), &limits)?;
        let records = match &options.records {
            Some(name) => workspace.json_file(name, &limits)?,
            None => json!({}),
        };
        let mut compile = eal_aspic::CompileOptions::new(
            options.claim.as_deref().expect("claim checked above"),
            options.now.as_deref().expect("time checked above"),
            &context,
        );
        compile.registry = Some(registry);
        compile.limits = limits.clone();
        compile.resolver = Some(&resolver);
        compile.origin = &origin;
        compile.semantics = options.semantics.as_deref().unwrap_or("grounded");
        compile.query_mode = options.query_mode.as_deref().unwrap_or("sceptical");
        compile.preference = options.preference.as_deref();
        return aspic::compile(&source, &records, compile).map(Output::Json);
    }
    let program = composition::compile(&source, Some(&resolver), &origin, limits.clone())
        .map_err(|error| error.to_string())?;
    match options.command.as_str() {
        "compile" => {
            // JSON cannot represent the overflowing age literal retained for
            // static diagnostics. serde_json would otherwise replace it with null.
            if program
                .evidence
                .values()
                .any(|evidence| !evidence.max_age.is_finite())
            {
                return Err(
                    "Compiled max_age must be finite; use validate for static diagnostics".into(),
                );
            }
            serde_json::to_value(&program)
                .map(Output::Json)
                .map_err(|e| e.to_string())
        }
        "format" => semantic_format::format_program(&program, registry)
            .map(Output::Source)
            .map_err(|e| e.to_string()),
        "plan" => {
            let plan = EvidencePlanner::new_with_registry(&program, registry)?
                .plan(options.claim.as_deref().expect("claim checked above"))?;
            serde_json::to_value(plan)
                .map(Output::Json)
                .map_err(|e| e.to_string())
        }
        "validate" => {
            let diagnostics = validation::validate(&program, registry);
            Ok(Output::Json(
                json!({"language":program.language,"source_digest":program.source_digest,"valid":diagnostics.is_empty(),"diagnostics":diagnostics}),
            ))
        }
        "evaluate" => {
            let context = workspace.context(options.context.as_deref(), &limits)?;
            let records = match options.records {
                Some(name) => workspace.json_file(&name, &limits)?,
                None => json!({}),
            };
            evaluation::evaluate(
                &program,
                &records,
                options.now.as_deref().expect("time checked above"),
                &context,
                registry,
                None,
            )
            .map(Output::Json)
        }
        _ => unreachable!("command checked above"),
    }
}

fn run() -> Result<bool, String> {
    let Some(options) = core_options::parse()? else {
        return Ok(true);
    };
    let result = execute(options)?;
    let mut output = io::stdout().lock();
    match result {
        Output::Json(result) => {
            serde_json::to_writer_pretty(&mut output, &result)
                .map_err(|error| error.to_string())?;
            writeln!(output).map_err(|error| error.to_string())?;
            Ok(result.get("valid").and_then(Value::as_bool) != Some(false)
                && result.get("complete").and_then(Value::as_bool) != Some(false)
                && result.get("status").and_then(Value::as_str) != Some("incomplete"))
        }
        Output::Source(source) => {
            output
                .write_all(source.as_bytes())
                .map_err(|error| error.to_string())?;
            Ok(true)
        }
    }
}

fn main() -> ExitCode {
    let outcome = std::thread::Builder::new()
        .name("eal-core-cli".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(run);
    let outcome = match outcome {
        Ok(worker) => worker
            .join()
            .unwrap_or_else(|_| Err("Semantic CLI worker failed".into())),
        Err(error) => Err(format!("Cannot start semantic CLI worker: {error}")),
    };
    match outcome {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
