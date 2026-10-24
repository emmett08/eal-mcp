//! Application-side ownership, deadline and bounded result transfer.
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

use rustix::event::{PollFd, PollFlags};

use super::Retention;
use super::{
    CancellationToken, CommandRequest, ProcessError, ProcessOutcome, StartupHandshake,
    StreamLimits,
    protocol::{self, Configuration, MAX_CONFIG_BYTES, MAX_STATUS_BYTES},
};

struct Owner {
    child: Option<Child>,
    control: Option<ChildStdin>,
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.control.take();
        let Some(mut child) = self.child.take() else {
            return;
        };
        if matches!(child.try_wait(), Ok(Some(_))) {
            return;
        }
        // Never terminate a supervisor while it may still hold a live group's
        // lease. Its control pipe is closed; retain a reaper for its own exit.
        let _ = std::thread::Builder::new()
            .name("eal-supervisor-reaper".into())
            .spawn(move || {
                let _ = child.wait();
            });
    }
}

pub(super) fn run(
    executable: &Path,
    request: CommandRequest,
    lease: Option<&dyn Retention>,
    cancellation: &CancellationToken,
    handshake: Option<StartupHandshake>,
    stream_limits: Option<StreamLimits>,
) -> Result<ProcessOutcome, ProcessError> {
    let started = protocol::monotonic_nanos();
    validate(&request, handshake.as_ref())?;
    if let Some(limits) = stream_limits {
        limits.validate(request.max_output_bytes)?;
    }
    let initial_timeout = handshake
        .as_ref()
        .map(|ready| ready.startup_timeout)
        .unwrap_or(request.timeout);
    let initial_deadline = started
        .checked_add(initial_timeout.as_nanos())
        .ok_or_else(|| ProcessError::Invalid("Command deadline exceeds clock resources".into()))?;
    let mut awaiting_ready = handshake.is_some();
    // The companion enforces the exact clock. A short notification allowance
    // prevents the parent watchdog racing an on-time READY in its output pipe.
    let mut outer_deadline =
        initial_deadline.saturating_add(if awaiting_ready { 50_000_000 } else { 0 });
    let configuration = Configuration {
        argv: request.argv,
        environment: request.environment,
        input_length: request.input.len(),
        execution_timeout: request.timeout,
        max_output_bytes: request.max_output_bytes,
        protocol_version: stream_limits.map(|_| 2),
        stream_limits,
        handshake,
    };
    let encoded = serde_json::to_vec(&configuration)
        .map_err(|error| ProcessError::Invalid(error.to_string()))?;
    if encoded.len() > MAX_CONFIG_BYTES {
        return Err(ProcessError::Invalid(
            "Command supervisor configuration exceeds its byte limit".into(),
        ));
    }
    let mut prefix = Vec::with_capacity(8 + encoded.len());
    prefix.extend_from_slice(&(encoded.len() as u64).to_be_bytes());
    prefix.extend_from_slice(&encoded);
    let inherited = match lease {
        Some(lease) => lease.duplicate_for_supervisor()?,
        None => Stdio::null(),
    };
    let child = Command::new(executable)
        .arg(initial_deadline.to_string())
        .current_dir(&request.cwd)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(inherited)
        .process_group(0)
        .spawn()?;
    let supervisor_pid = child.id();
    let mut owner = Owner {
        child: Some(child),
        control: None,
    };
    let child = owner
        .child
        .as_mut()
        .ok_or_else(|| ProcessError::Protocol("Missing supervisor process".into()))?;
    owner.control = child.stdin.take();
    let output = child
        .stdout
        .take()
        .ok_or_else(|| ProcessError::Protocol("Missing supervisor output pipe".into()))?;
    protocol::nonblocking(&output)?;
    if let Some(control) = &owner.control {
        protocol::nonblocking(control)?;
    }
    let maximum = request
        .max_output_bytes
        .checked_add(MAX_STATUS_BYTES + 24)
        .ok_or_else(|| {
            ProcessError::Invalid("Command output budget exceeds protocol resources".into())
        })?;
    let mut bytes = Vec::new();
    let mut announced_streams_checked = stream_limits.is_none();
    let mut prefix_offset = 0usize;
    let mut input_offset = 0usize;
    let mut eof = false;
    let mut closing = None;
    loop {
        if (cancellation.is_cancelled() || protocol::monotonic_nanos() >= outer_deadline)
            && owner.control.is_some()
        {
            owner.control.take();
            closing = Some(Instant::now());
        }
        if closing.is_some_and(|time: Instant| time.elapsed() > Duration::from_millis(500)) {
            return Err(ProcessError::Protocol(
                "Command supervisor did not finish cleanup after cancellation".into(),
            ));
        }
        let (out_ready, in_ready) = {
            let mut polls = vec![PollFd::new(&output, PollFlags::IN)];
            if let Some(control) = &owner.control
                && (prefix_offset < prefix.len() || input_offset < request.input.len())
            {
                polls.push(PollFd::new(control, PollFlags::OUT));
            }
            protocol::poll(&mut polls)?;
            (
                polls[0].revents(),
                polls
                    .get(1)
                    .map(PollFd::revents)
                    .unwrap_or(PollFlags::empty()),
            )
        };
        if out_ready.intersects(PollFlags::IN | PollFlags::HUP | PollFlags::ERR) && !eof {
            let mut chunk = [0u8; 65536];
            loop {
                match rustix::io::read(&output, &mut chunk) {
                    Ok(0) => {
                        eof = true;
                        break;
                    }
                    Ok(count) => {
                        if count > maximum.saturating_sub(bytes.len()) {
                            return Err(ProcessError::Protocol(
                                "Command supervisor output exceeds its protocol byte limit".into(),
                            ));
                        }
                        bytes.extend_from_slice(&chunk[..count]);
                        if awaiting_ready && bytes.len() >= 8 {
                            if &bytes[..8] == protocol::READY_MAGIC {
                                bytes.drain(..8);
                                outer_deadline = protocol::monotonic_nanos()
                                    .saturating_add(request.timeout.as_nanos())
                                    .saturating_add(50_000_000);
                            } else if &bytes[..8] != protocol::MAGIC {
                                return Err(ProcessError::Protocol(
                                    "Invalid command supervisor startup notification".into(),
                                ));
                            }
                            awaiting_ready = false;
                        }
                        if !announced_streams_checked && !awaiting_ready {
                            announced_streams_checked = protocol::check_stream_limits(
                                &bytes,
                                stream_limits.expect("limits selected"),
                            )?;
                        }
                    }
                    Err(rustix::io::Errno::AGAIN) => break,
                    Err(rustix::io::Errno::INTR) => continue,
                    Err(error) => return Err(ProcessError::Io(error.into())),
                }
            }
        }
        if in_ready.intersects(PollFlags::OUT | PollFlags::ERR | PollFlags::HUP)
            && let Some(control) = &owner.control
        {
            let data = if prefix_offset < prefix.len() {
                &prefix[prefix_offset..]
            } else {
                &request.input[input_offset..]
            };
            match rustix::io::write(control, &data[..data.len().min(65536)]) {
                Ok(count) => {
                    if prefix_offset < prefix.len() {
                        prefix_offset += count;
                    } else {
                        input_offset += count;
                    }
                }
                Err(rustix::io::Errno::PIPE) => {
                    owner.control.take();
                    closing.get_or_insert_with(Instant::now);
                }
                Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
                Err(error) => return Err(ProcessError::Io(error.into())),
            }
        }
        let exited = owner
            .child
            .as_mut()
            .ok_or_else(|| ProcessError::Protocol("Missing supervisor process".into()))?
            .try_wait()?
            .is_some();
        if exited && eof {
            break;
        }
    }
    let mut outcome = protocol::decode(&bytes, request.max_output_bytes, stream_limits)?;
    outcome.supervisor_pid = Some(supervisor_pid);
    Ok(outcome)
}

fn validate(
    request: &CommandRequest,
    handshake: Option<&StartupHandshake>,
) -> Result<(), ProcessError> {
    if request.argv.is_empty()
        || request.argv[0].is_empty()
        || request.timeout.is_zero()
        || request.max_output_bytes == 0
    {
        return Err(ProcessError::Invalid(
            "Command requires argv, a positive timeout and a positive output byte budget".into(),
        ));
    }
    let mut size = 0usize;
    for value in &request.argv {
        if value.contains('\0') {
            return Err(ProcessError::Invalid(
                "Command arguments cannot contain NUL".into(),
            ));
        }
        size = size.saturating_add(value.len());
    }
    for (key, value) in &request.environment {
        if key.is_empty() || key.contains(['=', '\0']) || value.contains('\0') {
            return Err(ProcessError::Invalid(
                "Command environment requires valid names and NUL-free values".into(),
            ));
        }
        size = size.saturating_add(key.len()).saturating_add(value.len());
    }
    if let Some(ready) = handshake {
        let expected = &ready.expected_first_stdout_line;
        if ready.startup_timeout.is_zero()
            || expected.is_empty()
            || expected.len() > 16384
            || !expected.ends_with(b"\n")
            || expected[..expected.len() - 1].contains(&b'\n')
        {
            return Err(ProcessError::Invalid("Command startup handshake requires one bounded complete line and a positive timeout".into()));
        }
    }
    if size > MAX_CONFIG_BYTES {
        return Err(ProcessError::Invalid(
            "Command supervisor configuration exceeds its byte limit".into(),
        ));
    }
    request
        .max_output_bytes
        .checked_add(MAX_STATUS_BYTES + 24)
        .ok_or_else(|| {
            ProcessError::Invalid("Command output budget exceeds protocol resources".into())
        })?;
    Ok(())
}
