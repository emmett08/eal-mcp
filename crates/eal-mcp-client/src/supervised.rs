//! Supervisor admission and lease ownership for official-SDK duplex transports.
#[cfg(unix)]
use crate::client::StdioTarget;
use eal_process::{Retention, Supervisor};
use std::{path::PathBuf, time::Duration};
#[cfg(unix)]
use std::{
    process::Stdio,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct StdioSupervision {
    supervisor: Supervisor,
    #[cfg(unix)]
    retention: Option<Arc<TransferredRetention>>,
    max_input_bytes: usize,
    max_output_bytes: usize,
    max_session_duration: Duration,
}
impl std::fmt::Debug for StdioSupervision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdioSupervision")
            .field("supervisor", &self.supervisor)
            .field("max_input_bytes", &self.max_input_bytes)
            .field("max_output_bytes", &self.max_output_bytes)
            .field("max_session_duration", &self.max_session_duration)
            .finish_non_exhaustive()
    }
}
#[cfg(unix)]
struct TransferredRetention(Mutex<Option<Stdio>>);
#[cfg(unix)]
impl Retention for TransferredRetention {
    fn duplicate_for_supervisor(&self) -> std::io::Result<Stdio> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("Poisoned MCP retention"))?
            .take()
            .ok_or_else(|| std::io::Error::other("MCP lease has already been transferred"))
    }
}
impl StdioSupervision {
    pub fn new(
        supervisor: &Supervisor,
        retention: Option<&dyn Retention>,
        max_input_bytes: usize,
        max_output_bytes: usize,
        max_session_duration: Duration,
    ) -> Result<Self, String> {
        if max_input_bytes == 0 || max_output_bytes == 0 || max_session_duration.is_zero() {
            return Err("MCP supervision requires positive time and transcript byte bounds".into());
        }
        #[cfg(not(unix))]
        let _ = retention;
        Ok(Self {
            supervisor: supervisor.clone(),
            #[cfg(unix)]
            retention: retention
                .map(|retained| {
                    retained
                        .duplicate_for_supervisor()
                        .map(|owned| Arc::new(TransferredRetention(Mutex::new(Some(owned)))))
                })
                .transpose()
                .map_err(|e| e.to_string())?,
            max_input_bytes,
            max_output_bytes,
            max_session_duration,
        })
    }
    pub(crate) fn installed() -> Result<Self, String> {
        let executable = companion()?;
        // Persistent host sessions have a finite lifetime independently of
        // their per-operation deadlines. Callers can supply a tighter bound.
        Self::new(
            &Supervisor::new(executable),
            None,
            64 * 1024 * 1024,
            64 * 1024 * 1024,
            Duration::from_secs(3600),
        )
    }
    #[cfg(unix)]
    pub(crate) fn spawn(
        &self,
        target: StdioTarget,
    ) -> Result<eal_process::duplex::DuplexProcess, String> {
        let mut argv = vec![
            target
                .command
                .to_str()
                .ok_or("MCP command path must be UTF-8")?
                .to_owned(),
        ];
        argv.extend(target.arguments);
        self.supervisor
            .duplex(
                eal_process::duplex::DuplexRequest {
                    argv,
                    environment: target.environment,
                    cwd: target
                        .cwd
                        .unwrap_or(std::env::current_dir().map_err(|e| e.to_string())?),
                    timeout: self.max_session_duration,
                    max_input_bytes: self.max_input_bytes,
                    max_output_bytes: self.max_output_bytes,
                    max_stderr_bytes: 65536,
                },
                self.retention.as_deref().map(|r| r as &dyn Retention),
            )
            .map_err(|e| e.to_string())
    }
}

fn companion() -> Result<PathBuf, String> {
    if let Some(value) = std::env::var_os("EAL_COMMAND_SUPERVISOR") {
        let path = PathBuf::from(value);
        if path.is_absolute() && path.is_file() {
            return Ok(path);
        }
        return Err("EAL_COMMAND_SUPERVISOR must identify an absolute installed companion".into());
    }
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let directory = executable
        .parent()
        .ok_or("Cannot locate MCP client executable")?;
    let mut directories = vec![directory.to_owned()];
    if directory.file_name().is_some_and(|n| n == "deps")
        && let Some(profile) = directory.parent()
    {
        directories.push(profile.to_owned());
    }
    if let Some(path) = std::env::var_os("PATH") {
        directories.extend(std::env::split_paths(&path).filter(|p| p.is_absolute()));
    }
    directories.into_iter().map(|p| eal_process::companion_in(&p)).find(|p| p.is_file())
        .ok_or_else(|| "Install eal-process-supervisor beside the MCP client or configure EAL_COMMAND_SUPERVISOR".into())
}
