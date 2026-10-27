//! Explicit response mapping and pinned schema validation, independent of transport.
use super::{AcquisitionError, AcquisitionResult, ToolBinding, files};
use eal_collector_contracts::PinnedFile;
use eal_core::{ExecutionLimits, canonical};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

fn schema(
    pin: &PinnedFile,
    binding: &ToolBinding,
    workspace: &Path,
) -> Result<Value, AcquisitionError> {
    let path = if Path::new(&pin.path).is_absolute() {
        files::resolve(Path::new(&pin.path))?
    } else {
        files::bounded_path(binding.package_root(workspace), &pin.path)?
    };
    let bytes = files::read_regular(&path, 1024 * 1024, "Schema must be a regular file")?;
    if format!("{:x}", Sha256::digest(&bytes)) != pin.sha256 {
        return Err("Pinned schema identity differs from operator configuration".into());
    }
    let schema = eal_collector_contracts::strict_json_exact(
        &bytes,
        eal_collector_contracts::JsonLimits {
            max_bytes: 1024 * 1024,
            ..Default::default()
        },
    )
    .map_err(AcquisitionError::invalid)?;
    eal_mcp_client::schema::compile_schema(&schema).map_err(AcquisitionError::invalid)?;
    Ok(schema)
}
pub(super) fn preflight(binding: &ToolBinding, workspace: &Path) -> Result<(), AcquisitionError> {
    if let Some(settings) = &binding.direct {
        for pin in settings
            .input_schema
            .iter()
            .chain(settings.output_schema.iter())
        {
            schema(pin, binding, workspace)?;
        }
    }
    Ok(())
}
pub(super) fn input(
    binding: &ToolBinding,
    request: &Value,
    workspace: &Path,
) -> Result<Value, AcquisitionError> {
    let input = request
        .get("input")
        .filter(|v| v.is_object())
        .ok_or("Direct operations require object input")?
        .clone();
    let settings = binding.direct.as_ref().expect("validated direct settings");
    if let Some(pin) = &settings.input_schema {
        eal_mcp_client::schema::validate_schema(&input, &schema(pin, binding, workspace)?)
            .map_err(AcquisitionError::invalid)?;
    }
    // Validate output schema configuration before the operation has effects.
    if let Some(pin) = &settings.output_schema {
        schema(pin, binding, workspace)?;
    }
    Ok(input)
}
pub(super) fn finish(
    binding: &ToolBinding,
    decoded: &Value,
    workspace: &Path,
    limits: &ExecutionLimits,
    metadata: Value,
) -> Result<AcquisitionResult, AcquisitionError> {
    let settings = binding.direct.as_ref().expect("validated direct settings");
    let value = decoded
        .pointer(&settings.value_pointer)
        .ok_or("Response omitted configured value pointer")?
        .clone();
    if let Some(pin) = &settings.output_schema {
        eal_mcp_client::schema::validate_schema(&value, &schema(pin, binding, workspace)?)
            .map_err(AcquisitionError::invalid)?;
    }
    let mut envelope = json!({"value":value});
    if let Some(pointer) = &settings.observed_at_pointer {
        let observed = decoded
            .pointer(pointer)
            .and_then(Value::as_str)
            .ok_or("Response omitted original observation time")?;
        eal_core::validation::parse_time(observed).map_err(AcquisitionError::invalid)?;
        envelope["observed_at"] = json!(observed);
    }
    let mut bounded = limits.clone();
    bounded.record_bytes = binding.max_output_bytes;
    let stdout =
        canonical::canonical_json_bytes(&envelope, &bounded).map_err(AcquisitionError::invalid)?;
    Ok(AcquisitionResult {
        stdout,
        metadata,
        ..Default::default()
    })
}
