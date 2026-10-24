use super::{
    AcquisitionError, AcquisitionFailureCategory, AcquisitionResult, MAX_REQUEST_BYTES,
    ToolBinding, ToolKind, files, registry,
};
use crate::{
    lease::AcquisitionLease,
    process::{CommandRequest, ExecutionError, ProcessError, Supervisor},
};
use eal_core::{ExecutionLimits, canonical};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};

pub fn acquire(
    binding: &ToolBinding,
    request: &Value,
    workspace: &Path,
    secret: &[u8],
    supervisor: &Supervisor,
    lease: Option<&AcquisitionLease>,
) -> Result<AcquisitionResult, AcquisitionError> {
    acquire_with_limits(
        binding,
        request,
        workspace,
        secret,
        supervisor,
        lease,
        &ExecutionLimits::default(),
    )
}
#[allow(clippy::too_many_arguments)]
pub fn acquire_with_limits(
    binding: &ToolBinding,
    request: &Value,
    workspace: &Path,
    secret: &[u8],
    supervisor: &Supervisor,
    lease: Option<&AcquisitionLease>,
    limits: &ExecutionLimits,
) -> Result<AcquisitionResult, AcquisitionError> {
    limits.validate()?;
    registry::validate_binding(binding)?;
    match binding.kind {
        ToolKind::Command => command(
            binding, request, workspace, secret, supervisor, lease, limits,
        ),
        ToolKind::JsonFile => file(binding, workspace),
    }
}

fn command(
    binding: &ToolBinding,
    request: &Value,
    workspace: &Path,
    secret: &[u8],
    supervisor: &Supervisor,
    lease: Option<&AcquisitionLease>,
    limits: &ExecutionLimits,
) -> Result<AcquisitionResult, AcquisitionError> {
    let input = encode_request(request, limits)?;
    let environment = binding.effective_environment()?;
    let digest =
        binding.process_environment_digest_with_limits(secret, Some(&environment), limits)?;
    let outcome = supervisor
        .run(
            CommandRequest {
                argv: binding.argv.clone(),
                environment,
                cwd: workspace.to_owned(),
                input,
                timeout: Duration::from_secs_f64(binding.timeout_seconds),
                max_output_bytes: binding.max_output_bytes,
            },
            lease.map(|lease| lease as &dyn eal_process::Retention),
        )
        .map_err(|error| match error {
            ProcessError::Io(error) => AcquisitionError::from(error),
            ProcessError::Invalid(message) => AcquisitionError::invalid(message),
            error => AcquisitionError {
                category: AcquisitionFailureCategory::RuntimeError,
                message: error.to_string(),
            },
        })?;
    let execution = match outcome.execution_error.as_ref() {
        Some(ExecutionError::Spawn(_)) => None,
        Some(error) => Some(error.label()),
        None if outcome.timed_out => Some("timeout"),
        None => None,
    };
    let mut metadata = json!({"process_environment_digest":digest,"returncode":outcome.returncode,"output_truncated":execution==Some("output_limit")});
    let error = if let Some(error) = execution {
        metadata["execution_error"] = json!(error);
        Some(AcquisitionError::invalid(error))
    } else if outcome.returncode != 0 {
        Some(AcquisitionError::invalid(format!(
            "Tool exited with status {}",
            outcome.returncode
        )))
    } else {
        None
    };
    Ok(AcquisitionResult {
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        metadata,
        error,
    })
}

fn file(binding: &ToolBinding, workspace: &Path) -> Result<AcquisitionResult, AcquisitionError> {
    let path = files::bounded_path(
        workspace,
        binding.path.as_deref().expect("validated file path"),
    )?;
    let workspace = workspace.canonicalize()?;
    let relative = path
        .strip_prefix(&workspace)
        .map_err(|_| AcquisitionError::invalid("File path escaped the workspace"))?;
    let relative = relative
        .to_str()
        .ok_or("File path must contain valid Unicode scalar values")?;
    let mut metadata = json!({"file":relative});
    let mut stdout = match files::read_regular(
        &path,
        binding.max_output_bytes,
        "File observations require a regular file",
    ) {
        Ok(stdout) => stdout,
        Err(error) => {
            return Ok(AcquisitionResult {
                metadata,
                error: Some(error),
                ..AcquisitionResult::default()
            });
        }
    };
    let error = if stdout.len() > binding.max_output_bytes {
        stdout.truncate(binding.max_output_bytes);
        metadata["output_truncated"] = json!(true);
        Some(AcquisitionError::invalid("output_limit"))
    } else {
        None
    };
    Ok(AcquisitionResult {
        stdout,
        metadata,
        error,
        ..AcquisitionResult::default()
    })
}

/// Python's sorted, ASCII-escaped JSON request plus one newline, bounded while writing.
pub fn encode_request(
    value: &Value,
    limits: &ExecutionLimits,
) -> Result<Vec<u8>, AcquisitionError> {
    limits.validate()?;
    let mut writer = RequestWriter {
        bytes: Vec::new(),
        remaining: limits.json_nodes,
        limits,
    };
    writer.value(value, 0)?;
    writer.append(b"\n")?;
    Ok(writer.bytes)
}
struct RequestWriter<'a> {
    bytes: Vec<u8>,
    remaining: usize,
    limits: &'a ExecutionLimits,
}
impl RequestWriter<'_> {
    fn append(&mut self, bytes: &[u8]) -> Result<(), AcquisitionError> {
        if self.bytes.len().saturating_add(bytes.len()) > MAX_REQUEST_BYTES {
            return Err(format!("Tool request exceeds {MAX_REQUEST_BYTES} bytes").into());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn string(&mut self, text: &str) -> Result<(), AcquisitionError> {
        self.append(b"\"")?;
        for c in text.chars() {
            match c {
                '"' => self.append(b"\\\"")?,
                '\\' => self.append(b"\\\\")?,
                '\u{08}' => self.append(b"\\b")?,
                '\u{0c}' => self.append(b"\\f")?,
                '\n' => self.append(b"\\n")?,
                '\r' => self.append(b"\\r")?,
                '\t' => self.append(b"\\t")?,
                c if (' '..='~').contains(&c) => self.append(&[c as u8])?,
                c if (c as u32) <= 0xffff => {
                    self.append(format!("\\u{:04x}", c as u32).as_bytes())?
                }
                c => {
                    let v = c as u32 - 0x10000;
                    self.append(
                        format!(
                            "\\u{:04x}\\u{:04x}",
                            0xd800 + (v >> 10),
                            0xdc00 + (v & 0x3ff)
                        )
                        .as_bytes(),
                    )?
                }
            }
        }
        self.append(b"\"")
    }
    fn value(&mut self, value: &Value, depth: usize) -> Result<(), AcquisitionError> {
        if self.remaining == 0 || depth > self.limits.json_depth.min(128) {
            return Err("JSON resource limit exceeded".into());
        }
        self.remaining -= 1;
        match value {
            Value::Null => self.append(b"null"),
            Value::Bool(true) => self.append(b"true"),
            Value::Bool(false) => self.append(b"false"),
            Value::String(value) => self.string(value),
            Value::Number(value) => {
                if value.as_str().len() > MAX_REQUEST_BYTES - self.bytes.len() {
                    return Err(format!("Tool request exceeds {MAX_REQUEST_BYTES} bytes").into());
                }
                self.append(&canonical::canonical_json_bytes(
                    &Value::Number(value.clone()),
                    self.limits,
                )?)
            }
            Value::Array(values) => {
                if values.len() > self.remaining {
                    return Err("JSON resource limit exceeded".into());
                }
                self.append(b"[")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        self.append(b", ")?
                    }
                    self.value(value, depth + 1)?
                }
                self.append(b"]")
            }
            Value::Object(values) => {
                if values.len() > self.remaining {
                    return Err("JSON resource limit exceeded".into());
                }
                self.append(b"{")?;
                let mut keys = values.keys().collect::<Vec<_>>();
                keys.sort();
                for (i, key) in keys.into_iter().enumerate() {
                    if i > 0 {
                        self.append(b", ")?
                    }
                    self.string(key)?;
                    self.append(b": ")?;
                    self.value(&values[key], depth + 1)?
                }
                self.append(b"}")
            }
        }
    }
}
