//! Bundled native method worker and resource-limited launcher for trusted commands.
use crate::methods::protocol;
use eal_core::methods::MethodError;
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::process::ExitCode;

pub fn main() -> ExitCode {
    match run_stdio() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Method worker failed: {error}");
            ExitCode::FAILURE
        }
    }
}
pub fn run_stdio() -> Result<(), MethodError> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--timeout-seconds") {
        return Err("Method worker requires host resource limits".into());
    }
    let timeout: f64 = args
        .next()
        .ok_or("Missing method CPU limit")?
        .parse()
        .map_err(|_| "Invalid method CPU limit")?;
    if args.next().as_deref() != Some("--max-memory-bytes") {
        return Err("Missing method memory limit".into());
    }
    let memory: u64 = args
        .next()
        .ok_or("Missing method memory limit")?
        .parse()
        .map_err(|_| "Invalid method memory limit")?;
    let rest: Vec<String> = args.collect();
    let external = if rest.is_empty() {
        None
    } else if rest.first().map(String::as_str) == Some("--exec") && rest.len() > 1 {
        Some(&rest[1..])
    } else {
        return Err("Invalid method worker arguments".into());
    };
    apply_resources(timeout, memory, external.is_some())?;
    #[cfg(unix)]
    if let Some(argv) = external {
        use std::os::unix::process::CommandExt;
        let error = std::process::Command::new(&argv[0]).args(&argv[1..]).exec();
        return Err(format!("Cannot execute registered method command: {error}").into());
    }
    #[cfg(not(unix))]
    if external.is_some() {
        return Err("Supervised methods require POSIX resource limits".into());
    }
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(protocol::READY)
        .map_err(|e| e.to_string())?;
    stdout.flush().map_err(|e| e.to_string())?;
    let mut input = Vec::new();
    std::io::stdin()
        .take((protocol::MAX_REQUEST_BYTES + 2) as u64)
        .read_to_end(&mut input)
        .map_err(|e| e.to_string())?;
    let reply = run_request(&input);
    let response = match reply {
        Ok((output, maximum)) => (json!({"ok":true,"output":output}), maximum),
        Err((error, maximum)) => {
            let incomplete = matches!(error, MethodError::Incomplete(_));
            let mut message = error.to_string();
            while message.len() > 512 {
                message.pop();
            }
            (
                json!({"ok":false,"incomplete":incomplete,"error":message}),
                maximum,
            )
        }
    };
    let bytes = protocol::encode(&response.0, response.1)?;
    stdout.write_all(&bytes).map_err(|e| e.to_string())?;
    stdout.flush().map_err(|e| e.to_string())?;
    Ok(())
}
fn run_request(input: &[u8]) -> Result<(Value, usize), (MethodError, usize)> {
    let mut maximum = 1024 * 1024;
    let mut computation = || -> Result<(Value, usize), MethodError> {
        let frame = input
            .strip_suffix(b"\n")
            .ok_or("Method request requires a final newline")?;
        if frame.contains(&b'\n') {
            return Err("Method request contains more than one frame".into());
        }
        let value = protocol::strict_value(frame, protocol::MAX_REQUEST_BYTES)?;
        let request = protocol::Request::decode(value)?;
        if request.protocol != protocol::VERSION {
            return Err("Unsupported method worker protocol version".into());
        }
        let registry = eal_aspic::registry::with_aspic(&eal_core::methods::engineering_registry())?;
        let contract = registry
            .get(&request.method)
            .ok_or("Method is not bundled in the native worker")?;
        maximum = contract.max_output_bytes;
        if !eal_core::canonical::same_json(
            &request.contract,
            &contract.describe(),
            &protocol::wire_limits(protocol::MAX_REQUEST_BYTES),
        ) {
            return Err("Worker method contract differs from the registered implementation".into());
        }
        let result = eal_core::methods::assess_mode_with_limits(
            &request.method,
            &[json!({"id":"worker-input","kind":contract.evidence_kind,"value":request.payload})],
            &[],
            &registry,
            &request.limits,
        );
        if result["status"] != "supported" {
            let error = result["reasons"]
                .as_array()
                .and_then(|r| r.first())
                .and_then(Value::as_str)
                .unwrap_or("Native method failed")
                .to_owned();
            return Err(if result["status"] == "incomplete" {
                MethodError::Incomplete(error)
            } else {
                MethodError::Invalid(error)
            });
        }
        let output = result["details"].clone();
        // The public protocol result is the same complete success envelope the contract caps.
        if eal_core::canonical::canonical_json_bytes(
            &json!({"ok":true,"output":&output}),
            &protocol::wire_limits(contract.max_output_bytes),
        )?
        .len()
            > contract.max_output_bytes
        {
            return Err("Method output exceeds byte limit".into());
        }
        Ok((output, contract.max_output_bytes))
    };
    computation().map_err(|error| (error, maximum))
}

#[cfg(unix)]
fn apply_resources(timeout: f64, memory: u64, external: bool) -> Result<(), MethodError> {
    use rustix::process::{Resource, Rlimit, setrlimit};
    if !timeout.is_finite()
        || timeout <= 0.0
        || timeout > 60.0
        || !(64 * 1024 * 1024..=4 * 1024u64.pow(3)).contains(&memory)
    {
        return Err("Invalid method resource limits".into());
    }
    let used = rustix::time::clock_gettime(rustix::time::ClockId::ProcessCPUTime);
    // CPU is cumulative from process birth. Native setup is charged at its actual
    // process CPU time; an exec launcher reserves the separate five-second startup window.
    let cpu = (used.tv_sec as f64
        + used.tv_nsec as f64 / 1e9
        + timeout
        + if external { 5.0 } else { 0.0 })
    .ceil() as u64;
    for (resource, value) in [
        (Resource::Cpu, cpu),
        (Resource::As, memory),
        (Resource::Core, 0),
    ] {
        setrlimit(
            resource,
            Rlimit {
                current: Some(value),
                maximum: Some(value),
            },
        )
        .map_err(|e| MethodError::Invalid(format!("Cannot apply method resource limit: {e}")))?;
    }
    Ok(())
}
#[cfg(not(unix))]
fn apply_resources(_: f64, _: u64, _: bool) -> Result<(), MethodError> {
    Err("Supervised methods require POSIX resource limits".into())
}
