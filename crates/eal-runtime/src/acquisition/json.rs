use super::{AcquisitionError, AcquisitionFailureCategory, MAX_JSON_DEPTH, MAX_OUTPUT_BYTES};
use eal_core::{ExecutionLimits, canonical};
use serde_json::Value;
pub fn strict_json(text: &str) -> Result<Value, AcquisitionError> {
    strict_json_with_limits(
        text,
        &ExecutionLimits {
            json_depth: MAX_JSON_DEPTH,
            record_bytes: MAX_OUTPUT_BYTES,
            ..ExecutionLimits::default()
        },
    )
}
pub fn strict_json_with_limits(
    text: &str,
    limits: &ExecutionLimits,
) -> Result<Value, AcquisitionError> {
    limits.validate()?;
    if text.len() > limits.record_bytes {
        return Err("JSON input exceeds the record byte budget".into());
    }
    eal_collector_contracts::strict_json_diagnostic(
        text.as_bytes(),
        eal_collector_contracts::JsonLimits {
            max_bytes: limits.record_bytes,
            max_depth: limits.json_depth.min(MAX_JSON_DEPTH),
            max_nodes: limits.json_nodes,
        },
    )
    .map_err(|diagnostic| AcquisitionError {
        category: match diagnostic.code {
            eal_collector_contracts::JsonDiagnosticCode::Syntax => {
                AcquisitionFailureCategory::JsonDecodeError
            }
            eal_collector_contracts::JsonDiagnosticCode::Unicode => {
                AcquisitionFailureCategory::UnicodeDecodeError
            }
            _ => AcquisitionFailureCategory::ValueError,
        },
        message: diagnostic.message,
    })
}

pub fn validate_envelope(
    stdout: &[u8],
    file_import: bool,
    context: &Value,
    acquisition: &Value,
) -> Result<Value, AcquisitionError> {
    validate_envelope_with_limits(
        stdout,
        file_import,
        context,
        acquisition,
        &ExecutionLimits {
            json_depth: MAX_JSON_DEPTH,
            record_bytes: MAX_OUTPUT_BYTES,
            ..ExecutionLimits::default()
        },
    )
}
pub fn validate_envelope_with_limits(
    stdout: &[u8],
    file_import: bool,
    context: &Value,
    acquisition: &Value,
    limits: &ExecutionLimits,
) -> Result<Value, AcquisitionError> {
    let text = std::str::from_utf8(stdout).map_err(|error| AcquisitionError {
        category: AcquisitionFailureCategory::UnicodeDecodeError,
        message: error.to_string(),
    })?;
    let envelope = strict_json_with_limits(text, limits)?;
    let _: eal_collector_contracts::ObservationEnvelope = serde_json::from_value(envelope.clone())
        .map_err(|_| {
            AcquisitionError::invalid(
                "EAL_OBSERVATION_ENVELOPE: invalid observation envelope fields",
            )
        })?;
    let object = envelope
        .as_object()
        .expect("shared envelope admission checked the object");
    if file_import
        && ["observed_at", "context", "request"]
            .iter()
            .any(|k| !object.contains_key(*k))
    {
        return Err("File observations require observed_at and context and request; import must preserve age, scope and acquisition identity".into());
    }
    if let Some(request) = object.get("request")
        && (!request.is_object()
            || canonical::canonical_digest(request, limits)?
                != canonical::canonical_digest(acquisition, limits)?)
    {
        return Err("Observation request differs from the declared acquisition request".into());
    }
    if let Some(supplied) = object.get("context")
        && (!supplied.is_object()
            || canonical::canonical_digest(supplied, limits)?
                != canonical::canonical_digest(context, limits)?)
    {
        return Err("Observation context differs from the requested context".into());
    }
    Ok(envelope)
}
