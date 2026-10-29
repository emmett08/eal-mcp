#![cfg(unix)]
//! Cancellation still owns a live duplex collector when its stdin is backpressured.
use eal_process::{Supervisor, duplex::DuplexRequest};
use rustix::{
    event::{PollFd, PollFlags},
    process::{Pid, Signal},
};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    net::Shutdown,
    os::unix::{
        fs::DirBuilderExt,
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};

const FIXTURE: &str = "EAL_DUPLEX_BACKPRESSURE_FIXTURE";
const REPORT: &str = "EAL_BACKPRESSURE:";

fn poll_readable(fd: &impl std::os::fd::AsFd) -> io::Result<()> {
    let mut polls = [PollFd::new(fd, PollFlags::IN)];
    rustix::event::poll(
        &mut polls,
        Some(&rustix::time::Timespec {
            tv_sec: 0,
            tv_nsec: 20_000_000,
        }),
    )?;
    Ok(())
}

fn fixture(path: &std::path::Path) {
    let mut control = UnixStream::connect(path).unwrap();
    control
        .write_all(&std::process::id().to_le_bytes())
        .unwrap();
    let input = io::stdin();
    let mut output = io::stdout();
    loop {
        let mut probe = [0; 8];
        if control.read_exact(&mut probe).is_err() {
            return;
        }
        // Observe the pipe without consuming any of its bytes. Replies pass
        // through the supervised stdout path, not this private control socket.
        let queued = rustix::io::ioctl_fionread(&input).unwrap();
        writeln!(output, "{REPORT}{}:{queued}", u64::from_le_bytes(probe)).unwrap();
        output.flush().unwrap();
    }
}

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        // Darwin's sockaddr_un path is short; do not use its long TMPDIR here.
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = PathBuf::from(format!("/tmp/eal-bp-{}-{nonce:x}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct OwnedGroup(Option<Pid>);
impl Drop for OwnedGroup {
    fn drop(&mut self) {
        if let Some(group) = self.0 {
            // The collector, never the lease-owning supervisor, is ours to stop.
            let _ = rustix::process::kill_process_group(group, Signal::KILL);
        }
    }
}

fn report(reader: &mut BufReader<UnixStream>, probe: u64, started: Instant) -> io::Result<u64> {
    let mut line = String::new();
    loop {
        if started.elapsed() >= Duration::from_secs(2) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "no numbered backpressure report",
            ));
        }
        match reader.read_line(&mut line) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(_) if line.ends_with('\n') => {
                // The libtest fixture can prefix its first uncaptured line.
                if let Some((_, payload)) = line.split_once(REPORT) {
                    let (reported, queued) = payload
                        .trim()
                        .split_once(':')
                        .ok_or(io::ErrorKind::InvalidData)?;
                    if reported.parse::<u64>().ok() != Some(probe) {
                        return Err(io::ErrorKind::InvalidData.into());
                    }
                    return queued
                        .parse()
                        .map_err(|_| io::ErrorKind::InvalidData.into());
                }
                line.clear();
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                poll_readable(reader.get_ref())?
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
}

fn establish_backpressure(
    writer: &mut UnixStream,
    reader: &mut BufReader<UnixStream>,
    control: &mut UnixStream,
) -> io::Result<(usize, u64, u64)> {
    let started = Instant::now();
    let mut written = 0;
    let mut previous = 0;
    let mut unchanged = 0;
    for probe in 1u64.. {
        if started.elapsed() >= Duration::from_secs(2) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "could not witness child-input backpressure",
            ));
        }
        loop {
            match writer.write(&[b'x'; 65536]) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(count) => {
                    written += count;
                    if written >= 4 * 1024 * 1024 {
                        return Err(io::Error::other(
                            "fixture exceeded its bounded input before backpressure",
                        ));
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error),
            }
        }
        control.write_all(&probe.to_le_bytes())?;
        let queued = report(reader, probe, started)?;
        if queued > 0 && queued == previous {
            unchanged += 1;
        } else {
            unchanged = 0;
        }
        // Each report crosses the supervised stdout path after filling input to
        // WouldBlock. Repeated unchanged kernel counts establish stalled stdin
        // delivery while the forwarder is actively handling those reports.
        if unchanged == 3 {
            return Ok((written, probe, queued));
        }
        previous = queued;
    }
    unreachable!()
}

#[test]
fn duplex_cancellation_while_child_input_is_backpressured() {
    if let Some(path) = std::env::var_os(FIXTURE) {
        fixture(std::path::Path::new(&path));
        return;
    }
    let directory = Directory::new();
    let socket = directory.0.join("s");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let session_started = Instant::now();
    let mut process = Supervisor::new(env!("CARGO_BIN_EXE_eal-process-supervisor").into())
        .duplex(
            DuplexRequest {
                argv: vec![
                    std::env::current_exe().unwrap().to_str().unwrap().into(),
                    "--exact".into(),
                    "duplex_cancellation_while_child_input_is_backpressured".into(),
                    "--nocapture".into(),
                ],
                environment: BTreeMap::from([(FIXTURE.into(), socket.to_str().unwrap().into())]),
                cwd: directory.0.clone(),
                timeout: Duration::from_secs(10),
                max_input_bytes: 8 * 1024 * 1024,
                max_output_bytes: 65536,
                max_stderr_bytes: 4096,
            },
            None,
        )
        .unwrap();
    let startup = Instant::now();
    let mut control = loop {
        match listener.accept() {
            Ok((control, _)) => break control,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                assert!(
                    startup.elapsed() < Duration::from_secs(2),
                    "collector did not connect"
                );
                poll_readable(&listener).unwrap();
            }
            Err(error) => panic!("collector connection failed: {error}"),
        }
    };
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    control
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut identity = [0; 4];
    control.read_exact(&mut identity).unwrap();
    let mut group = OwnedGroup(Some(
        Pid::from_raw(i32::try_from(u32::from_le_bytes(identity)).unwrap()).unwrap(),
    ));
    let mut reader = BufReader::new(process.stream.try_clone().unwrap());
    let witnessed = establish_backpressure(&mut process.stream, &mut reader, &mut control);
    let cancellation_was_early = session_started.elapsed() < Duration::from_secs(5);
    let cancelled = Instant::now();
    let closed = process.owner.close();
    let acknowledged_before_deadline = session_started.elapsed() < Duration::from_secs(9);
    let eof = control.read(&mut [0; 1]);
    if closed.is_ok() || matches!(eof, Ok(0)) {
        // An acknowledged group or closed fixture capability needs no PID kill.
        group.0.take();
        if closed.is_err() {
            let _ = process.owner.close();
        }
    } else {
        let _ = rustix::process::kill_process_group(group.0.take().unwrap(), Signal::KILL);
        let _ = process.owner.close();
    }
    if let Ok((written, probes, queued)) = witnessed {
        println!(
            "duplex backpressure: sent={written} bytes, probes={probes}, queued={queued} bytes, cancellation={} ms",
            cancelled.elapsed().as_millis()
        );
    }
    assert!(
        witnessed.is_ok(),
        "backpressure was not established: {witnessed:?}"
    );
    assert!(
        cancellation_was_early && acknowledged_before_deadline,
        "fixture cleanup did not distinguish cancellation from the ten-second execution deadline"
    );
    assert!(
        closed.is_ok(),
        "cancellation did not acknowledge cleanup: {closed:?}, witness={witnessed:?}"
    );
    assert!(
        matches!(eof, Ok(0)),
        "collector retained its execution capability after cancellation: {eof:?}"
    );
}

#[test]
fn hangup_interest_ignores_unread_socket_payload_but_observes_owner_shutdown() {
    let (reader, mut owner) = UnixStream::pair().unwrap();
    owner.write_all(b"pending request").unwrap();
    let timeout = rustix::time::Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    let mut polls = [PollFd::new(&reader, PollFlags::HUP)];
    assert_eq!(rustix::event::poll(&mut polls, Some(&timeout)).unwrap(), 0);
    assert!(polls[0].revents().is_empty());
    owner.shutdown(Shutdown::Both).unwrap();
    assert_eq!(rustix::event::poll(&mut polls, Some(&timeout)).unwrap(), 1);
    assert!(polls[0].revents().contains(PollFlags::HUP));
}
