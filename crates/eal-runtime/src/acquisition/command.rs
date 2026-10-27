//! Supervised bounded command acquisition.
use super::{
    AcquisitionError, AcquisitionFailureCategory, AcquisitionResult, ToolBinding, encode_request,
};
use crate::{
    lease::AcquisitionLease,
    process::{CommandRequest, ExecutionError, ProcessError, Supervisor},
};
use eal_core::ExecutionLimits;
use serde_json::{Value, json};
use std::{path::Path, time::Duration};
pub(super) fn acquire(
    binding: &ToolBinding,
    request: &Value,
    workspace: &Path,
    secret: &[u8],
    supervisor: &Supervisor,
    lease: Option<&AcquisitionLease>,
    limits: &ExecutionLimits,
) -> Result<AcquisitionResult, AcquisitionError> {
    let input = encode_request(request, limits)?;
    let environment = binding.effective_environment()?;
    let digest =
        binding.process_environment_digest_with_limits(secret, Some(&environment), limits)?;
    let outcome = supervisor
        .run(
            CommandRequest {
                argv: binding.resolved_argv(workspace)?,
                environment,
                cwd: binding.package_root(workspace).to_owned(),
                input,
                timeout: Duration::from_secs_f64(binding.timeout_seconds),
                max_output_bytes: binding.max_output_bytes,
            },
            lease.map(|lease| lease as &dyn eal_process::Retention),
        )
        .map_err(|error| match error {
            ProcessError::Io(error) => AcquisitionError::from(error),
            ProcessError::Invalid(message) => AcquisitionError::invalid(message),
            error => AcquisitionError {
                category: AcquisitionFailureCategory::RuntimeError,
                message: error.to_string(),
            },
        })?;
    let execution = match outcome.execution_error.as_ref() {
        Some(ExecutionError::Spawn(_)) => None,
        Some(error) => Some(error.label()),
        None if outcome.timed_out => Some("timeout"),
        None => None,
    };
    let mut metadata = json!({"process_environment_digest":digest,"returncode":outcome.returncode,"output_truncated":execution==Some("output_limit")});
    let error = if let Some(error) = execution {
        metadata["execution_error"] = json!(error);
        Some(AcquisitionError::invalid(error))
    } else if outcome.returncode != 0 {
        Some(AcquisitionError::invalid(format!(
            "Tool exited with status {}",
            outcome.returncode
        )))
    } else {
        None
    };
    Ok(AcquisitionResult {
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        metadata,
        error,
    })
}
