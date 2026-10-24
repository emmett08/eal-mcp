//! The supervisor owns all collector-group cleanup before its lease FD closes.
use std::os::fd::OwnedFd;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use rustix::event::{PollFd, PollFlags};
use rustix::process::{Pid, Signal, WaitOptions};

use crate::process::{
    ExecutionError,
    protocol::{self, Configuration, MAGIC, MAX_CONFIG_BYTES, Status},
};

struct Collector {
    child: Option<Child>,
    group: Pid,
}
impl Collector {
    fn finish(&mut self) -> std::io::Result<ExitStatus> {
        let _ = rustix::process::kill_process_group(self.group, Signal::KILL);
        let status = self
            .child
            .as_mut()
            .ok_or_else(|| std::io::Error::other("Collector already reaped"))?
            .wait()?;
        loop {
            loop {
                match rustix::process::waitpgid(self.group, WaitOptions::NOHANG) {
                    Ok(Some(_)) => continue,
                    Ok(None) | Err(rustix::io::Errno::CHILD) => break,
                    Err(rustix::io::Errno::INTR) => continue,
                    Err(error) => return Err(error.into()),
                }
            }
            match rustix::process::test_kill_process_group(self.group) {
                Err(rustix::io::Errno::SRCH) => break,
                Ok(()) => {
                    let _ = rustix::process::kill_process_group(self.group, Signal::KILL);
                }
                Err(error) => return Err(error.into()),
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        self.child.take();
        Ok(status)
    }
}
impl Drop for Collector {
    fn drop(&mut self) {
        if self.child.is_some() {
            // If cleanup is temporarily interrupted, retain the process (and
            // inherited lease) and retry rather than releasing live ownership.
            while self.finish().is_err() {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

pub(super) fn entrypoint() -> i32 {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let Some(deadline) = arguments
        .first()
        .filter(|_| arguments.len() == 1)
        .and_then(|value| value.parse::<u128>().ok())
    else {
        return 2;
    };
    let stopping = Arc::new(AtomicBool::new(false));
    for signal in [
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGHUP,
    ] {
        if signal_hook::flag::register(signal, stopping.clone()).is_err() {
            return 2;
        }
    }
    #[cfg(target_os = "linux")]
    if rustix::process::set_child_subreaper(Some(rustix::process::getpid())).is_err() {
        return 2;
    }
    let control = std::io::stdin();
    let output = std::io::stdout();
    if protocol::nonblocking(&control).is_err() || protocol::nonblocking(&output).is_err() {
        return 2;
    }
    let result = match configuration(&control, deadline, &stopping) {
        Ok(configuration) => execute(configuration, &control, deadline, &stopping),
        Err(error) => (
            Vec::new(),
            Vec::new(),
            Status {
                returncode: 127,
                timed_out: matches!(error, ExecutionError::Timeout),
                execution_error: Some(error),
                startup_completed: false,
                stdout_length: 0,
                stderr_length: 0,
            },
        ),
    };
    let (stdout, stderr, mut status) = result;
    status.stdout_length = stdout.len();
    status.stderr_length = stderr.len();
    let Ok(encoded) = serde_json::to_vec(&status) else {
        return 2;
    };
    let mut header = Vec::with_capacity(16 + encoded.len());
    header.extend_from_slice(MAGIC);
    header.extend_from_slice(&(encoded.len() as u64).to_be_bytes());
    header.extend_from_slice(&encoded);
    for data in [&header[..], &stdout[..], &stderr[..]] {
        if write_result(&output, &control, data, &stopping).is_err() {
            return 0;
        }
    }
    0
}

fn interrupted(deadline: u128, stopping: &AtomicBool) -> Option<ExecutionError> {
    if stopping.load(Ordering::Acquire) {
        Some(ExecutionError::Cancelled)
    } else if protocol::monotonic_nanos() >= deadline {
        Some(ExecutionError::Timeout)
    } else {
        None
    }
}

fn configuration(
    control: &std::io::Stdin,
    deadline: u128,
    stopping: &AtomicBool,
) -> Result<Configuration, ExecutionError> {
    let mut header = [0u8; 8];
    read_exact(control, &mut header, deadline, stopping)?;
    let length = usize::try_from(u64::from_be_bytes(header)).map_err(|_| {
        ExecutionError::Supervisor("Invalid command supervisor configuration".into())
    })?;
    if length > MAX_CONFIG_BYTES {
        return Err(ExecutionError::Supervisor(
            "Command supervisor configuration exceeds its byte limit".into(),
        ));
    }
    let mut encoded = vec![0u8; length];
    read_exact(control, &mut encoded, deadline, stopping)?;
    let configuration: Configuration = serde_json::from_slice(&encoded).map_err(|_| {
        ExecutionError::Supervisor("Invalid command supervisor configuration".into())
    })?;
    if configuration.argv.is_empty()
        || configuration.argv[0].is_empty()
        || configuration.max_output_bytes == 0
        || configuration.execution_timeout.is_zero()
    {
        return Err(ExecutionError::Supervisor(
            "Invalid command supervisor configuration".into(),
        ));
    }
    match (configuration.protocol_version, configuration.stream_limits) {
        (None, None) => {}
        (Some(2), Some(limits)) => limits
            .validate(configuration.max_output_bytes)
            .map_err(|_| ExecutionError::Supervisor("Invalid independent stream limits".into()))?,
        _ => {
            return Err(ExecutionError::Supervisor(
                "Unsupported supervisor request protocol version".into(),
            ));
        }
    }
    if let Some(handshake) = &configuration.handshake {
        let line = &handshake.expected_first_stdout_line;
        if line.is_empty()
            || line.len() > 16384
            || !line.ends_with(b"\n")
            || line[..line.len() - 1].contains(&b'\n')
            || handshake.startup_timeout.is_zero()
        {
            return Err(ExecutionError::Supervisor(
                "Invalid command supervisor configuration".into(),
            ));
        }
    }
    Ok(configuration)
}

fn read_exact(
    control: &std::io::Stdin,
    mut destination: &mut [u8],
    deadline: u128,
    stopping: &AtomicBool,
) -> Result<(), ExecutionError> {
    while !destination.is_empty() {
        if let Some(error) = interrupted(deadline, stopping) {
            return Err(error);
        }
        let mut polls = [PollFd::new(control, PollFlags::IN)];
        protocol::poll(&mut polls)
            .map_err(|error| ExecutionError::Supervisor(error.to_string()))?;
        if polls[0].revents().contains(PollFlags::HUP) {
            return Err(interrupted(deadline, stopping).unwrap_or(ExecutionError::Cancelled));
        }
        let maximum = destination.len().min(65536);
        match rustix::io::read(control, &mut destination[..maximum]) {
            Ok(0) => {
                return Err(interrupted(deadline, stopping).unwrap_or(ExecutionError::Cancelled));
            }
            Ok(count) => destination = &mut destination[count..],
            Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
            Err(error) => return Err(ExecutionError::Supervisor(error.to_string())),
        }
    }
    Ok(())
}

fn execute(
    configuration: Configuration,
    control: &std::io::Stdin,
    mut deadline: u128,
    stopping: &AtomicBool,
) -> (Vec<u8>, Vec<u8>, Status) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut failure = interrupted(deadline, stopping);
    let mut ready = configuration.handshake.is_none();
    let mut ready_position = 0usize;
    let mut returncode = 127;
    if failure.is_none() {
        let mut command = Command::new(&configuration.argv[0]);
        command
            .args(&configuration.argv[1..])
            .env_clear()
            .envs(&configuration.environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        match command.spawn() {
            Err(error) => {
                failure = Some(ExecutionError::Spawn(error.to_string()));
                let bytes = error.to_string().into_bytes();
                stderr.extend_from_slice(
                    &bytes[..bytes.len().min(configuration.max_output_bytes).min(
                        configuration
                            .stream_limits
                            .map_or(usize::MAX, |limits| limits.max_stderr_bytes),
                    )],
                );
            }
            Ok(mut child) => {
                let Some(group) = Pid::from_raw(child.id() as i32) else {
                    let _ = child.kill();
                    let _ = child.wait();
                    return (
                        stdout,
                        stderr,
                        Status {
                            returncode: 127,
                            timed_out: false,
                            execution_error: Some(ExecutionError::Supervisor(
                                "Invalid collector process group".into(),
                            )),
                            startup_completed: ready,
                            stdout_length: 0,
                            stderr_length: 0,
                        },
                    );
                };
                let mut input = child.stdin.take();
                let mut out = child.stdout.take().map(OwnedFd::from);
                let mut err = child.stderr.take().map(OwnedFd::from);
                let mut collector = Collector {
                    child: Some(child),
                    group,
                };
                let setup = input
                    .as_ref()
                    .map(protocol::nonblocking)
                    .transpose()
                    .and_then(|_| out.as_ref().map(protocol::nonblocking).transpose())
                    .and_then(|_| err.as_ref().map(protocol::nonblocking).transpose());
                if let Err(error) = setup {
                    failure = Some(ExecutionError::Supervisor(error.to_string()));
                }
                let mut pending = Vec::new();
                let mut input_offset = 0usize;
                let mut input_left = configuration.input_length;
                while failure.is_none() {
                    if let Some(error) = interrupted(deadline, stopping) {
                        failure = Some(error);
                        break;
                    }
                    if let Some(child) = collector.child.as_mut() {
                        match child.try_wait() {
                            Ok(Some(_)) => break,
                            Ok(None) => {}
                            Err(error) => {
                                failure = Some(ExecutionError::Supervisor(error.to_string()));
                                break;
                            }
                        }
                    }
                    if input_left == 0 && input_offset == pending.len() {
                        input.take();
                    }
                    let (control_flags, out_flags, err_flags, input_flags) = {
                        let mut polls = vec![PollFd::new(
                            control,
                            if input_left > 0 && input_offset == pending.len() {
                                PollFlags::IN
                            } else {
                                PollFlags::empty()
                            },
                        )];
                        let out_index = out.as_ref().map(|pipe| {
                            let index = polls.len();
                            polls.push(PollFd::new(pipe, PollFlags::IN));
                            index
                        });
                        let err_index = err.as_ref().map(|pipe| {
                            let index = polls.len();
                            polls.push(PollFd::new(pipe, PollFlags::IN));
                            index
                        });
                        let input_index = input
                            .as_ref()
                            .filter(|_| ready && input_offset < pending.len())
                            .map(|pipe| {
                                let index = polls.len();
                                polls.push(PollFd::new(pipe, PollFlags::OUT));
                                index
                            });
                        if let Err(error) = protocol::poll(&mut polls) {
                            failure = Some(ExecutionError::Supervisor(error.to_string()));
                        }
                        let flags = |index: Option<usize>| {
                            index
                                .map(|index| polls[index].revents())
                                .unwrap_or(PollFlags::empty())
                        };
                        (
                            polls[0].revents(),
                            flags(out_index),
                            flags(err_index),
                            flags(input_index),
                        )
                    };
                    if failure.is_some() {
                        break;
                    }
                    if control_flags.contains(PollFlags::HUP) {
                        failure =
                            interrupted(deadline, stopping).or(Some(ExecutionError::Cancelled));
                        break;
                    }
                    let mut chunk = [0u8; 65536];
                    for (pipe, flags, is_stdout) in
                        [(&mut out, out_flags, true), (&mut err, err_flags, false)]
                    {
                        if flags.intersects(PollFlags::IN | PollFlags::HUP | PollFlags::ERR)
                            && let Some(stream) = pipe.as_ref()
                        {
                            match rustix::io::read(stream, &mut chunk) {
                                Ok(0) => {
                                    pipe.take();
                                }
                                Ok(count) => {
                                    capture(
                                        &chunk[..count],
                                        is_stdout,
                                        &mut stdout,
                                        &mut stderr,
                                        configuration.max_output_bytes,
                                        configuration.stream_limits,
                                        &mut failure,
                                    );
                                    if is_stdout
                                        && failure.is_none()
                                        && let Err(error) = observe_ready(
                                            &chunk[..count],
                                            &configuration,
                                            &mut ready,
                                            &mut ready_position,
                                            &mut deadline,
                                        )
                                    {
                                        failure = Some(error);
                                    }
                                }
                                Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
                                Err(error) => {
                                    failure = Some(ExecutionError::Supervisor(error.to_string()))
                                }
                            }
                        }
                        if failure.is_some() {
                            break;
                        }
                    }
                    if failure.is_some() {
                        break;
                    }
                    if control_flags.contains(PollFlags::IN)
                        && input_left > 0
                        && input_offset == pending.len()
                    {
                        let maximum = input_left.min(chunk.len());
                        match rustix::io::read(control, &mut chunk[..maximum]) {
                            Ok(0) => {
                                failure = interrupted(deadline, stopping)
                                    .or(Some(ExecutionError::Cancelled));
                                break;
                            }
                            Ok(count) => {
                                pending.clear();
                                pending.extend_from_slice(&chunk[..count]);
                                input_offset = 0;
                                input_left -= count;
                            }
                            Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
                            Err(error) => {
                                failure = Some(ExecutionError::Supervisor(error.to_string()))
                            }
                        }
                    }
                    if input_flags.intersects(PollFlags::OUT | PollFlags::HUP | PollFlags::ERR)
                        && let Some(stream) = &input
                    {
                        match rustix::io::write(stream, &pending[input_offset..]) {
                            Ok(count) => input_offset += count,
                            Err(rustix::io::Errno::PIPE) => {
                                input.take();
                                pending.clear();
                                input_offset = 0;
                            }
                            Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
                            Err(error) => {
                                failure = Some(ExecutionError::Supervisor(error.to_string()))
                            }
                        }
                    }
                }
                input.take();
                match collector.finish() {
                    Ok(status) => {
                        returncode = status
                            .code()
                            .unwrap_or_else(|| -status.signal().unwrap_or(127))
                    }
                    Err(error) => {
                        failure
                            .get_or_insert_with(|| ExecutionError::Supervisor(error.to_string()));
                    }
                }
                for pipe in [out, err].into_iter().enumerate() {
                    if let (index, Some(stream)) = pipe {
                        let mut chunk = [0u8; 65536];
                        loop {
                            match rustix::io::read(&stream, &mut chunk) {
                                Ok(0) => break,
                                Ok(count) => {
                                    if failure.is_none() {
                                        capture(
                                            &chunk[..count],
                                            index == 0,
                                            &mut stdout,
                                            &mut stderr,
                                            configuration.max_output_bytes,
                                            configuration.stream_limits,
                                            &mut failure,
                                        );
                                        if index == 0
                                            && failure.is_none()
                                            && let Err(error) = observe_ready(
                                                &chunk[..count],
                                                &configuration,
                                                &mut ready,
                                                &mut ready_position,
                                                &mut deadline,
                                            )
                                        {
                                            failure = Some(error);
                                        }
                                    }
                                }
                                Err(rustix::io::Errno::INTR) => continue,
                                Err(rustix::io::Errno::AGAIN) => {
                                    failure.get_or_insert_with(||ExecutionError::Supervisor("Collector output pipe remained open after process-group cleanup".into()));
                                    break;
                                }
                                Err(_) => break,
                            }
                        }
                    }
                }
            }
        }
    }
    let timed_out = matches!(failure, Some(ExecutionError::Timeout));
    (
        stdout,
        stderr,
        Status {
            returncode,
            timed_out,
            execution_error: failure,
            startup_completed: ready,
            stdout_length: 0,
            stderr_length: 0,
        },
    )
}

fn capture(
    bytes: &[u8],
    is_stdout: bool,
    stdout: &mut Vec<u8>,
    stderr: &mut Vec<u8>,
    limit: usize,
    stream_limits: Option<crate::process::StreamLimits>,
    failure: &mut Option<ExecutionError>,
) {
    let remaining = limit
        .saturating_sub(stdout.len())
        .saturating_sub(stderr.len());
    let target = if is_stdout { stdout } else { stderr };
    let stream_remaining = stream_limits.map_or(remaining, |limits| {
        let limit = if is_stdout {
            limits.max_stdout_bytes
        } else {
            limits.max_stderr_bytes
        };
        limit.saturating_sub(target.len())
    });
    let remaining = remaining.min(stream_remaining);
    target.extend_from_slice(&bytes[..bytes.len().min(remaining)]);
    if bytes.len() > remaining && failure.is_none() {
        *failure = Some(ExecutionError::OutputLimit);
    }
}

fn observe_ready(
    bytes: &[u8],
    configuration: &Configuration,
    ready: &mut bool,
    position: &mut usize,
    deadline: &mut u128,
) -> Result<(), ExecutionError> {
    if *ready {
        return Ok(());
    }
    if let Some(handshake) = &configuration.handshake {
        for &byte in bytes {
            if byte != handshake.expected_first_stdout_line[*position] {
                return Err(ExecutionError::Startup(
                    "Invalid method worker startup frame".into(),
                ));
            }
            *position += 1;
            if *position == handshake.expected_first_stdout_line.len() {
                *ready = true;
                *deadline = protocol::monotonic_nanos()
                    .saturating_add(configuration.execution_timeout.as_nanos());
                loop {
                    match rustix::io::write(std::io::stdout(), protocol::READY_MAGIC) {
                        Ok(8) => break,
                        Err(rustix::io::Errno::INTR) => continue,
                        Err(error) => return Err(ExecutionError::Supervisor(error.to_string())),
                        Ok(_) => {
                            return Err(ExecutionError::Supervisor(
                                "Incomplete supervisor startup notification".into(),
                            ));
                        }
                    }
                }
                break;
            }
        }
    }
    Ok(())
}

fn write_result(
    output: &std::io::Stdout,
    control: &std::io::Stdin,
    mut data: &[u8],
    _stopping: &AtomicBool,
) -> std::io::Result<()> {
    while !data.is_empty() {
        let mut polls = [
            PollFd::new(output, PollFlags::OUT),
            PollFd::new(control, PollFlags::empty()),
        ];
        protocol::poll(&mut polls)?;
        // After cleanup, a closed owner pipe is harmless; still deliver the
        // bounded final status to a live reader after an intentional timeout.
        match rustix::io::write(output, &data[..data.len().min(65536)]) {
            Ok(count) => data = &data[count..],
            Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
