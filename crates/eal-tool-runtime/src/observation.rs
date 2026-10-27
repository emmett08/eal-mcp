//! Shared strict observation correspondence and original-clock admission.
use crate::Error;
use eal_collector_contracts::{AcquisitionRequest, ObservationEnvelope};
use serde_json::Value;

pub fn validate_observation(
    result_schema: &Value,
    require_observed_at: bool,
    request: &AcquisitionRequest,
    bytes: &[u8],
    max_output_bytes: usize,
) -> Result<ObservationEnvelope, Error> {
    let value = eal_collector_contracts::strict_json_exact(
        bytes,
        eal_collector_contracts::JsonLimits {
            max_bytes: max_output_bytes,
            ..Default::default()
        },
    )
    .map_err(Error::InvalidOutput)?;
    let envelope: ObservationEnvelope = serde_json::from_value(value).map_err(|_| {
        Error::InvalidOutput("collector returned an invalid observation envelope".into())
    })?;
    if envelope.context.as_ref() != Some(&request.context)
        || envelope.request.as_ref() != Some(&request.acquisition())
    {
        return Err(Error::InvalidOutput(
            "collector changed request or context correspondence".into(),
        ));
    }
    if envelope.value.get("collected") != Some(&Value::Bool(true)) {
        return Err(Error::InvalidOutput(
            "collector did not record a successful acquisition".into(),
        ));
    }
    let validator = jsonschema::validator_for(result_schema)
        .map_err(|_| Error::InvalidInput("invalid collector result schema".into()))?;
    if !validator.is_valid(&envelope.value) {
        return Err(Error::InvalidOutput(
            "collector value differs from the frozen result schema".into(),
        ));
    }
    if require_observed_at && envelope.observed_at.is_none() {
        return Err(Error::InvalidOutput(
            "the frozen operation contract requires original observed_at in the acquisition envelope"
                .into(),
        ));
    }
    if envelope
        .observed_at
        .as_deref()
        .is_some_and(|value| !valid_timestamp(value))
    {
        return Err(Error::InvalidOutput(
            "collector observed_at requires a valid UTC RFC3339 timestamp".into(),
        ));
    }
    Ok(envelope)
}

pub fn valid_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes.len() > 64
        || !value.is_ascii()
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes.last() != Some(&b'Z')
    {
        return false;
    }
    let number = |start: usize, end: usize| -> Option<u32> {
        bytes[start..end]
            .iter()
            .all(u8::is_ascii_digit)
            .then(|| value[start..end].parse().ok())
            .flatten()
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        number(0, 4),
        number(5, 7),
        number(8, 10),
        number(11, 13),
        number(14, 16),
        number(17, 19),
    ) else {
        return false;
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    year > 0
        && (1..=days).contains(&day)
        && hour <= 23
        && minute <= 59
        && second <= 59
        && (bytes.len() == 20
            || (bytes[19] == b'.'
                && bytes.len() >= 22
                && bytes[20..bytes.len() - 1].iter().all(u8::is_ascii_digit)))
}
