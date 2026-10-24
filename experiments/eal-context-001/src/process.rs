//! Bounded native process execution for the isolated diagnostics.
use std::{
    ffi::OsString,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
const MAX_OUTPUT: usize = 8 * 1024 * 1024;
const DEADLINE: Duration = Duration::from_secs(240);
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        terminate(&mut self.0);
        let _ = self.0.wait();
    }
}
fn terminate(child: &mut Child) {
    #[cfg(unix)]
    if let Some(pid) = rustix::process::Pid::from_raw(child.id() as i32) {
        let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
    }
    let _ = child.kill();
}
fn drain(mut source: impl Read, used: Arc<AtomicUsize>) -> Result<Vec<u8>, String> {
    let mut retained = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = source.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            return Ok(retained);
        }
        let before = used.fetch_add(read, Ordering::Relaxed);
        let accepted = MAX_OUTPUT.saturating_sub(before).min(read);
        retained.extend_from_slice(&buffer[..accepted]);
    }
}
pub struct Output {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
pub fn execute(
    program: &Path,
    args: &[OsString],
    input: Option<&[u8]>,
    cwd: Option<&Path>,
    empty_path: bool,
    expected: i32,
) -> Result<Output, String> {
    if input.is_some_and(|bytes| bytes.len() > 1024 * 1024) {
        return Err("native request exceeds 1 MiB".into());
    }
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    if empty_path {
        command.env("PATH", "");
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = OwnedChild(
        command
            .spawn()
            .map_err(|e| format!("Cannot launch {}: {e}", program.display()))?,
    );
    let used = Arc::new(AtomicUsize::new(0));
    let out = child.0.stdout.take().ok_or("stdout pipe absent")?;
    let err = child.0.stderr.take().ok_or("stderr pipe absent")?;
    let out_used = Arc::clone(&used);
    let err_used = Arc::clone(&used);
    let stdout = thread::spawn(move || drain(out, out_used));
    let stderr = thread::spawn(move || drain(err, err_used));
    let mut stdin = child.0.stdin.take().ok_or("stdin pipe absent")?;
    let bytes = input.unwrap_or_default().to_vec();
    let writer = thread::spawn(move || stdin.write_all(&bytes));
    let started = Instant::now();
    let mut failure = None;
    let status = loop {
        if used.load(Ordering::Relaxed) > MAX_OUTPUT {
            failure = Some("native process exceeds combined 8 MiB output limit");
            terminate(&mut child.0);
            break child.0.wait().map_err(|e| e.to_string())?;
        }
        if started.elapsed() >= DEADLINE {
            failure = Some("native process exceeds 240-second deadline");
            terminate(&mut child.0);
            break child.0.wait().map_err(|e| e.to_string())?;
        }
        if let Some(status) = child.0.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        thread::sleep(Duration::from_millis(5));
    };
    // Pipe-owning descendants belong to this invocation and are cleaned before joining readers.
    terminate(&mut child.0);
    let stdout = stdout.join().map_err(|_| "stdout reader panicked")??;
    let stderr = stderr.join().map_err(|_| "stderr reader panicked")??;
    let input_result = writer.join().map_err(|_| "stdin writer panicked")?;
    if let Some(failure) = failure {
        return Err(failure.into());
    }
    if used.load(Ordering::Relaxed) > MAX_OUTPUT {
        return Err("native process exceeds combined 8 MiB output limit".into());
    }
    if status.code() != Some(expected) {
        return Err(format!(
            "{} returned {:?}; expected {expected}; diagnostic {}",
            program.display(),
            status.code(),
            String::from_utf8_lossy(&stderr[..stderr.len().min(1500)])
        ));
    }
    input_result.map_err(|e| format!("native process input failed: {e}"))?;
    Ok(Output { stdout, stderr })
}
pub fn executable(name: &str) -> Result<PathBuf, String> {
    let path = Path::new(name);
    if path.components().count() > 1 {
        if !path.is_file() {
            return Err(format!(
                "Selected executable does not exist: {}",
                path.display()
            ));
        }
        return std::path::absolute(path).map_err(|e| e.to_string());
    }
    for directory in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let path = directory.join(name);
        if path.is_file() {
            // Preserve the selected basename for multicall launchers such as rustup's cargo.
            return std::path::absolute(path).map_err(|e| e.to_string());
        }
    }
    Err(format!("Cannot find explicitly selected executable {name}"))
}
