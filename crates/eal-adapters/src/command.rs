//! Asynchronous ownership of the sole EAL process supervisor implementation.
use crate::Error;
use eal_process::{CancellationToken, CommandRequest, ProcessOutcome, StreamLimits, Supervisor};
use std::path::PathBuf;

/// Explicit operator selection wins. Otherwise use the installed sibling companion.
/// Missing companions fail rather than falling back to unsupervised execution.
pub fn companion() -> Result<PathBuf, Error> {
    if let Some(path) = std::env::var_os("EAL_COMMAND_SUPERVISOR") {
        let path = PathBuf::from(path);
        if !path.is_absolute() || !path.is_file() {
            return Err(Error::InvalidInput(
                "EAL_COMMAND_SUPERVISOR requires an absolute regular executable path".into(),
            ));
        }
        return Ok(path);
    }
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| Error::InvalidInput("Cannot locate process companion directory".into()))?;
    if let Some(companion) = find_in(directory) {
        return Ok(companion);
    }
    // Cargo runs direct library integration harnesses from target/{profile}/deps.
    // The packaged executable companion remains in the adjacent profile directory.
    if directory.file_name().is_some_and(|name| name == "deps")
        && let Some(companion) = directory.parent().and_then(find_in)
    {
        return Ok(companion);
    }
    if let Some(path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path) {
            if !directory.is_absolute() {
                continue;
            }
            if let Some(companion) = find_in(&directory) {
                return Ok(companion);
            }
        }
    }
    Err(Error::InvalidInput(
        "Install the EAL process supervisor companion or set EAL_COMMAND_SUPERVISOR".into(),
    ))
}

fn find_in(directory: &std::path::Path) -> Option<PathBuf> {
    [
        "engarg-process-supervisor",
        "eal-process-supervisor",
        "eal-command-supervisor",
    ]
    .into_iter()
    .map(|name| directory.join(format!("{name}{}", std::env::consts::EXE_SUFFIX)))
    .find(|candidate| candidate.is_file())
}

struct CancelOnDrop(CancellationToken);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

/// Cancellation closes ownership through the supervisor protocol and cleans descendants.
pub async fn run(request: CommandRequest) -> Result<ProcessOutcome, Error> {
    run_with(Supervisor::new(companion()?), request).await
}

pub async fn run_with_stream_limits(
    request: CommandRequest,
    limits: StreamLimits,
) -> Result<ProcessOutcome, Error> {
    execute(Supervisor::new(companion()?), request, Some(limits)).await
}

pub async fn run_with(
    supervisor: Supervisor,
    request: CommandRequest,
) -> Result<ProcessOutcome, Error> {
    execute(supervisor, request, None).await
}

async fn execute(
    supervisor: Supervisor,
    request: CommandRequest,
    limits: Option<StreamLimits>,
) -> Result<ProcessOutcome, Error> {
    let cancellation = CancellationToken::default();
    let guard = CancelOnDrop(cancellation.clone());
    let result = tokio::task::spawn_blocking(move || match limits {
        Some(limits) => {
            supervisor.run_with_stream_limits_and_cancellation(request, None, &cancellation, limits)
        }
        None => supervisor.run_with_cancellation(request, None, &cancellation),
    })
    .await
    .map_err(|_| Error::Provider("Process supervisor worker failed".into()))?
    .map_err(|_| Error::Provider("Process supervisor could not complete bounded execution".into()));
    drop(guard);
    result
}
