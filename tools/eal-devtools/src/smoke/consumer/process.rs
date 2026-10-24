//! Execute each native or Node child under the existing bounded supervisor.
use super::input::parse_json;
use super::{MAX_BYTES, Result};
use eal_runtime::process::{CommandRequest, Supervisor};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::Path,
    time::Duration,
};
pub(super) struct Runner {
    pub(super) supervisor: Supervisor,
    pub(super) environment: BTreeMap<String, String>,
}
impl Runner {
    pub(super) fn run(
        &self,
        binary: &Path,
        args: &[String],
        cwd: &Path,
        seconds: u64,
        maximum: usize,
    ) -> Result<Vec<u8>> {
        let argv = std::iter::once(binary.to_string_lossy().into_owned())
            .chain(args.iter().cloned())
            .collect();
        let outcome = self
            .supervisor
            .run(
                CommandRequest {
                    argv,
                    environment: self.environment.clone(),
                    cwd: cwd.to_owned(),
                    input: Vec::new(),
                    timeout: Duration::from_secs(seconds),
                    max_output_bytes: maximum,
                },
                None,
            )
            .map_err(|e| e.to_string())?;
        if outcome.returncode != 0 || outcome.execution_error.is_some() || outcome.timed_out {
            return Err(format!(
                "{} failed with status {} ({:?}): {}",
                binary.display(),
                outcome.returncode,
                outcome.execution_error,
                String::from_utf8_lossy(&outcome.stderr)
            ));
        }
        Ok(outcome.stdout)
    }
    pub(super) fn core(&self, binary: &Path, work: &Path, args: &[String]) -> Result<Value> {
        let args: Vec<_> = [
            "--workspace".to_owned(),
            work.to_string_lossy().into_owned(),
        ]
        .into_iter()
        .chain(args.iter().cloned())
        .collect();
        parse_json(&self.run(binary, &args, work, 30, MAX_BYTES)?, MAX_BYTES)
    }
}
