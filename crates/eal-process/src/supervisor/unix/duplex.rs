//! Bounded bidirectional forwarding; Collector remains the sole cleanup owner.
use super::{Collector, interrupted};
use crate::process::protocol::{self, Configuration};
use rustix::{
    event::{PollFd, PollFlags},
    process::Pid,
};
use std::{
    os::{fd::OwnedFd, unix::process::CommandExt},
    process::{Command, Stdio},
    sync::atomic::AtomicBool,
};

pub(super) fn execute(
    configuration: Configuration,
    control: &std::io::Stdin,
    output: &std::io::Stdout,
    deadline: u128,
    stopping: &AtomicBool,
) -> i32 {
    if interrupted(deadline, stopping).is_some() {
        return 1;
    }
    let mut command = Command::new(&configuration.argv[0]);
    command
        .args(&configuration.argv[1..])
        .env_clear()
        .envs(&configuration.environment)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let Ok(mut child) = command.spawn() else {
        return 2;
    };
    let Some(group) = Pid::from_raw(child.id() as i32) else {
        let _ = child.kill();
        let _ = child.wait();
        return 2;
    };
    let mut input = child.stdin.take().map(OwnedFd::from);
    let mut out = child.stdout.take().map(OwnedFd::from);
    let mut err = child.stderr.take().map(OwnedFd::from);
    let mut collector = Collector {
        child: Some(child),
        group,
    };
    for pipe in [&input, &out, &err].into_iter().flatten() {
        if protocol::nonblocking(pipe).is_err() {
            return 2;
        }
    }
    let limits = configuration.stream_limits.expect("admitted duplex limits");
    let input_limit = configuration.duplex_input_limit.expect("duplex mode");
    let mut input_bytes = 0usize;
    let mut output_bytes = 0usize;
    let mut error_bytes = 0usize;
    let mut pending_input = Vec::new();
    let mut input_offset = 0usize;
    let mut pending_output = Vec::new();
    let mut output_offset = 0usize;
    let mut exited = false;
    loop {
        if interrupted(deadline, stopping).is_some() {
            return 1;
        }
        if !exited {
            match collector.child.as_mut().expect("owned child").try_wait() {
                Ok(Some(_)) => {
                    if collector.finish().is_err() {
                        return 2;
                    }
                    input.take();
                    exited = true;
                }
                Ok(None) => {}
                Err(_) => return 2,
            }
        }
        if exited && out.is_none() && err.is_none() && output_offset == pending_output.len() {
            return 0;
        }
        let (owner_flags, output_flags, input_flags, out_flags, err_flags) = {
            let mut polls = vec![
                PollFd::new(
                    control,
                    if !exited && input_offset == pending_input.len() {
                        PollFlags::IN
                    } else {
                        PollFlags::empty()
                    },
                ),
                PollFd::new(
                    output,
                    if output_offset < pending_output.len() {
                        PollFlags::OUT
                    } else {
                        PollFlags::empty()
                    },
                ),
            ];
            let input_index = input
                .as_ref()
                .filter(|_| input_offset < pending_input.len())
                .map(|p| {
                    let i = polls.len();
                    polls.push(PollFd::new(p, PollFlags::OUT));
                    i
                });
            let out_index = out
                .as_ref()
                .filter(|_| output_offset == pending_output.len())
                .map(|p| {
                    let i = polls.len();
                    polls.push(PollFd::new(p, PollFlags::IN));
                    i
                });
            let err_index = err.as_ref().map(|p| {
                let i = polls.len();
                polls.push(PollFd::new(p, PollFlags::IN));
                i
            });
            if protocol::poll(&mut polls).is_err() {
                return 2;
            }
            let flags = |index: Option<usize>| {
                index
                    .map(|i| polls[i].revents())
                    .unwrap_or(PollFlags::empty())
            };
            (
                polls[0].revents(),
                polls[1].revents(),
                flags(input_index),
                flags(out_index),
                flags(err_index),
            )
        };
        if owner_flags.intersects(PollFlags::HUP | PollFlags::ERR)
            || output_flags.intersects(PollFlags::HUP | PollFlags::ERR)
        {
            return 1;
        }
        let mut chunk = [0u8; 65536];
        if owner_flags.contains(PollFlags::IN) {
            match rustix::io::read(control, &mut chunk) {
                Ok(0) => return 1,
                Ok(n) => {
                    if n > input_limit.saturating_sub(input_bytes) {
                        return 1;
                    }
                    input_bytes += n;
                    pending_input.clear();
                    pending_input.extend_from_slice(&chunk[..n]);
                    input_offset = 0;
                }
                Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
                Err(_) => return 2,
            }
        }
        if input_flags.intersects(PollFlags::OUT | PollFlags::HUP | PollFlags::ERR)
            && let Some(pipe) = &input
        {
            match rustix::io::write(pipe, &pending_input[input_offset..]) {
                Ok(n) => input_offset += n,
                Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
                Err(_) => {
                    input.take();
                    pending_input.clear();
                    input_offset = 0;
                }
            }
        }
        if out_flags.intersects(PollFlags::IN | PollFlags::HUP | PollFlags::ERR)
            && let Some(pipe) = &out
        {
            match rustix::io::read(pipe, &mut chunk) {
                Ok(0) => {
                    out.take();
                }
                Ok(n) => {
                    if n > limits.max_stdout_bytes.saturating_sub(output_bytes) {
                        return 1;
                    }
                    output_bytes += n;
                    pending_output.clear();
                    pending_output.extend_from_slice(&chunk[..n]);
                    output_offset = 0;
                }
                Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
                Err(_) => return 2,
            }
        }
        if err_flags.intersects(PollFlags::IN | PollFlags::HUP | PollFlags::ERR)
            && let Some(pipe) = &err
        {
            match rustix::io::read(pipe, &mut chunk) {
                Ok(0) => {
                    err.take();
                }
                Ok(n) => {
                    if n > limits.max_stderr_bytes.saturating_sub(error_bytes) {
                        return 1;
                    }
                    error_bytes += n;
                }
                Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
                Err(_) => return 2,
            }
        }
        if output_flags.contains(PollFlags::OUT) && output_offset < pending_output.len() {
            match rustix::io::write(output, &pending_output[output_offset..]) {
                Ok(n) => output_offset += n,
                Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
                Err(_) => return 1,
            }
        }
    }
}
