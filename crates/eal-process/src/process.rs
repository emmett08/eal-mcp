//! Bounded command execution through an independently owned process supervisor.
pub(crate) mod protocol;
#[cfg(unix)]
mod unix;

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A narrowly scoped capability retained by the supervisor until descendants stop.
/// Implementations duplicate ownership without explicitly releasing another copy.
pub trait Retention {
    fn duplicate_for_supervisor(&self) -> std::io::Result<std::process::Stdio>;
}

#[derive(Clone, Debug)]
pub struct CommandRequest {
    pub argv: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub cwd: PathBuf,
    pub input: Vec<u8>,
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

/// Independent stream budgets additionally constrained by the combined request budget.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamLimits {
    pub max_stdout_bytes: usize,
    pub max_stderr_bytes: usize,
}
impl StreamLimits {
    pub(crate) fn validate(self, combined: usize) -> Result<(), ProcessError> {
        if self.max_stdout_bytes > combined || self.max_stderr_bytes > combined {
            return Err(ProcessError::Invalid(
                "Stream limits must fit within the combined output byte budget".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartupHandshake {
    pub expected_first_stdout_line: Vec<u8>,
    pub startup_timeout: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionError {
    Timeout,
    OutputLimit,
    Cancelled,
    Spawn(String),
    Startup(String),
    Supervisor(String),
}

impl ExecutionError {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::OutputLimit => "output_limit",
            Self::Cancelled => "cancelled",
            Self::Spawn(_) => "spawn_error",
            Self::Startup(_) => "startup_error",
            Self::Supervisor(_) => "supervisor_error",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProcessOutcome {
    /// Parent-observed PID of the independently owned command supervisor.
    pub supervisor_pid: Option<u32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub returncode: i32,
    pub timed_out: bool,
    pub execution_error: Option<ExecutionError>,
    pub startup_completed: bool,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
}

impl ProcessOutcome {
    pub(crate) fn new(stdout: Vec<u8>, stderr: Vec<u8>, status: protocol::Status) -> Self {
        Self {
            supervisor_pid: None,
            stdout_bytes: stdout.len(),
            stderr_bytes: stderr.len(),
            stdout_sha256: format!("{:x}", Sha256::digest(&stdout)),
            stderr_sha256: format!("{:x}", Sha256::digest(&stderr)),
            stdout,
            stderr,
            returncode: status.returncode,
            timed_out: status.timed_out,
            execution_error: status.execution_error,
            startup_completed: status.startup_completed,
        }
    }
}

#[derive(Debug)]
pub enum ProcessError {
    Invalid(String),
    Io(std::io::Error),
    Protocol(String),
    Unsupported,
}

impl ProcessError {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Invalid(_) => "ValueError",
            Self::Io(_) => "OSError",
            _ => "RuntimeError",
        }
    }
}
impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) | Self::Protocol(message) => f.write_str(message),
            Self::Io(error) => error.fmt(f),
            Self::Unsupported => {
                f.write_str("The bounded command adapter currently requires a POSIX host")
            }
        }
    }
}
impl std::error::Error for ProcessError {}
impl From<std::io::Error> for ProcessError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Default, Debug)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Clone, Debug)]
pub struct Supervisor {
    executable: PathBuf,
}
impl Supervisor {
    pub fn new(executable: PathBuf) -> Self {
        Self { executable }
    }
    pub fn executable(&self) -> &std::path::Path {
        &self.executable
    }
    pub fn run(
        &self,
        request: CommandRequest,
        lease: Option<&dyn Retention>,
    ) -> Result<ProcessOutcome, ProcessError> {
        self.run_with_cancellation(request, lease, &CancellationToken::default())
    }
    pub fn run_with_cancellation(
        &self,
        request: CommandRequest,
        lease: Option<&dyn Retention>,
        cancellation: &CancellationToken,
    ) -> Result<ProcessOutcome, ProcessError> {
        self.execute(request, lease, cancellation, None, None)
    }
    pub fn run_framed(
        &self,
        request: CommandRequest,
        lease: Option<&dyn Retention>,
        handshake: StartupHandshake,
    ) -> Result<ProcessOutcome, ProcessError> {
        self.execute(
            request,
            lease,
            &CancellationToken::default(),
            Some(handshake),
            None,
        )
    }
    pub fn run_framed_with_cancellation(
        &self,
        request: CommandRequest,
        lease: Option<&dyn Retention>,
        handshake: StartupHandshake,
        cancellation: &CancellationToken,
    ) -> Result<ProcessOutcome, ProcessError> {
        self.execute(request, lease, cancellation, Some(handshake), None)
    }
    /// Enforce independent stream limits through the same supervisor boundary.
    pub fn run_with_stream_limits(
        &self,
        request: CommandRequest,
        lease: Option<&dyn Retention>,
        limits: StreamLimits,
    ) -> Result<ProcessOutcome, ProcessError> {
        self.run_with_stream_limits_and_cancellation(
            request,
            lease,
            &CancellationToken::default(),
            limits,
        )
    }
    pub fn run_with_stream_limits_and_cancellation(
        &self,
        request: CommandRequest,
        lease: Option<&dyn Retention>,
        cancellation: &CancellationToken,
        limits: StreamLimits,
    ) -> Result<ProcessOutcome, ProcessError> {
        limits.validate(request.max_output_bytes)?;
        self.execute(request, lease, cancellation, None, Some(limits))
    }
    fn execute(
        &self,
        request: CommandRequest,
        lease: Option<&dyn Retention>,
        cancellation: &CancellationToken,
        handshake: Option<StartupHandshake>,
        stream_limits: Option<StreamLimits>,
    ) -> Result<ProcessOutcome, ProcessError> {
        #[cfg(unix)]
        {
            unix::run(
                &self.executable,
                request,
                lease,
                cancellation,
                handshake,
                stream_limits,
            )
        }
        #[cfg(not(unix))]
        {
            let _ = (request, lease, cancellation, handshake, stream_limits);
            Err(ProcessError::Unsupported)
        }
    }
}
