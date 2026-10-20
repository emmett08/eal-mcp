//! Exercise native pure semantics with explicit source, observations and time.
mod input;

use eal_core::{
    composition, dialectic, evaluation, methods, planning::EvidencePlanner, propositions,
    semantic_format, validation,
};
use serde_json::{Value, json};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: eal-core [--workspace DIR] [--limits FILE] [--methods builtin|engineering] <COMMAND>\n\
    compile SOURCE                  Emit composed semantic entities\n\
    validate SOURCE                 Emit ordered static diagnostics\n\
    format SOURCE                   Emit checked canonical source\n\
    plan SOURCE --claim NAME        Plan complete evidence dependencies\n\
    evaluate SOURCE --now ISO_TIME [--context JSON|@FILE] [--records FILE]\n\
                                    Assess supplied observations without collecting\n\
    grounded GRAPH                  Solve arguments/attacks JSON\n\
    describe                        Describe pure method and quantity contracts\n\
SOURCE may be - for standard input. Limits use a TOML [limits] table.\n\
All file inputs and trusted imports stay within the selected workspace.";

struct Options {
    workspace: PathBuf,
    limits: Option<String>,
    command: String,
    source: Option<String>,
    context: Option<String>,
    records: Option<String>,
    now: Option<String>,
    claim: Option<String>,
    methods: String,
}

enum Output {
    Json(Value),
    Source(String),
}

fn options() -> Result<Option<Options>, String> {
    let mut supplied = Options {
        workspace: std::env::current_dir().map_err(|e| e.to_string())?,
        limits: None,
        command: String::new(),
        source: None,
        context: None,
        records: None,
        now: None,
        claim: None,
        methods: "builtin".into(),
    };
    let arguments = std::env::args_os()
        .skip(1)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| "Arguments must be UTF-8".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        if matches!(argument.as_str(), "--help" | "-h") {
            println!("{USAGE}");
            return Ok(None);
        }
        match argument.as_str() {
            "--workspace" | "--limits" | "--context" | "--records" | "--now" | "--claim"
            | "--methods" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| format!("{argument} requires a value"))?;
                match argument.as_str() {
                    "--workspace" => supplied.workspace = value.into(),
                    "--limits" => supplied.limits = Some(value),
                    "--context" => supplied.context = Some(value),
                    "--records" => supplied.records = Some(value),
                    "--now" => supplied.now = Some(value),
                    "--claim" => supplied.claim = Some(value),
                    "--methods" => supplied.methods = value,
                    _ => unreachable!(),
                }
            }
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("Unknown option {value:?}\n{USAGE}"));
            }
            _ if supplied.command.is_empty() => supplied.command = argument,
            _ if supplied.source.is_none() => supplied.source = Some(argument),
            _ => return Err(USAGE.into()),
        }
    }
    if !matches!(
        supplied.command.as_str(),
        "compile" | "validate" | "format" | "plan" | "evaluate" | "grounded" | "describe"
    ) || !matches!(supplied.methods.as_str(), "builtin" | "engineering")
        || (supplied.command != "describe") != supplied.source.is_some()
        || (supplied.command == "evaluate" && supplied.now.is_none())
        || (supplied.command == "plan") != supplied.claim.is_some()
        || (supplied.command != "evaluate"
            && (supplied.now.is_some() || supplied.context.is_some() || supplied.records.is_some()))
    {
        return Err(USAGE.into());
    }
    Ok(Some(supplied))
}

fn execute(options: Options) -> Result<Output, String> {
    let workspace = input::Workspace::new(&options.workspace)?;
    let limits = workspace.limits(options.limits.as_deref())?;
    let engineering = (options.methods == "engineering").then(methods::engineering_registry);
    let registry = engineering
        .as_ref()
        .unwrap_or_else(|| methods::default_registry());
    if options.command == "describe" {
        return Ok(Output::Json(
            json!({"language":"EAL/3","methods":registry.describe(),"typed_bindings":propositions::describe_bindings(registry),"limits":limits,"scope":"pure semantic core; supplied observations; no collection or persistence"}),
        ));
    }
    let source_name = options.source.as_deref().expect("source checked above");
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
    let Some(options) = options()? else {
        return Ok(true);
    };
    let result = execute(options)?;
    let mut output = io::stdout().lock();
    match result {
        Output::Json(result) => {
            serde_json::to_writer_pretty(&mut output, &result)
                .map_err(|error| error.to_string())?;
            writeln!(output).map_err(|error| error.to_string())?;
            Ok(result.get("valid").and_then(Value::as_bool) != Some(false))
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
