#![cfg(unix)]
//! Real process-group execution, not mocked command responses.
use eal_runtime::process::{
    CancellationToken, CommandRequest, ExecutionError, StartupHandshake, Supervisor,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn supervisor() -> Supervisor {
    Supervisor::new(PathBuf::from(env!("CARGO_BIN_EXE_eal-command-supervisor")))
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
