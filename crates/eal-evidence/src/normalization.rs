//! Normalise explicit MCP response formats without manufacturing measurement age.
use crate::{
    BridgeError, BridgeErrorKind,
    config::{ObservationTime, ResponseFormat, ToolMapping},
};
use chrono::Utc;
use eal_collector_contracts::capability::{
    CapabilityDescriptor, CoverageGuarantee, LogicalStrength,
};
use rmcp::model::CallToolResult;
use serde_json::Value;

pub(crate) struct Normalised {
    pub value: Value,
    pub observed_at: String,
    pub coverage: Vec<CoverageGuarantee>,
    pub bytes: u64,
}
fn error(kind: BridgeErrorKind, message: &str) -> BridgeError {
    BridgeError::new(kind, message)
}
pub(crate) fn normalise(
    result: &CallToolResult,
    mapping: &ToolMapping,
    descriptor: &CapabilityDescriptor,
    scope: &Value,
) -> Result<Normalised, BridgeError> {
    if result.is_error.unwrap_or(false) {
        return Err(error(
            BridgeErrorKind::Acquisition,
            "MCP tool reported acquisition failure",
        ));
    }
    let bytes = serde_json::to_vec(result)
        .map_err(|_| error(BridgeErrorKind::Protocol, "Cannot encode MCP result"))?;
    if bytes.len() > descriptor.limits.output_bytes as usize {
        return Err(error(
            BridgeErrorKind::Budget,
            "MCP result exceeded the qualified output budget",
        ));
    }
    let decoded = match mapping.response_format {
        ResponseFormat::Structured => result.structured_content.clone().ok_or_else(|| {
            error(
                BridgeErrorKind::Protocol,
                "MCP tool omitted configured structured content",
            )
        })?,
        ResponseFormat::JsonText { block } => {
            let blocks = serde_json::to_value(&result.content)
                .map_err(|_| error(BridgeErrorKind::Protocol, "Cannot inspect MCP content"))?;
            let selected = blocks
                .as_array()
                .and_then(|blocks| blocks.get(block))
                .filter(|block| block["type"] == "text")
                .and_then(|block| block["text"].as_str())
                .ok_or_else(|| {
                    error(
                        BridgeErrorKind::Protocol,
                        "MCP tool omitted the configured JSON text block",
                    )
                })?;
            eal_collector_contracts::strict_json_exact(
                selected.as_bytes(),
                eal_collector_contracts::JsonLimits {
                    max_bytes: descriptor.limits.output_bytes as usize,
                    ..Default::default()
                },
            )
            .map_err(|_| {
                error(
                    BridgeErrorKind::Protocol,
                    "MCP text is not exact bounded JSON",
                )
            })?
        }
    };
    normalise_decoded(&decoded, mapping, descriptor, scope, bytes.len() as u64)
}

pub(crate) fn normalise_decoded(
    decoded: &Value,
    mapping: &ToolMapping,
    descriptor: &CapabilityDescriptor,
    scope: &Value,
    bytes: u64,
) -> Result<Normalised, BridgeError> {
    let semantics = descriptor.semantics.as_ref().ok_or_else(|| {
        error(
            BridgeErrorKind::Configuration,
            "MCP capability lacks qualified measurement semantics",
        )
    })?;
    let observed_at = match &mapping.observation_time {
        ObservationTime::Pointer { pointer } => {
            let value = decoded
                .pointer(pointer)
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    error(
                        BridgeErrorKind::Protocol,
                        "MCP measurement omitted its original observation time",
                    )
                })?;
            chrono::DateTime::parse_from_rfc3339(value).map_err(|_| {
                error(
                    BridgeErrorKind::Protocol,
                    "MCP observation time is not RFC3339",
                )
            })?;
            value.to_owned()
        }
        ObservationTime::CollectionTime => {
            if semantics.time.original_observation_time || semantics.time.historical_intervals {
                return Err(error(
                    BridgeErrorKind::Configuration,
                    "Historical or original-age measurements require a response timestamp pointer",
                ));
            }
            Utc::now().to_rfc3339()
        }
    };
    let value = decoded
        .pointer(&mapping.value_pointer)
        .cloned()
        .ok_or_else(|| {
            error(
                BridgeErrorKind::Protocol,
                "MCP result omitted the configured measurement pointer",
            )
        })?;
    eal_mcp_client::schema::validate_schema(&value, &descriptor.output_schema).map_err(|_| {
        error(
            BridgeErrorKind::Protocol,
            "MCP measurement differs from its qualified output schema",
        )
    })?;
    for quantity in &semantics.quantities {
        if value.pointer(&quantity.output_pointer).is_none() {
            return Err(error(
                BridgeErrorKind::Protocol,
                "Measurement omitted a qualified quantity",
            ));
        }
    }
    for dimension in &semantics.identity_dimensions {
        let pointer = mapping.scope_pointers.get(dimension).ok_or_else(|| {
            error(
                BridgeErrorKind::Configuration,
                "Qualified MCP identity dimension lacks a configured response pointer",
            )
        })?;
        let measured = value
            .pointer(pointer)
            .filter(|value| !value.is_null())
            .ok_or_else(|| {
                error(
                    BridgeErrorKind::Protocol,
                    "MCP measurement omitted a qualified identity dimension",
                )
            })?;
        if scope
            .get(dimension)
            .is_some_and(|expected| expected != measured)
        {
            return Err(error(
                BridgeErrorKind::Protocol,
                "MCP measurement identity differs from the requested source scope",
            ));
        }
    }
    let coverage = semantics
        .coverage
        .iter()
        .filter(|guarantee| match guarantee.strength {
            LogicalStrength::CompleteEnumeration => guarantee
                .completeness_pointer
                .as_ref()
                .is_some_and(|pointer| value.pointer(pointer) == Some(&Value::Bool(true))),
            LogicalStrength::StatisticalEstimate => guarantee
                .sample_size_pointer
                .as_ref()
                .and_then(|pointer| value.pointer(pointer))
                .and_then(Value::as_u64)
                .is_some_and(|count| count > 0),
            _ => true,
        })
        .cloned()
        .collect();
    Ok(Normalised {
        value,
        observed_at,
        coverage,
        bytes,
    })
}
