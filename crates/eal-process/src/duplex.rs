//! A bounded duplex stream whose companion retains process-tree ownership.
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

/// A finite protocol session. Byte budgets include every protocol message.
#[derive(Clone, Debug)]
pub struct DuplexRequest {
    pub argv: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub cwd: PathBuf,
    pub timeout: Duration,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub max_stderr_bytes: usize,
}

#[cfg(unix)]
mod unix {
    use super::*;
    use crate::{
        ProcessError, Retention, StreamLimits, Supervisor,
        process::protocol::{self, Configuration, MAX_CONFIG_BYTES},
    };
    use std::{
        io::Write,
        net::Shutdown,
        os::unix::{net::UnixStream, process::CommandExt},
        process::{Child, Command, Stdio},
        time::Instant,
    };

    /// The SDK owns `stream`; this independent owner closes it before reaping.
    pub struct DuplexProcess {
        pub stream: UnixStream,
        pub owner: DuplexOwner,
    }

    pub struct DuplexOwner {
        control: UnixStream,
        child: Option<Child>,
    }
    impl DuplexOwner {
        pub fn pid(&self) -> Option<u32> {
            self.child.as_ref().map(Child::id)
        }
        /// Request cleanup without killing the lease-owning companion itself.
        pub fn close(&mut self) -> Result<(), ProcessError> {
            let _ = self.control.shutdown(Shutdown::Both);
            let started = Instant::now();
            while let Some(child) = self.child.as_mut() {
                if child.try_wait()?.is_some() {
                    self.child.take();
                    return Ok(());
                }
                if started.elapsed() >= Duration::from_millis(500) {
                    return Err(ProcessError::Protocol(
                        "Duplex supervisor is retaining ownership while cleanup completes".into(),
                    ));
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(())
        }
    }
    impl Drop for DuplexOwner {
        fn drop(&mut self) {
            let _ = self.close();
            if let Some(mut child) = self.child.take() {
                // The lease remains in the companion until Collector::finish
                // has stopped every descendant. A reaper never kills it early.
                let _ = std::thread::Builder::new()
                    .name("eal-duplex-reaper".into())
                    .spawn(move || {
                        let _ = child.wait();
                    });
            }
        }
    }

    impl Supervisor {
        pub fn duplex(
            &self,
            request: DuplexRequest,
            retention: Option<&dyn Retention>,
        ) -> Result<DuplexProcess, ProcessError> {
            if request.argv.is_empty()
                || request.argv[0].is_empty()
                || request.timeout.is_zero()
                || request.max_input_bytes == 0
                || request.max_output_bytes == 0
            {
                return Err(ProcessError::Invalid(
                    "Duplex sessions require argv and positive time/byte limits".into(),
                ));
            }
            let maximum = request
                .max_output_bytes
                .checked_add(request.max_stderr_bytes)
                .ok_or_else(|| ProcessError::Invalid("Duplex byte allowance overflow".into()))?;
            let configuration = Configuration {
                argv: request.argv,
                environment: request.environment,
                input_length: 0,
                execution_timeout: request.timeout,
                max_output_bytes: maximum,
                protocol_version: Some(3),
                stream_limits: Some(StreamLimits {
                    max_stdout_bytes: request.max_output_bytes,
                    max_stderr_bytes: request.max_stderr_bytes,
                }),
                handshake: None,
                duplex_input_limit: Some(request.max_input_bytes),
            };
            let encoded = serde_json::to_vec(&configuration)
                .map_err(|e| ProcessError::Invalid(e.to_string()))?;
            if encoded.len() > MAX_CONFIG_BYTES {
                return Err(ProcessError::Invalid(
                    "Duplex configuration exceeds its byte limit".into(),
                ));
            }
            let deadline = protocol::monotonic_nanos()
                .checked_add(request.timeout.as_nanos())
                .ok_or_else(|| {
                    ProcessError::Invalid("Duplex deadline exceeds clock resources".into())
                })?;
            let (mut stream, child_stream) = UnixStream::pair()?;
            let inherited = retention
                .map(Retention::duplicate_for_supervisor)
                .transpose()?
                .unwrap_or_else(Stdio::null);
            let control = stream.try_clone()?;
            let child = Command::new(self.executable())
                .arg(deadline.to_string())
                .current_dir(request.cwd)
                .env_clear()
                .stdin(Stdio::from(std::os::fd::OwnedFd::from(
                    child_stream.try_clone()?,
                )))
                .stdout(Stdio::from(std::os::fd::OwnedFd::from(child_stream)))
                .stderr(inherited)
                .process_group(0)
                .spawn()?;
            let owner = DuplexOwner {
                control,
                child: Some(child),
            };
            stream.set_write_timeout(Some(request.timeout))?;
            stream.write_all(&(encoded.len() as u64).to_be_bytes())?;
            stream.write_all(&encoded)?;
            stream.set_write_timeout(None)?;
            stream.set_nonblocking(true)?;
            Ok(DuplexProcess { stream, owner })
        }
    }
}

#[cfg(unix)]
pub use unix::{DuplexOwner, DuplexProcess};
