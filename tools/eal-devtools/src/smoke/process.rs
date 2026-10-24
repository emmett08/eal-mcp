//! Own bounded HTTP server streams and the complete native process group.
use super::{Result, ensure};
use rustix::process::{Pid, Signal};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{
    collections::BTreeMap,
    io::Read,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
const STREAM_LIMIT: usize = 4 * 1024 * 1024;
pub struct Background {
    child: Child,
    readers: Vec<JoinHandle<std::io::Result<Vec<u8>>>>,
    group: Pid,
    stopped: Arc<AtomicBool>,
    watchdog: Option<JoinHandle<()>>,
    finished: bool,
}
impl Background {
    pub fn start(
        executable: &Path,
        args: &[String],
        cwd: &Path,
        environment: &BTreeMap<String, String>,
    ) -> Result<Self> {
        let mut command = Command::new(executable);
        command
            .args(args)
            .current_dir(cwd)
            .env_clear()
            .envs(environment)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn().map_err(|e| e.to_string())?;
        let group = Pid::from_raw(child.id() as i32).ok_or("Invalid native server PID")?;
        let readers = vec![
            drain(child.stdout.take().ok_or("Missing server stdout")?, group),
            drain(child.stderr.take().ok_or("Missing server stderr")?, group),
        ];
        let stopped = Arc::new(AtomicBool::new(false));
        let flag = stopped.clone();
        let watchdog = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(180);
            while !flag.load(Ordering::Acquire) {
                if Instant::now() >= deadline {
                    let _ = rustix::process::kill_process_group(group, Signal::KILL);
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
        });
        Ok(Self {
            child,
            readers,
            group,
            stopped,
            watchdog: Some(watchdog),
            finished: false,
        })
    }
    pub fn alive(&mut self) -> Result<bool> {
        self.child
            .try_wait()
            .map(|s| s.is_none())
            .map_err(|e| e.to_string())
    }
    pub fn finish(mut self) -> Result<()> {
        self.stop(true)
    }
    fn stop(&mut self, check: bool) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        let mut status = self.child.try_wait().map_err(|e| e.to_string())?;
        if status.is_none() {
            let _ = rustix::process::kill_process_group(self.group, Signal::INT);
            let deadline = Instant::now() + Duration::from_secs(5);
            while Instant::now() < deadline {
                status = self.child.try_wait().map_err(|e| e.to_string())?;
                if status.is_some() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
        let _ = rustix::process::kill_process_group(self.group, Signal::KILL);
        let status = match status {
            Some(s) => s,
            None => self.child.wait().map_err(|e| e.to_string())?,
        };
        self.stopped.store(true, Ordering::Release);
        if let Some(watchdog) = self.watchdog.take() {
            let _ = watchdog.join();
        }
        self.finished = true;
        let mut streams = Vec::new();
        for reader in self.readers.drain(..) {
            streams.push(
                reader
                    .join()
                    .map_err(|_| "HTTP stream reader panicked")?
                    .map_err(|e| e.to_string())?,
            );
        }
        if check {
            ensure(
                status.success(),
                format!(
                    "HTTP server exited {status}: {}",
                    streams
                        .get(1)
                        .map(|s| String::from_utf8_lossy(s).into_owned())
                        .unwrap_or_default()
                ),
            )?;
            ensure(
                streams.first().is_some_and(Vec::is_empty),
                "HTTP server wrote unexpected stdout",
            )?;
        }
        Ok(())
    }
}
impl Drop for Background {
    fn drop(&mut self) {
        let _ = self.stop(false);
    }
}
fn drain(
    mut stream: impl Read + Send + 'static,
    group: Pid,
) -> JoinHandle<std::io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut output = Vec::new();
        let mut chunk = [0; 8192];
        loop {
            let n = stream.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            if output.len() + n > STREAM_LIMIT {
                let _ = rustix::process::kill_process_group(group, Signal::KILL);
                return Err(std::io::Error::other("HTTP server stream exceeds 4 MiB"));
            }
            output.extend_from_slice(&chunk[..n]);
        }
        Ok(output)
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn completed_server_stdout_is_rejected_and_process_is_reaped() {
        let environment = BTreeMap::from([("PATH".into(), "".into())]);
        let server = Background::start(
            Path::new("/bin/sh"),
            &["-c".into(), "printf unexpected".into()],
            &std::env::temp_dir(),
            &environment,
        )
        .unwrap();
        std::thread::sleep(Duration::from_millis(20));
        assert!(server.finish().unwrap_err().contains("unexpected stdout"));
    }
    #[test]
    fn dropping_a_running_server_cleans_its_group() {
        let environment = BTreeMap::from([("PATH".into(), "".into())]);
        let server = Background::start(
            Path::new("/bin/sh"),
            &["-c".into(), "trap 'exit 0' INT; while :; do :; done".into()],
            &std::env::temp_dir(),
            &environment,
        )
        .unwrap();
        let pid = server.group;
        std::thread::sleep(Duration::from_millis(20));
        drop(server);
        assert_eq!(
            rustix::process::test_kill_process_group(pid),
            Err(rustix::io::Errno::SRCH)
        );
    }
}
