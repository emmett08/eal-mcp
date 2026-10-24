//! Strict one-request text adapter to the official MCP client.
use crate::{
    client::{self, ClientTarget, StdioTarget},
    settings::{ServerSettings, parse_server_settings},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::IpAddr,
    path::PathBuf,
    process::ExitCode,
};
pub const MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;
pub fn parse_request(text: &str) -> Result<(String, Value), String> {
    if text.len() > MAX_REQUEST_BYTES {
        return Err("Host request exceeds 2 MiB".into());
    }
    let limits = eal_core::ExecutionLimits {
        record_bytes: MAX_REQUEST_BYTES,
        json_depth: 128,
        ..Default::default()
    };
    let value = eal_core::json::parse(text, &limits)?;
    let mut arguments = value
        .as_object()
        .ok_or("Request must be one JSON object")?
        .clone();
    let operation = arguments
        .remove("operation")
        .and_then(|v| v.as_str().map(str::to_owned));
    let operation = operation
        .filter(|name| !name.starts_with("eal_"))
        .and_then(|name| crate::operations::Operation::parse(&name))
        .ok_or("Unknown EAL operation [EAL_OPERATION_UNKNOWN]")?;
    let arguments = Value::Object(arguments);
    crate::operations::OperationRequest::decode(operation.name(), &arguments)?;
    Ok((operation.tool_name(), arguments))
}
pub async fn dispatch_request(
    text: &str,
    target: ClientTarget,
    timeout_seconds: f64,
) -> Result<Value, String> {
    let (tool, arguments) = parse_request(text)?;
    client::dispatch_tool(&tool, &arguments, target, timeout_seconds).await
}
pub fn server_arguments(settings: &ServerSettings) -> Vec<String> {
    let mut args = vec![
        "--workspace".into(),
        settings.workspace.to_string_lossy().into_owned(),
        "--transport".into(),
        "stdio".into(),
        "--exposure".into(),
        settings.exposure.clone(),
        "--max-in-flight".into(),
        settings.max_in_flight.to_string(),
        "--methods".into(),
        settings.methods.clone(),
    ];
    for (name, path) in [
        ("registry", &settings.registry),
        ("database", &settings.database),
        ("limits", &settings.limits),
    ] {
        if let Some(path) = path {
            args.extend([format!("--{name}"), path.to_string_lossy().into_owned()]);
        }
    }
    for id in &settings.known_entries {
        args.extend(["--known-entry".into(), id.clone()]);
    }
    args
}
pub fn configured_http_url(settings: &ServerSettings) -> Result<String, String> {
    if settings
        .host
        .parse::<IpAddr>()
        .is_ok_and(|ip| ip.is_unspecified())
    {
        return Err("An HTTP configuration with a wildcard bind address requires --url".into());
    }
    let host = if settings.host.contains(':') {
        format!("[{}]", settings.host)
    } else {
        settings.host.clone()
    };
    Ok(format!("http://{host}:{}{}", settings.port, settings.path))
}
pub fn stdio_target(
    settings: &ServerSettings,
    environ: &BTreeMap<String, String>,
    executable: PathBuf,
) -> ClientTarget {
    ClientTarget::Stdio(StdioTarget {
        command: executable,
        arguments: server_arguments(settings),
        environment: environ
            .iter()
            .filter(|(key, _)| !key.starts_with("EAL_MCP_"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        cwd: None,
    })
}
async fn run(
    argv: Vec<String>,
    environ: BTreeMap<String, String>,
    token: &mut Option<String>,
) -> Result<Value, String> {
    let mut service_args = vec![];
    let mut url = None;
    let mut timeout = 120.0;
    let mut index = 0;
    while index < argv.len() {
        let (flag, inline) = argv[index]
            .split_once('=')
            .map(|(a, b)| (a, Some(b)))
            .unwrap_or((&argv[index], None));
        if ["--url", "--timeout"].contains(&flag) {
            let value = if let Some(value) = inline {
                value.to_owned()
            } else {
                index += 1;
                argv.get(index)
                    .ok_or_else(|| format!("Missing value for {flag}"))?
                    .clone()
            };
            if flag == "--url" {
                url = Some(value);
            } else {
                timeout = value
                    .parse::<f64>()
                    .map_err(|_| "Host timeout must be positive and finite")?;
            }
        } else {
            service_args.push(argv[index].clone());
        }
        index += 1;
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let settings = parse_server_settings(&service_args, &environ, &cwd)?;
    let mut raw = Vec::new();
    std::io::stdin()
        .take((MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() > MAX_REQUEST_BYTES {
        return Err("Host input exceeds its configured byte limit".into());
    }
    let text = String::from_utf8(raw).map_err(|_| "Host input must be strict UTF-8")?;
    let target = if url.is_some() || settings.transport == "http" {
        *token = crate::auth::read_bearer_token(settings.token_env.as_deref(), &environ)?;
        client::create_http_target(
            &url.map(Ok)
                .unwrap_or_else(|| configured_http_url(&settings))?,
            settings.token_env.as_deref(),
            &environ,
        )?
    } else {
        let executable = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .with_file_name(if cfg!(windows) {
                "eal-mcp.exe"
            } else {
                "eal-mcp"
            });
        stdio_target(&settings, &environ, executable)
    };
    dispatch_request(&text, target, timeout).await
}
pub async fn main() -> ExitCode {
    let argv = std::env::args_os()
        .skip(1)
        .map(|argument| {
            argument
                .into_string()
                .map_err(|_| "Host arguments must be UTF-8".to_owned())
        })
        .collect::<Result<Vec<_>, _>>();
    let argv = match argv {
        Ok(argv) => argv,
        Err(error) => return print_error(error),
    };
    if argv.iter().any(|s| s == "--help" || s == "-h") {
        println!(
            "Send one JSON request through MCP.\nUsage: eal-host [--url HTTP_URL] [--timeout SECONDS] [eal-mcp settings]\nSettings: --config --workspace --registry --database --methods --limits --known-entry --token-env"
        );
        return ExitCode::SUCCESS;
    }
    let environ = match crate::application::environment() {
        Ok(environ) => environ,
        Err(error) => return print_error(error),
    };
    let mut token = None;
    let (response, code) = match run(argv, environ, &mut token).await {
        Ok(value) => {
            let code = if value["is_error"] == true { 1 } else { 0 };
            (value, code)
        }
        Err(error) => (json!({"is_error":true,"error":error}), 2),
    };
    let response = match crate::redaction::sanitise_response(&response, token.as_deref()) {
        Ok(value) => value,
        Err(error) => {
            let fallback = crate::redaction::sanitise_response(
                &json!({"is_error":true,"error":error}),
                token.as_deref(),
            )
            .unwrap_or_else(|_| json!({"is_error":true,"error":""}));
            let _ = writeln!(
                std::io::stdout(),
                "{}",
                serde_json::to_string(&fallback).unwrap()
            );
            return ExitCode::from(2);
        }
    };
    if writeln!(
        std::io::stdout(),
        "{}",
        serde_json::to_string(&response).unwrap()
    )
    .is_err()
    {
        return ExitCode::from(2);
    }
    ExitCode::from(code)
}

fn print_error(error: String) -> ExitCode {
    let response = json!({"is_error":true,"error":error});
    let _ = writeln!(std::io::stdout(), "{response}");
    ExitCode::from(2)
}
