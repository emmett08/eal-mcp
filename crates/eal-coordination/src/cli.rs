//! Operator-selected lifecycle commands over frozen durable coordination runs.
mod io;
mod options;

use crate::{
    contracts::{ServerSpec, TaskSpec},
    coordinator::Coordinator,
    host::HostPaths,
    workers::{CommandWorker, CommandWorkerConfig},
};
use eal_core::ExecutionLimits;
use eal_runtime::process::Supervisor;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path, process::ExitCode};

pub fn main() -> ExitCode {
    let arguments = match std::env::args_os()
        .skip(1)
        .map(|argument| {
            argument
                .into_string()
                .map_err(|_| "Arguments must be UTF-8")
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(arguments) => arguments,
        Err(error) => return failure("UsageError", error, 2),
    };
    if arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), "--help" | "-h"))
    {
        println!("{}", options::USAGE);
        return ExitCode::SUCCESS;
    }
    if arguments == ["--response-schema"] {
        println!("{}", crate::RESPONSE_SCHEMA.trim_end());
        return ExitCode::SUCCESS;
    }
    if let Some(result) = crate::debate_session::command(&arguments) {
        return result;
    }
    if let Some(result) = crate::ticket::command(&arguments) {
        return result;
    }
    let options = match options::parse(&arguments) {
        Ok(options) => options,
        Err(error) => return failure("UsageError", &error, 2),
    };
    match run(&options) {
        Ok(result) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&result).expect("finite JSON result")
            );
            if result.get("accepted") == Some(&Value::Bool(false))
                || matches!(
                    result["status"].as_str(),
                    Some("failed" | "cancelled" | "expired")
                )
            {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => failure("Error", &error, 1),
    }
}

fn failure(kind: &str, message: &str, status: u8) -> ExitCode {
    eprintln!("{}", json!({"error":{"type":kind,"message":message}}));
    ExitCode::from(status)
}

fn run(options: &options::Options) -> Result<Value, String> {
    let host = HostPaths::current()?;
    let coordinator = if options.operation == "start" {
        start(options, &host)?
    } else {
        Coordinator::open_with_host(&options.run, host.clone())?
    };
    let result = operation(&coordinator, options, &host);
    let closed = coordinator.close();
    result.and_then(|value| closed.map(|()| value))
}

fn start(options: &options::Options, host: &HostPaths) -> Result<Coordinator, String> {
    let mut spec = if let Some(path) = options.get("--spec") {
        let value = io::json(Path::new(path))?;
        let object = value
            .as_object()
            .ok_or("Run spec requires server/task and only declared optional fields")?;
        if !object.contains_key("server")
            || !object.contains_key("task")
            || object
                .keys()
                .any(|key| !["server", "task", "budgets", "transport"].contains(&key.as_str()))
        {
            return Err("Run spec requires server/task and only declared optional fields".into());
        }
        value
    } else {
        let manifest = io::json(Path::new(options.required("--demo")))?;
        let tasks = manifest["tasks"]
            .as_array()
            .ok_or("Demo manifest requires a task array")?;
        let selected = tasks
            .iter()
            .filter(|task| task["task_id"].as_str() == options.get("--task-id"))
            .collect::<Vec<_>>();
        if selected.len() != 1 {
            return Err("Choose one exact trusted demo task ID".into());
        }
        json!({"server":manifest.get("server").ok_or("Demo manifest lacks its server")?,"task":selected[0]})
    };
    if let Some(runtime) = options.get("--runtime") {
        spec["transport"] = json!({"kind":"lab","runtime":runtime});
    }
    if let Some(seconds) = options.get("--wall-seconds") {
        let seconds = seconds
            .parse::<i64>()
            .map_err(|_| "--wall-seconds requires an integer")?;
        if spec.get("budgets").is_none() {
            spec["budgets"] = json!({});
        }
        spec["budgets"]
            .as_object_mut()
            .ok_or("Unknown coordination budget")?
            .insert("wall_seconds".into(), json!(seconds));
    }
    let base = std::env::current_dir().map_err(|error| error.to_string())?;
    let server = ServerSpec::from_value(&spec["server"], &base)?;
    let task = TaskSpec::from_value(&spec["task"])?;
    Coordinator::start_with_host(
        &options.run,
        server,
        task,
        spec.get("budgets"),
        spec.get("transport"),
        host.clone(),
    )
}

fn operation(
    coordinator: &Coordinator,
    options: &options::Options,
    host: &HostPaths,
) -> Result<Value, String> {
    match options.operation.as_str() {
        "start" | "status" => coordinator.status(),
        "resume" => coordinator.resume(),
        "cancel" => coordinator.cancel(),
        "reserve" => {
            let result = coordinator.reserve_worker(
                options.required("--role"),
                options.required("--execution-mode"),
            )?;
            if let Some(path) = options.get("--context-out") {
                io::write_json(Path::new(path), &result["context"])?;
            }
            Ok(result)
        }
        "query" => coordinator.query(options.required("--call-id")),
        "lookup" => coordinator.lookup(
            options.required("--call-id"),
            options.required("--finding-id"),
        ),
        "record" => coordinator.record(
            options.required("--call-id"),
            &io::read(Path::new(options.required("--response")), 16 * 1024 * 1024)?,
            options.required("--execution-id"),
            options.required("--execution-mode"),
            None,
            None,
        ),
        "export" => coordinator.export(Path::new(options.required("--output"))),
        "execute" => {
            let argv = parse_json(options.required("--worker-argv").as_bytes())?
                .as_array()
                .ok_or("argv must be a nonempty sequence of strings")?
                .iter()
                .map(|argument| {
                    argument
                        .as_str()
                        .map(str::to_owned)
                        .ok_or("argv must contain nonempty strings without NUL".to_owned())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let environment = match options.get("--worker-environment") {
                None => BTreeMap::new(),
                Some(path) => io::json(Path::new(path))?
                    .as_object()
                    .ok_or("environment must map valid variable names to string values")?
                    .iter()
                    .map(|(key, value)| {
                        Ok((
                            key.clone(),
                            value
                                .as_str()
                                .ok_or(
                                    "environment must map valid variable names to string values",
                                )?
                                .to_owned(),
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, String>>()?,
            };
            let worker = CommandWorker::new(
                Supervisor::new(host.supervisor.clone()),
                CommandWorkerConfig {
                    argv,
                    execution_mode: options.required("--execution-mode").into(),
                    timeout_seconds: coordinator.budget("worker_seconds")? as u64,
                    max_output_bytes: coordinator.budget("max_worker_output_bytes")?,
                    environment,
                    working_directory: coordinator.server.workspace.clone(),
                    lease_directory: coordinator.directory.clone(),
                },
            )?;
            coordinator.run(&worker)
        }
        _ => unreachable!("closed CLI command was checked"),
    }
}

fn parse_json(bytes: &[u8]) -> Result<Value, String> {
    let limits = ExecutionLimits {
        record_bytes: 4 * 1024 * 1024,
        json_depth: 128,
        ..Default::default()
    };
    eal_core::json::parse(
        std::str::from_utf8(bytes).map_err(|error| error.to_string())?,
        &limits,
    )
}

fn strict_json(bytes: &[u8]) -> Result<Value, String> {
    let value = parse_json(bytes)?;
    if !value.is_object() {
        return Err("Worker response must be one JSON object".into());
    }
    Ok(value)
}
