//! Private bounded framing between the application and its command supervisor.
use super::{ExecutionError, ProcessError, ProcessOutcome, StartupHandshake};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

pub(crate) const MAX_CONFIG_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_STATUS_BYTES: usize = 64 * 1024;
pub(crate) const MAGIC: &[u8; 8] = b"EALPRC01";
pub(crate) const READY_MAGIC: &[u8; 8] = b"EALRDY01";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Configuration {
    pub argv: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub input_length: usize,
    pub execution_timeout: Duration,
    pub max_output_bytes: usize,
    pub handshake: Option<StartupHandshake>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Status {
    pub returncode: i32,
    pub timed_out: bool,
    pub execution_error: Option<ExecutionError>,
    pub startup_completed: bool,
    pub stdout_length: usize,
    pub stderr_length: usize,
}

pub(crate) fn decode(bytes: &[u8], limit: usize) -> Result<ProcessOutcome, ProcessError> {
    let invalid =
        || ProcessError::Protocol("Invalid collector status from command supervisor".into());
    if bytes.len() < 16 || &bytes[..8] != MAGIC {
        return Err(invalid());
    }
    let length = u64::from_be_bytes(bytes[8..16].try_into().map_err(|_| invalid())?);
    let length = usize::try_from(length).map_err(|_| invalid())?;
    if length > MAX_STATUS_BYTES || bytes.len() < 16 + length {
        return Err(invalid());
    }
    let status: Status = serde_json::from_slice(&bytes[16..16 + length]).map_err(|_| invalid())?;
    let streams = status
        .stdout_length
        .checked_add(status.stderr_length)
        .ok_or_else(invalid)?;
    if streams > limit
        || streams.checked_add(16 + length) != Some(bytes.len())
        || !(-127..=255).contains(&status.returncode)
    {
        return Err(invalid());
    }
    let split = 16 + length + status.stdout_length;
    Ok(ProcessOutcome::new(
        bytes[16 + length..split].to_vec(),
        bytes[split..].to_vec(),
        status,
    ))
}

#[cfg(unix)]
pub(crate) fn monotonic_nanos() -> u128 {
    let time = rustix::time::clock_gettime(rustix::time::ClockId::Monotonic);
    (time.tv_sec as u128) * 1_000_000_000 + (time.tv_nsec as u128)
}

#[cfg(unix)]
pub(crate) fn nonblocking(fd: impl std::os::fd::AsFd) -> std::io::Result<()> {
    let flags = rustix::fs::fcntl_getfl(&fd)?;
    rustix::fs::fcntl_setfl(fd, flags | rustix::fs::OFlags::NONBLOCK)?;
    Ok(())
}

#[cfg(unix)]
pub(crate) fn poll(fds: &mut [rustix::event::PollFd<'_>]) -> std::io::Result<()> {
    let timeout = rustix::time::Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    match rustix::event::poll(fds, Some(&timeout)) {
        Ok(_) => Ok(()),
        Err(rustix::io::Errno::INTR) => Ok(()),
        Err(error) => Err(error.into()),
    }
}
