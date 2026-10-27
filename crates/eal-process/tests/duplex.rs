#![cfg(unix)]
//! Live protocol streams retain the same process-tree and lease ownership as commands.
use eal_process::{Retention, Supervisor, duplex::DuplexRequest};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    os::fd::AsFd,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn supervisor() -> Supervisor {
    Supervisor::new(env!("CARGO_BIN_EXE_eal-process-supervisor").into())
}
fn request(script: &str) -> DuplexRequest {
    DuplexRequest {
        argv: vec!["/bin/sh".into(), "-c".into(), script.into()],
        environment: BTreeMap::new(),
        cwd: std::env::current_dir().unwrap(),
        timeout: Duration::from_secs(3),
        max_input_bytes: 65536,
        max_output_bytes: 65536,
        max_stderr_bytes: 1024,
    }
}
fn pid(line: &str) -> rustix::process::Pid {
    rustix::process::Pid::from_raw(line.trim().parse().unwrap()).unwrap()
}
fn gone(pid: rustix::process::Pid) {
    assert_eq!(
        rustix::process::test_kill_process(pid),
        Err(rustix::io::Errno::SRCH)
    );
}
fn blocking(stream: &std::os::unix::net::UnixStream) {
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(4)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(4)))
        .unwrap();
}
#[test]
fn duplex_exchanges_messages_before_exit_and_cancels_the_owned_group() {
    let mut process = supervisor()
        .duplex(request("sleep 30 & printf '%s\\n' $!; cat"), None)
        .unwrap();
    blocking(&process.stream);
    let mut reader = BufReader::new(process.stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let descendant = pid(&line);
    for message in ["first\n", "second\n"] {
        process.stream.write_all(message.as_bytes()).unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line, message);
    }
    process.owner.close().unwrap();
    gone(descendant);
}
#[test]
fn duplex_deadline_stops_descendants_without_client_polling() {
    let mut bounded = request("sleep 30 & printf '%s\\n' $!; wait");
    bounded.timeout = Duration::from_millis(250);
    let mut process = supervisor().duplex(bounded, None).unwrap();
    blocking(&process.stream);
    let mut reader = BufReader::new(process.stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let descendant = pid(&line);
    let started = Instant::now();
    let mut trailing = Vec::new();
    reader.read_to_end(&mut trailing).unwrap();
    assert!(started.elapsed() < Duration::from_secs(2));
    process.owner.close().unwrap();
    gone(descendant);
}
#[test]
fn duplex_transcript_and_stderr_limits_stop_the_tree() {
    for (script, input_limit, output_limit, stderr_limit) in [
        (
            "sleep 30 & printf '%s\\n' $!; read request; printf '123456789012345678901234567890'; wait",
            65536,
            16,
            1024,
        ),
        (
            "sleep 30 & printf '%s\\n' $!; read request; printf '123456789' >&2; wait",
            65536,
            65536,
            8,
        ),
        (
            "sleep 30 & printf '%s\\n' $!; read request; wait",
            4,
            65536,
            1024,
        ),
    ] {
        let mut bounded = request(script);
        bounded.max_input_bytes = input_limit;
        bounded.max_output_bytes = output_limit;
        bounded.max_stderr_bytes = stderr_limit;
        let mut process = supervisor().duplex(bounded, None).unwrap();
        blocking(&process.stream);
        let mut reader = BufReader::new(process.stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let descendant = pid(&line);
        process.stream.write_all(b"request\n").unwrap();
        let started = Instant::now();
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        assert!(started.elapsed() < Duration::from_secs(2));
        process.owner.close().unwrap();
        gone(descendant);
        assert!(output.len() + line.len() <= output_limit);
    }
}

struct Lease(File);
impl Retention for Lease {
    fn duplicate_for_supervisor(&self) -> std::io::Result<Stdio> {
        Ok(Stdio::from(self.0.try_clone()?))
    }
}

#[test]
fn duplex_host_death_retains_lease_until_descendants_stop() {
    const KEY: &str = "EAL_DUPLEX_OWNER_FIXTURE";
    if let Some(marker) = std::env::var_os(KEY) {
        let lease = Lease(File::from(
            std::io::stderr().as_fd().try_clone_to_owned().unwrap(),
        ));
        let process = supervisor()
            .duplex(request("sleep 30 & printf '%s\\n' $!; wait"), Some(&lease))
            .unwrap();
        blocking(&process.stream);
        let mut reader = BufReader::new(process.stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        std::fs::write(marker, line).unwrap();
        // The parent kills this owner. Its Drop implementations cannot run.
        std::thread::sleep(Duration::from_secs(30));
        return;
    }
    let marker = std::env::temp_dir().join(format!("eal-duplex-host-{}.pid", std::process::id()));
    let _ = std::fs::remove_file(&marker);
    let (reader, writer) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC).unwrap();
    let mut owner = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "duplex_host_death_retains_lease_until_descendants_stop",
            "--nocapture",
        ])
        .env(KEY, &marker)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(writer))
        .spawn()
        .unwrap();
    let started = Instant::now();
    while !marker.exists() {
        assert!(started.elapsed() < Duration::from_secs(3));
        assert!(owner.try_wait().unwrap().is_none());
        std::thread::sleep(Duration::from_millis(5));
    }
    let descendant = pid(&std::fs::read_to_string(&marker).unwrap());
    assert!(rustix::process::test_kill_process(descendant).is_ok());
    owner.kill().unwrap();
    owner.wait().unwrap();
    let started = Instant::now();
    loop {
        let mut polls = [rustix::event::PollFd::new(
            &reader,
            rustix::event::PollFlags::IN,
        )];
        rustix::event::poll(
            &mut polls,
            Some(&rustix::time::Timespec {
                tv_sec: 0,
                tv_nsec: 20_000_000,
            }),
        )
        .unwrap();
        if polls[0].revents().contains(rustix::event::PollFlags::HUP) {
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "supervised lease did not close"
        );
    }
    gone(descendant);
    std::fs::remove_file(marker).unwrap();
}
