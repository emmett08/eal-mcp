//! One supervised independent command. Reservations and retained response bytes belong to the caller.
use eal_core::{ExecutionLimits, canonical};
use eal_runtime::{
    lease::{AcquisitionLease, WorkspaceAcquisition},
    process::{CancellationToken, CommandRequest, ExecutionError, ProcessError, Supervisor},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

pub type StopCheck<'a> = dyn Fn() -> Result<bool, String> + Sync + 'a;
#[derive(Clone)]
pub struct CommandWorkerConfig {
    pub argv: Vec<String>,
    pub execution_mode: String,
    pub timeout_seconds: u64,
    pub max_output_bytes: usize,
    pub environment: BTreeMap<String, String>,
    pub working_directory: PathBuf,
    pub lease_directory: PathBuf,
}
impl std::fmt::Debug for CommandWorkerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandWorkerConfig")
            .field("execution_mode", &self.execution_mode)
            .field("timeout_seconds", &self.timeout_seconds)
            .field("max_output_bytes", &self.max_output_bytes)
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Debug)]
pub struct CommandWorker {
    supervisor: Supervisor,
    config: CommandWorkerConfig,
    lease: WorkspaceAcquisition,
}
#[derive(Clone, Debug)]
pub struct WorkerResult {
    pub execution_id: String,
    pub execution_mode: String,
    pub raw_response: Vec<u8>,
    pub evidence: Value,
    pub response: Value,
}
#[derive(Clone, Debug)]
pub struct WorkerInterrupted {
    pub reason: String,
    pub execution_id: String,
    pub execution_mode: String,
    pub raw_response: Vec<u8>,
    pub evidence: Value,
}
impl std::fmt::Display for WorkerInterrupted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Worker command failed: {}", self.reason)
    }
}
impl std::error::Error for WorkerInterrupted {}
#[derive(Debug)]
pub enum WorkerError {
    Invalid(String),
    Interrupted(Box<WorkerInterrupted>),
}
impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => f.write_str(message),
            Self::Interrupted(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for WorkerError {}
impl From<String> for WorkerError {
    fn from(value: String) -> Self {
        Self::Invalid(value)
    }
}
impl CommandWorker {
    pub fn new(supervisor: Supervisor, mut config: CommandWorkerConfig) -> Result<Self, String> {
        if config.argv.is_empty() || config.argv.iter().any(|a| a.is_empty() || a.contains('\0')) {
            return Err("argv must contain nonempty strings without NUL".into());
        }
        if !["software_test", "provider", "actual_host_agent"]
            .contains(&config.execution_mode.as_str())
        {
            return Err("Unknown worker execution mode".into());
        }
        if !(1..=3600).contains(&config.timeout_seconds) {
            return Err("timeout_seconds must be an integer from 1 through 3600".into());
        }
        if !(1..=16 * 1024 * 1024).contains(&config.max_output_bytes) {
            return Err("max_output_bytes must be an integer from 1 through 16777216".into());
        }
        if config
            .environment
            .iter()
            .any(|(k, v)| k.is_empty() || k.contains(['=', '\0']) || v.contains('\0'))
        {
            return Err("environment must map valid variable names to string values".into());
        }
        config.working_directory = config
            .working_directory
            .canonicalize()
            .map_err(|_| "Worker workspace must be an existing directory")?;
        config.lease_directory = config
            .lease_directory
            .canonicalize()
            .map_err(|_| "Worker lease directory must be an existing coordination-run directory")?;
        if !config.working_directory.is_dir() || config.working_directory.to_str().is_none() {
            return Err("Worker workspace must be an existing UTF-8 directory".into());
        }
        if !config.lease_directory.is_dir()
            || config.lease_directory.to_str().is_none()
            || config.working_directory == config.lease_directory
        {
            return Err(
                "Worker lease directory must be distinct from its working directory".into(),
            );
        }
        if !supervisor.executable().is_absolute() {
            return Err("Worker supervisor must be an absolute executable path".into());
        }
        let lease = WorkspaceAcquisition::new(&config.lease_directory)?;
        Ok(Self {
            supervisor,
            config,
            lease,
        })
    }
    pub fn execution_mode(&self) -> &str {
        &self.config.execution_mode
    }
    pub fn max_output_bytes(&self) -> usize {
        self.config.max_output_bytes
    }
    pub fn execute(
        &self,
        context: &Value,
        call_id: &str,
        remaining: Duration,
        should_stop: Option<&StopCheck<'_>>,
    ) -> Result<WorkerResult, WorkerError> {
        let started = Instant::now();
        let timeout = remaining.min(Duration::from_secs(self.config.timeout_seconds));
        self.validate_call(context, call_id, timeout)?;
        if let Some(check) = should_stop {
            match check() {
                Ok(true) => return Err(self.empty_failure(call_id, "cancelled", started)),
                Err(_) => return Err(self.empty_failure(call_id, "supervisor_failure", started)),
                Ok(false) => {}
            }
        }
        let lease = self.lease.hold(timeout).map_err(WorkerError::Invalid)?;
        let timeout = timeout.saturating_sub(started.elapsed());
        if timeout.is_zero() {
            return Err(self.empty_failure(call_id, "timeout", started));
        }
        self.execute_with_lease(context, call_id, timeout, should_stop, &lease)
    }
    fn execute_with_lease(
        &self,
        context: &Value,
        call_id: &str,
        remaining: Duration,
        should_stop: Option<&StopCheck<'_>>,
        lease: &AcquisitionLease,
    ) -> Result<WorkerResult, WorkerError> {
        let timeout = remaining.min(Duration::from_secs(self.config.timeout_seconds));
        self.validate_call(context, call_id, timeout)?;
        let input = context_bytes(context)?;
        let started = Instant::now();
        let cancellation = CancellationToken::default();
        let done = AtomicBool::new(false);
        let callback_failed = AtomicBool::new(false);
        let mut evidence = json!({"call_id":call_id,"supervisor_pid":null,"returncode":null,"model":null,"usage":null,"receipt_verification":null});
        let mut execution_id = format!("command:{call_id}");
        let preflight = should_stop.map(|check| check()).transpose();
        let (outcome, mut failure) = match preflight {
            Err(_) => (None, Some("supervisor_failure")),
            Ok(Some(true)) => (None, Some("cancelled")),
            _ => {
                let result = std::thread::scope(|scope| {
                    if let Some(check) = should_stop {
                        let cancellation = &cancellation;
                        let done = &done;
                        let callback_failed = &callback_failed;
                        scope.spawn(move || {
                            while !done.load(Ordering::Acquire) {
                                match check() {
                                    Ok(true) => {
                                        cancellation.cancel();
                                        break;
                                    }
                                    Err(_) => {
                                        callback_failed.store(true, Ordering::Release);
                                        cancellation.cancel();
                                        break;
                                    }
                                    Ok(false) => {}
                                }
                                std::thread::park_timeout(Duration::from_millis(20));
                            }
                        });
                    }
                    let result = self.supervisor.run_with_cancellation(
                        CommandRequest {
                            argv: self.config.argv.clone(),
                            environment: self.config.environment.clone(),
                            cwd: self.config.working_directory.clone(),
                            input,
                            timeout,
                            max_output_bytes: self.config.max_output_bytes,
                        },
                        Some(lease),
                        &cancellation,
                    );
                    done.store(true, Ordering::Release);
                    result
                });
                match result {
                    Ok(result) => {
                        let failure = if callback_failed.load(Ordering::Acquire) {
                            Some("supervisor_failure")
                        } else {
                            match &result.execution_error {
                                Some(ExecutionError::Timeout) => Some("timeout"),
                                Some(ExecutionError::OutputLimit) => Some("output_limit"),
                                Some(ExecutionError::Cancelled) => Some("cancelled"),
                                Some(ExecutionError::Spawn(_)) => Some("nonzero_exit"),
                                Some(_) => Some("supervisor_failure"),
                                None if result.returncode != 0 => Some("nonzero_exit"),
                                None => None,
                            }
                        };
                        (Some(result), failure)
                    }
                    Err(ProcessError::Io(_)) => (None, Some("start_failure")),
                    Err(_) => (None, Some("supervisor_failure")),
                }
            }
        };
        let (stdout, stderr) = if let Some(outcome) = outcome {
            evidence["supervisor_pid"] = json!(outcome.supervisor_pid);
            evidence["returncode"] = json!(outcome.returncode);
            if let Some(pid) = outcome.supervisor_pid {
                execution_id = format!("command:{call_id}:{pid}");
            }
            (outcome.stdout, outcome.stderr)
        } else {
            (vec![], vec![])
        };
        evidence["elapsed_ms"] = json!(started.elapsed().as_millis().min(u64::MAX as u128) as u64);
        evidence["stdout_bytes"] = json!(stdout.len());
        evidence["stderr_bytes"] = json!(stderr.len());
        evidence["output_truncated"] = json!(failure == Some("output_limit"));
        if self.config.execution_mode == "software_test" {
            let mut diagnostic = String::from_utf8_lossy(&stderr).into_owned();
            let mut secrets = self
                .config
                .environment
                .values()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            secrets.sort_by_key(|s| std::cmp::Reverse(s.len()));
            for secret in &secrets {
                let replacement = if "[redacted]".contains(secret.as_str()) {
                    ""
                } else {
                    "[redacted]"
                };
                diagnostic = diagnostic.replace(secret.as_str(), replacement);
            }
            // A later replacement must not reintroduce an earlier opaque credential.
            if secrets
                .iter()
                .any(|secret| diagnostic.contains(secret.as_str()))
            {
                diagnostic.clear();
            }
            diagnostic = diagnostic
                .chars()
                .take(4096.min(self.config.max_output_bytes))
                .collect();
            evidence["stderr"] = json!(diagnostic);
        }
        let mut response = None;
        if failure.is_none() {
            match response_object(&stdout) {
                Ok(parsed) if self.config.execution_mode == "software_test" => {
                    response = Some(parsed)
                }
                Ok(parsed) => {
                    if let Ok((actual, receipt)) = live_response(&parsed, call_id) {
                        execution_id = receipt["execution_id"].as_str().unwrap().to_owned();
                        evidence["model"] = receipt["model"].clone();
                        evidence["usage"] = receipt["usage"].clone();
                        evidence["external_execution_receipt"] = receipt;
                        evidence["receipt_verification"] = json!("operator_command_assertion");
                        response = Some(actual);
                    } else {
                        failure = Some("invalid_response");
                    }
                }
                Err(_) => failure = Some("invalid_response"),
            }
        }
        if let Some(reason) = failure {
            evidence["execution_error"] = json!(reason);
            return Err(WorkerError::Interrupted(Box::new(WorkerInterrupted {
                reason: reason.into(),
                execution_id,
                execution_mode: self.config.execution_mode.clone(),
                raw_response: stdout,
                evidence,
            })));
        }
        Ok(WorkerResult {
            execution_id,
            execution_mode: self.config.execution_mode.clone(),
            raw_response: stdout,
            evidence,
            response: response.expect("successful response is present"),
        })
    }
    fn empty_failure(&self, call_id: &str, reason: &str, started: Instant) -> WorkerError {
        let mut evidence = json!({"call_id":call_id,"supervisor_pid":null,"returncode":null,"model":null,"usage":null,"receipt_verification":null,"elapsed_ms":started.elapsed().as_millis().min(u64::MAX as u128) as u64,"stdout_bytes":0,"stderr_bytes":0,"output_truncated":false,"execution_error":reason});
        if self.config.execution_mode == "software_test" {
            evidence["stderr"] = json!("");
        }
        WorkerError::Interrupted(Box::new(WorkerInterrupted {
            reason: reason.into(),
            execution_id: format!("command:{call_id}"),
            execution_mode: self.config.execution_mode.clone(),
            raw_response: vec![],
            evidence,
        }))
    }
    fn validate_call(
        &self,
        context: &Value,
        call_id: &str,
        remaining: Duration,
    ) -> Result<(), WorkerError> {
        if !context.is_object() {
            return Err(WorkerError::Invalid(
                "Worker context must be an object".into(),
            ));
        }
        if call_id.is_empty()
            || call_id.len() > 256
            || !call_id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_.:/-".contains(&c))
        {
            return Err(WorkerError::Invalid(
                "call_id must be a bounded identifier".into(),
            ));
        }
        if remaining.is_zero() {
            return Err(WorkerError::Invalid(
                "remaining_seconds must be positive".into(),
            ));
        }
        context_bytes(context)?;
        Ok(())
    }
}
fn response_object(bytes: &[u8]) -> Result<Value, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "Worker response requires UTF-8")?;
    let limits = ExecutionLimits {
        record_bytes: 16 * 1024 * 1024,
        json_depth: 128,
        ..Default::default()
    };
    let value = eal_core::json::parse(text, &limits)?;
    if !value.is_object() {
        return Err("Worker response must be an object".into());
    }
    Ok(value)
}
pub(crate) fn live_response(parsed: &Value, call_id: &str) -> Result<(Value, Value), String> {
    let object = parsed
        .as_object()
        .ok_or("Live response requires a receipt envelope")?;
    if object.len() != 2
        || !parsed.get("response").is_some_and(Value::is_object)
        || !object.contains_key("execution")
    {
        return Err("Live response requires a receipt envelope".into());
    }
    let receipt = &parsed["execution"];
    let object = receipt
        .as_object()
        .ok_or("Live execution requires an external execution receipt")?;
    if object.len() != 3
        || !["execution_id", "model", "usage"]
            .iter()
            .all(|k| object.contains_key(*k))
    {
        return Err("Live execution requires an external execution receipt".into());
    }
    let id = receipt["execution_id"]
        .as_str()
        .filter(|id| {
            !id.is_empty()
                && id.chars().count() <= 512
                && !id.chars().any(|c| c < ' ')
                && *id != call_id
                && !id.starts_with("command:")
        })
        .ok_or("Live execution requires an external execution identity")?;
    let _ = id;
    if !receipt["model"].is_null()
        && !receipt["model"].as_str().is_some_and(|s| {
            !s.is_empty() && s.chars().count() <= 256 && !s.chars().any(|c| c < ' ')
        })
    {
        return Err("Invalid model identity".into());
    }
    if !receipt["usage"].is_null() {
        let usage = receipt["usage"]
            .as_object()
            .ok_or("Invalid usage receipt")?;
        for (key, value) in usage {
            if !["input_tokens", "output_tokens", "total_tokens"].contains(&key.as_str())
                || !value.as_number().is_some_and(|n| {
                    let raw = n.to_string();
                    !raw.starts_with('-') && !raw.contains(['.', 'e', 'E'])
                })
            {
                return Err("Invalid usage receipt".into());
            }
        }
    }
    Ok((parsed["response"].clone(), receipt.clone()))
}
fn context_bytes(value: &Value) -> Result<Vec<u8>, WorkerError> {
    let limits = ExecutionLimits {
        json_depth: 128,
        record_bytes: 4 * 1024 * 1024,
        ..Default::default()
    };
    canonical::canonical_json_bytes(value, &limits).map_err(WorkerError::Invalid)?;
    fn encode(value: &Value, out: &mut Vec<u8>) -> Result<(), WorkerError> {
        match value {
            Value::Array(values) => {
                out.push(b'[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        out.extend_from_slice(b", ");
                    }
                    encode(value, out)?;
                }
                out.push(b']');
            }
            Value::Object(values) => {
                out.push(b'{');
                let mut keys = values.keys().collect::<Vec<_>>();
                keys.sort();
                for (index, key) in keys.into_iter().enumerate() {
                    if index > 0 {
                        out.extend_from_slice(b", ");
                    }
                    out.extend_from_slice(serde_json::to_string(key).unwrap().as_bytes());
                    out.extend_from_slice(b": ");
                    encode(&values[key], out)?;
                }
                out.push(b'}');
            }
            _ => out.extend_from_slice(
                &canonical::canonical_json_bytes(value, &ExecutionLimits::default())
                    .map_err(WorkerError::Invalid)?,
            ),
        };
        if out.len() >= 4 * 1024 * 1024 {
            return Err(WorkerError::Invalid(
                "Worker context exceeds its byte limit".into(),
            ));
        }
        Ok(())
    }
    let mut bytes = vec![];
    encode(value, &mut bytes)?;
    bytes.push(b'\n');
    Ok(bytes)
}
