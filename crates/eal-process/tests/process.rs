#![cfg(unix)]
//! Real process-group execution, not mocked command responses.
use eal_process::process::{
    CancellationToken, CommandRequest, ExecutionError, StartupHandshake, Supervisor,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn supervisor() -> Supervisor {
    Supervisor::new(PathBuf::from(env!("CARGO_BIN_EXE_eal-process-supervisor")))
}
fn request(argv: &[&str]) -> CommandRequest {
    CommandRequest {
        argv: argv.iter().map(|value| (*value).into()).collect(),
        environment: BTreeMap::new(),
        cwd: std::env::current_dir().unwrap(),
        input: Vec::new(),
        timeout: Duration::from_secs(2),
        max_output_bytes: 1024 * 1024,
    }
}

#[test]
fn exact_environment_and_binary_request_reach_the_collector() {
    let mut command = request(&["/usr/bin/env"]);
    command.environment.insert("ONLY".into(), "captured".into());
    let result = supervisor().run(command, None).unwrap();
    assert_eq!(result.stdout, b"ONLY=captured\n");
    assert_eq!(result.returncode, 0);
    assert_eq!(result.execution_error, None);
    let mut command = request(&["/bin/cat"]);
    command.input = vec![0, 1, 255, 10];
    let result = supervisor().run(command, None).unwrap();
    assert_eq!(result.stdout, [0, 1, 255, 10]);
    assert_eq!(result.stderr_bytes, 0);
    assert_eq!(result.stdout_bytes, 4);
}

#[test]
fn collector_exit_and_signal_status_are_not_supervisor_exit_codes() {
    let result = supervisor()
        .run(
            request(&["/bin/sh", "-c", "printf abc; printf def >&2; exit 23"]),
            None,
        )
        .unwrap();
    assert_eq!(result.returncode, 23);
    assert_eq!(result.stdout, b"abc");
    assert_eq!(result.stderr, b"def");
    let result = supervisor()
        .run(request(&["/bin/sh", "-c", "kill -TERM $$"]), None)
        .unwrap();
    assert_eq!(result.returncode, -15);
}

#[test]
fn combined_output_cap_accepts_equality_and_preserves_first_failure() {
    let mut command = request(&["/bin/sh", "-c", "printf abc;printf def >&2"]);
    command.max_output_bytes = 6;
    let result = supervisor().run(command.clone(), None).unwrap();
    assert_eq!(result.execution_error, None);
    assert_eq!(result.stdout.len() + result.stderr.len(), 6);
    command.max_output_bytes = 5;
    let result = supervisor().run(command, None).unwrap();
    assert_eq!(result.execution_error, Some(ExecutionError::OutputLimit));
    assert_eq!(result.stdout.len() + result.stderr.len(), 5);
    assert!(!result.timed_out);
}

#[test]
fn blocked_collector_input_does_not_extend_the_deadline() {
    let mut command = request(&["/bin/sleep", "30"]);
    command.timeout = Duration::from_millis(150);
    command.input = vec![b'x'; 2 * 1024 * 1024];
    let start = Instant::now();
    let result = supervisor().run(command, None).unwrap();
    assert!(start.elapsed() < Duration::from_secs(2));
    assert_eq!(result.execution_error, Some(ExecutionError::Timeout));
    assert!(result.timed_out);
    assert_eq!(result.returncode, -9);
}

#[test]
fn leader_exit_still_kills_and_reaps_its_descendant_group() {
    let result = supervisor()
        .run(
            request(&["/bin/sh", "-c", "sleep 30 & printf '%s' $!; exit 0"]),
            None,
        )
        .unwrap();
    assert_eq!(result.returncode, 0);
    let pid = std::str::from_utf8(&result.stdout)
        .unwrap()
        .parse::<i32>()
        .unwrap();
    let pid = rustix::process::Pid::from_raw(pid).unwrap();
    assert_eq!(
        rustix::process::test_kill_process(pid),
        Err(rustix::io::Errno::SRCH)
    );
}

#[test]
fn cancellation_closes_ownership_and_waits_for_group_cleanup() {
    let cancellation = CancellationToken::default();
    let other = cancellation.clone();
    let handle = std::thread::spawn(move || {
        supervisor()
            .run_with_cancellation(request(&["/bin/sleep", "30"]), None, &other)
            .unwrap()
    });
    std::thread::sleep(Duration::from_millis(100));
    cancellation.cancel();
    let result = handle.join().unwrap();
    assert_eq!(result.execution_error, Some(ExecutionError::Cancelled));
    assert_eq!(result.returncode, -9);
    assert!(!result.timed_out);
}

#[test]
fn cancellation_after_consumed_payload_observes_owner_pipe_closure() {
    use std::sync::mpsc;

    struct FixtureDirectory(PathBuf);
    impl Drop for FixtureDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let directory = FixtureDirectory(std::env::temp_dir().join(format!(
        "eal-process-consumed-input-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )));
    std::fs::create_dir(&directory.0).unwrap();
    let ready = directory.0.join("ready");
    let mut command = request(&[
        "/bin/sh",
        "-c",
        "IFS= read -r payload; [ \"$payload\" = 'complete request' ] || exit 7; printf '%s' \"$$\" > \"$1\"; exec /bin/sleep 30",
        "eal-consumed-input-fixture",
        ready.to_str().unwrap(),
    ]);
    command.input = b"complete request\n".to_vec();
    command.timeout = Duration::from_secs(10);
    let cancellation = CancellationToken::default();
    let other = cancellation.clone();
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = supervisor().run_with_cancellation(command, None, &other);
        sender.send(result).unwrap();
    });
    let startup = Instant::now();
    let collector = loop {
        if let Ok(text) = std::fs::read_to_string(&ready)
            && let Ok(pid) = text.parse::<i32>()
        {
            break rustix::process::Pid::from_raw(pid).unwrap();
        }
        if startup.elapsed() > Duration::from_secs(2) {
            cancellation.cancel();
            worker.join().unwrap();
            panic!("collector did not consume the complete request before readiness");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // Readiness follows the complete read, so the supervisor has no payload
    // left to deliver. Cancellation must still observe control-pipe EOF.
    cancellation.cancel();
    let completed = receiver.recv_timeout(Duration::from_secs(2));
    if !matches!(&completed, Ok(Ok(result)) if result.execution_error == Some(ExecutionError::Cancelled))
    {
        // Clean up the real owned group before reporting a failed regression.
        let _ = rustix::process::kill_process_group(collector, rustix::process::Signal::KILL);
    }
    worker.join().unwrap();
    let result = completed
        .expect("cancellation waited for the ten-second execution deadline")
        .expect("cancellation returned before acknowledging group cleanup");
    assert_eq!(result.execution_error, Some(ExecutionError::Cancelled));
    assert_eq!(result.returncode, -9);
    assert!(!result.timed_out);
}

#[test]
fn control_hangup_interest_ignores_pending_payload_until_owner_closes() {
    let (reader, writer) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC).unwrap();
    rustix::io::write(&writer, b"undelivered payload").unwrap();
    let mut polls = [rustix::event::PollFd::new(
        &reader,
        rustix::event::PollFlags::HUP,
    )];
    let timeout = rustix::time::Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    // Data readiness must not cause repeated wakeups while collector input is
    // blocked, but closing the owner must wake the same interest immediately.
    assert_eq!(rustix::event::poll(&mut polls, Some(&timeout)).unwrap(), 0);
    assert!(polls[0].revents().is_empty());
    drop(writer);
    assert_eq!(rustix::event::poll(&mut polls, Some(&timeout)).unwrap(), 1);
    assert!(polls[0].revents().contains(rustix::event::PollFlags::HUP));
}

#[test]
fn startup_and_execution_deadlines_are_distinct_and_input_waits_for_ready() {
    let mut command = request(&[
        "/bin/sh",
        "-c",
        "sleep 0.15; printf 'READY\\n'; read line; printf '%s' \"$line\"; sleep 30",
    ]);
    command.timeout = Duration::from_millis(100);
    command.input = b"request\n".to_vec();
    let handshake = StartupHandshake {
        expected_first_stdout_line: b"READY\n".to_vec(),
        startup_timeout: Duration::from_millis(600),
    };
    let start = Instant::now();
    let result = supervisor()
        .run_framed(command, None, handshake.clone())
        .unwrap();
    assert!(start.elapsed() < Duration::from_millis(600));
    assert!(result.startup_completed);
    assert_eq!(result.execution_error, Some(ExecutionError::Timeout));
    assert_eq!(result.stdout, b"READY\nrequest");
    let mut command = request(&["/bin/sh", "-c", "sleep 30"]);
    command.timeout = Duration::from_secs(1);
    let handshake = StartupHandshake {
        startup_timeout: Duration::from_millis(100),
        ..handshake
    };
    let result = supervisor().run_framed(command, None, handshake).unwrap();
    assert!(!result.startup_completed);
    assert_eq!(result.execution_error, Some(ExecutionError::Timeout));
}

#[test]
fn startup_line_is_exact_even_when_the_collector_exits_immediately() {
    let handshake = StartupHandshake {
        expected_first_stdout_line: b"READY\n".to_vec(),
        startup_timeout: Duration::from_secs(1),
    };
    let result = supervisor()
        .run_framed(
            request(&["/bin/sh", "-c", "printf 'READY\\n';exit 0"]),
            None,
            handshake.clone(),
        )
        .unwrap();
    assert!(result.startup_completed);
    assert_eq!(result.execution_error, None);
    assert_eq!(result.returncode, 0);
    let mut command = request(&["/bin/sh", "-c", "printf 'BAD';cat"]);
    command.input = b"secret request\n".to_vec();
    let result = supervisor().run_framed(command, None, handshake).unwrap();
    assert!(!result.startup_completed);
    assert!(matches!(
        result.execution_error,
        Some(ExecutionError::Startup(_))
    ));
    assert_eq!(result.stdout, b"BAD");
}

#[test]
fn supervisor_identity_is_observed_by_the_owner_and_reaped_before_return() {
    let result = supervisor()
        .run(request(&["/bin/sh", "-c", "printf '%s' \"$PPID\""]), None)
        .unwrap();
    let observed = std::str::from_utf8(&result.stdout)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    assert_eq!(result.supervisor_pid, Some(observed));
    assert_ne!(observed, std::process::id());
    assert_eq!(
        rustix::process::test_kill_process(
            rustix::process::Pid::from_raw(observed as i32).unwrap()
        ),
        Err(rustix::io::Errno::SRCH)
    );
}

#[test]
fn missing_or_incompatible_companion_never_executes_the_requested_command() {
    let directory =
        std::env::temp_dir().join(format!("eal-process-no-fallback-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let marker = directory.join("executed");
    let command = request(&["/bin/sh", "-c", &format!("touch '{}'", marker.display())]);
    assert!(
        Supervisor::new(directory.join("absent"))
            .run(command.clone(), None)
            .is_err()
    );
    assert!(
        Supervisor::new("/bin/true".into())
            .run(command, None)
            .is_err()
    );
    assert!(!marker.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn retained_capability_survives_its_original_handle_until_the_tree_stops() {
    use std::{
        fs::File,
        process::Stdio,
        sync::{Mutex, mpsc},
    };
    struct Lease {
        file: Mutex<Option<File>>,
        ready: mpsc::Sender<()>,
    }
    impl eal_process::Retention for Lease {
        fn duplicate_for_supervisor(&self) -> std::io::Result<Stdio> {
            let original = self.file.lock().unwrap().take().unwrap();
            let copy = original.try_clone()?;
            drop(original);
            self.ready.send(()).unwrap();
            Ok(Stdio::from(copy))
        }
    }
    let (reader, writer) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC).unwrap();
    let flags = rustix::fs::fcntl_getfl(&reader).unwrap();
    rustix::fs::fcntl_setfl(&reader, flags | rustix::fs::OFlags::NONBLOCK).unwrap();
    let (ready, started) = mpsc::channel();
    let lease = Lease {
        file: Mutex::new(Some(File::from(writer))),
        ready,
    };
    let thread = std::thread::spawn(move || {
        supervisor()
            .run(
                request(&["/bin/sh", "-c", "sleep 0.2 & wait"]),
                Some(&lease),
            )
            .unwrap()
    });
    started.recv_timeout(Duration::from_secs(1)).unwrap();
    let mut byte = [0u8; 1];
    assert_eq!(
        rustix::io::read(&reader, &mut byte),
        Err(rustix::io::Errno::AGAIN)
    );
    let result = thread.join().unwrap();
    assert_eq!(result.returncode, 0);
    assert_eq!(rustix::io::read(&reader, &mut byte).unwrap(), 0);
}

#[test]
fn independent_stream_limits_stop_each_stream_before_capture_exceeds_its_budget() {
    use eal_process::StreamLimits;
    let limits = StreamLimits {
        max_stdout_bytes: 8,
        max_stderr_bytes: 4,
    };
    for (script, expected_stdout, expected_stderr, failed) in [
        ("printf 12345678;printf 1234 >&2", 8, 4, false),
        ("printf 123456789", 8, 0, true),
        ("printf 12345 >&2", 0, 4, true),
    ] {
        let mut command = request(&["/bin/sh", "-c", script]);
        command.max_output_bytes = 12;
        let result = supervisor()
            .run_with_stream_limits(command, None, limits)
            .unwrap();
        assert_eq!(result.stdout.len(), expected_stdout);
        assert_eq!(result.stderr.len(), expected_stderr);
        assert_eq!(
            result.execution_error,
            failed.then_some(ExecutionError::OutputLimit)
        );
    }
    let mut command = request(&["/bin/sh", "-c", "printf x >&2"]);
    command.max_output_bytes = 8;
    let result = supervisor()
        .run_with_stream_limits(
            command,
            None,
            StreamLimits {
                max_stdout_bytes: 8,
                max_stderr_bytes: 0,
            },
        )
        .unwrap();
    assert!(result.stderr.is_empty());
    assert_eq!(result.execution_error, Some(ExecutionError::OutputLimit));
}

#[test]
fn independent_limits_reject_invalid_allowances_before_spawning() {
    use eal_process::StreamLimits;
    let command = request(&["/bin/sleep", "30"]);
    let result = supervisor().run_with_stream_limits(
        command,
        None,
        StreamLimits {
            max_stdout_bytes: usize::MAX,
            max_stderr_bytes: 1,
        },
    );
    assert!(matches!(result, Err(eal_process::ProcessError::Invalid(_))));
}
