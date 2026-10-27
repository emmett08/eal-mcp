//! Private bounded framing between the application and its command supervisor.
use super::{ExecutionError, ProcessError, ProcessOutcome, StartupHandshake, StreamLimits};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_limits: Option<StreamLimits>,
    pub handshake: Option<StartupHandshake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duplex_input_limit: Option<usize>,
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

pub(crate) fn decode(
    bytes: &[u8],
    limit: usize,
    stream_limits: Option<StreamLimits>,
) -> Result<ProcessOutcome, ProcessError> {
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
    if let Some(limits) = stream_limits {
        validate_stream_lengths(&status, limits)?;
    }
    let split = 16 + length + status.stdout_length;
    Ok(ProcessOutcome::new(
        bytes[16 + length..split].to_vec(),
        bytes[split..].to_vec(),
        status,
    ))
}

fn validate_stream_lengths(status: &Status, limits: StreamLimits) -> Result<(), ProcessError> {
    if status.stdout_length > limits.max_stdout_bytes
        || status.stderr_length > limits.max_stderr_bytes
    {
        return Err(ProcessError::Protocol(
            "Supervisor response exceeds an independent stream byte limit".into(),
        ));
    }
    Ok(())
}

/// Reject an oversized declared stream as soon as the bounded status header arrives.
pub(crate) fn check_stream_limits(
    bytes: &[u8],
    limits: StreamLimits,
) -> Result<bool, ProcessError> {
    if bytes.len() < 16 {
        return Ok(false);
    }
    if &bytes[..8] != MAGIC {
        return Err(ProcessError::Protocol(
            "Invalid supervisor response frame".into(),
        ));
    }
    let length = u64::from_be_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| ProcessError::Protocol("Invalid supervisor status length".into()))?,
    );
    let length = usize::try_from(length)
        .map_err(|_| ProcessError::Protocol("Invalid supervisor status length".into()))?;
    if length > MAX_STATUS_BYTES {
        return Err(ProcessError::Protocol(
            "Supervisor status exceeds its byte limit".into(),
        ));
    }
    if bytes.len() < 16 + length {
        return Ok(false);
    }
    let status: Status = serde_json::from_slice(&bytes[16..16 + length])
        .map_err(|_| ProcessError::Protocol("Invalid supervisor status".into()))?;
    validate_stream_lengths(&status, limits)?;
    Ok(true)
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_requests_omit_version_two_fields_and_new_limits_are_versioned() {
        let mut configuration = Configuration {
            argv: vec!["/bin/true".into()],
            environment: BTreeMap::new(),
            input_length: 0,
            execution_timeout: Duration::from_secs(1),
            max_output_bytes: 16,
            protocol_version: None,
            stream_limits: None,
            handshake: None,
            duplex_input_limit: None,
        };
        let old = serde_json::to_value(&configuration).unwrap();
        assert!(old.get("stream_limits").is_none());
        assert!(old.get("protocol_version").is_none());
        configuration.protocol_version = Some(2);
        configuration.stream_limits = Some(StreamLimits {
            max_stdout_bytes: 12,
            max_stderr_bytes: 4,
        });
        let new = serde_json::to_value(&configuration).unwrap();
        assert_eq!(new["protocol_version"], 2);
        assert_eq!(new["stream_limits"]["max_stderr_bytes"], 4);
    }
    #[test]
    fn over_limit_announced_stream_is_rejected_before_its_payload() {
        let status = Status {
            returncode: 0,
            timed_out: false,
            execution_error: None,
            startup_completed: true,
            stdout_length: 0,
            stderr_length: 1_000_000,
        };
        let encoded = serde_json::to_vec(&status).unwrap();
        let mut frame = MAGIC.to_vec();
        frame.extend_from_slice(&(encoded.len() as u64).to_be_bytes());
        frame.extend_from_slice(&encoded);
        assert!(
            check_stream_limits(
                &frame,
                StreamLimits {
                    max_stdout_bytes: 16_777_216,
                    max_stderr_bytes: 65_536
                }
            )
            .is_err()
        );
    }
}
