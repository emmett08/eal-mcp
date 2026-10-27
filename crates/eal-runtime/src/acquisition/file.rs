//! Read retained envelopes without refreshing their measurement clocks.
use super::{AcquisitionError, AcquisitionResult, ToolBinding, files};
use serde_json::json;
use std::path::Path;
pub(super) fn acquire(
    binding: &ToolBinding,
    workspace: &Path,
) -> Result<AcquisitionResult, AcquisitionError> {
    let workspace = binding.package_root(workspace);
    let path = files::bounded_path(
        workspace,
        binding.path.as_deref().expect("validated file path"),
    )?;
    let workspace = workspace.canonicalize()?;
    let relative = path
        .strip_prefix(&workspace)
        .map_err(|_| AcquisitionError::invalid("File path escaped the workspace"))?;
    let relative = relative
        .to_str()
        .ok_or("File path must contain valid Unicode scalar values")?;
    let mut metadata = json!({"file":relative});
    let mut stdout = match files::read_regular(
        &path,
        binding.max_output_bytes,
        "File observations require a regular file",
    ) {
        Ok(stdout) => stdout,
        Err(error) => {
            return Ok(AcquisitionResult {
                metadata,
                error: Some(error),
                ..AcquisitionResult::default()
            });
        }
    };
    let error = if stdout.len() > binding.max_output_bytes {
        stdout.truncate(binding.max_output_bytes);
        metadata["output_truncated"] = json!(true);
        Some(AcquisitionError::invalid("output_limit"))
    } else {
        None
    };
    Ok(AcquisitionResult {
        stdout,
        metadata,
        error,
        ..AcquisitionResult::default()
    })
}
