//! Immutable operation catalogue shared by SDK dispatch and the text host.
use eal_runtime::{
    catalogue::{FindOptions, WorkspaceKnowledgeCatalogue},
    packets::PacketLimits,
    registered_assessment::{RegisteredAssessmentHost, RegisteredAssessmentOptions},
    service::{FormalOptions, ReasoningService},
};
use rmcp::model::Tool;
use serde_json::{Value, json};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone, Copy, Debug)]
pub struct OperationFields {
    pub required: &'static [&'static str],
    pub optional: &'static [&'static str],
}
pub const OPERATION_FIELDS: &[(&str, OperationFields)] = &[
    (
        "describe",
        OperationFields {
            required: &[],
            optional: &[],
        },
    ),
    (
        "format",
        OperationFields {
            required: &["source"],
            optional: &[],
        },
    ),
    (
        "validate",
        OperationFields {
            required: &["source"],
            optional: &[],
        },
    ),
    (
        "plan",
        OperationFields {
            required: &["source", "claim"],
            optional: &[],
        },
    ),
    (
        "collect",
        OperationFields {
            required: &["source", "context"],
            optional: &["evidence_ids"],
        },
    ),
    (
        "collect_claim",
        OperationFields {
            required: &["source", "context", "claim"],
            optional: &[],
        },
    ),
    (
        "reason",
        OperationFields {
            required: &["source", "context"],
            optional: &["collection_id", "now"],
        },
    ),
    (
        "compile_aspic",
        OperationFields {
            required: &["source", "context", "collection_id", "goal"],
            optional: &["now", "semantics", "query_mode", "preference"],
        },
    ),
    (
        "explain",
        OperationFields {
            required: &["assessment_id"],
            optional: &["claim"],
        },
    ),
    (
        "packet",
        OperationFields {
            required: &["assessment_id"],
            optional: &["claim", "packet_limits"],
        },
    ),
    (
        "sources",
        OperationFields {
            required: &[],
            optional: &["limit", "offset"],
        },
    ),
    (
        "find_claims",
        OperationFields {
            required: &[],
            optional: &["query", "claim", "limit"],
        },
    ),
    (
        "assess_known",
        OperationFields {
            required: &["entry_id", "claim"],
            optional: &["packet_limits"],
        },
    ),
    (
        "get_known_packet",
        OperationFields {
            required: &["entry_id", "assessment_id", "claim"],
            optional: &["packet_limits"],
        },
    ),
    (
        "grounded",
        OperationFields {
            required: &["arguments", "attacks"],
            optional: &[],
        },
    ),
];
pub fn operation_fields(name: &str) -> Option<OperationFields> {
    let name = name.strip_prefix("eal_").unwrap_or(name);
    OPERATION_FIELDS
        .iter()
        .find_map(|(key, fields)| (*key == name).then_some(*fields))
}
pub fn is_acquisition_operation(name: &str) -> bool {
    matches!(
        name.strip_prefix("eal_").unwrap_or(name),
        "collect" | "collect_claim" | "assess_known"
    )
}

pub(crate) struct Operations {
    service: Arc<ReasoningService>,
    selected: Vec<String>,
    catalogue: Option<Arc<WorkspaceKnowledgeCatalogue>>,
    host: Option<RegisteredAssessmentHost>,
    pub tools: Vec<Tool>,
}
impl Operations {
    pub fn new(
        service: Arc<ReasoningService>,
        selected: Vec<String>,
        exposure: Option<&str>,
    ) -> Result<Self, String> {
        if selected.len() > 128
            || selected.iter().any(String::is_empty)
            || selected.iter().collect::<BTreeSet<_>>().len() != selected.len()
        {
            return Err("Known entries must contain up to 128 unique registered IDs".into());
        }
        let exposure =
            exposure
                .filter(|value| !value.is_empty())
                .unwrap_or(if selected.is_empty() {
                    "operator"
                } else {
                    "registered"
                });
        match exposure {
            "registered" if selected.is_empty() => {
                return Err("Registered exposure requires at least one known entry".into());
            }
            "operator" if !selected.is_empty() => {
                return Err("Operator exposure conflicts with known entries".into());
            }
            "registered" | "operator" => {}
            _ => return Err("exposure must be operator or registered".into()),
        }
        let catalogue = if selected.is_empty() {
            None
        } else {
            Some(Arc::new(WorkspaceKnowledgeCatalogue::new(Arc::clone(
                &service,
            ))?))
        };
        if let Some(catalogue) = &catalogue {
            for identifier in &selected {
                catalogue.get(identifier, false)?;
            }
        }
        let host = catalogue
            .as_ref()
            .map(|catalogue| {
                RegisteredAssessmentHost::new(Arc::clone(&service), Arc::clone(catalogue))
            })
            .transpose()?;
        let fixture: Value = serde_json::from_str(include_str!("operations/tools.json"))
            .map_err(|error| error.to_string())?;
        let tools = serde_json::from_value(fixture[format!("{exposure}_tools")].clone())
            .map_err(|error| error.to_string())?;
        Ok(Self {
            service,
            selected,
            catalogue,
            host,
            tools,
        })
    }

    /// Caller has already validated and acquired its admission permit. A complete
    /// acquiring operation retains one lease through reuse and persistence.
    pub fn dispatch(&self, name: &str, arguments: &Value) -> Result<Value, String> {
        if !self.tools.iter().any(|tool| tool.name == name) {
            return Err(format!(
                "Unknown tool: '{}'",
                name.replace('\\', "\\\\").replace('\'', "\\'")
            ));
        }
        // Projection admission precedes leases, catalogue lookup and acquisition.
        let packet_limits = requested_packet_limits(arguments)?;
        let lease = is_acquisition_operation(name)
            .then(|| self.service.acquisition_lease())
            .transpose()?;
        let string = |key| required_string(arguments, key);
        let optional = |key| optional_string(arguments, key);
        match name {
            "eal_describe" => self.service.describe(),
            "eal_format" => self.service.format(string("source")?),
            "eal_validate" => Ok(self.service.validate(string("source")?)),
            "eal_plan" => self.service.plan(string("source")?, string("claim")?),
            "eal_collect" => {
                let ids = match arguments
                    .get("evidence_ids")
                    .filter(|value| !value.is_null())
                {
                    None => None,
                    Some(value) => Some(
                        value
                            .as_array()
                            .ok_or("evidence_ids must be an array")?
                            .iter()
                            .map(|value| {
                                value
                                    .as_str()
                                    .map(str::to_owned)
                                    .ok_or("evidence_ids must contain strings")
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                };
                self.service.collect_with_lease(
                    string("source")?,
                    &arguments["context"],
                    ids.as_deref(),
                    lease.ok_or("Missing acquisition lease")?,
                )
            }
            "eal_collect_claim" => self.service.collect_claim_with_lease(
                string("source")?,
                &arguments["context"],
                string("claim")?,
                lease.ok_or("Missing acquisition lease")?,
            ),
            "eal_reason" => self.service.reason(
                string("source")?,
                &arguments["context"],
                optional("collection_id")?,
                optional("now")?,
            ),
            "eal_compile_aspic" => self.service.compile_aspic(
                string("source")?,
                &arguments["context"],
                string("collection_id")?,
                FormalOptions {
                    goal: string("goal")?,
                    now: optional("now")?,
                    semantics: optional("semantics")?.unwrap_or("grounded"),
                    query_mode: optional("query_mode")?.unwrap_or("sceptical"),
                    preference: optional("preference")?,
                },
            ),
            "eal_explain" => self
                .service
                .explain(string("assessment_id")?, optional("claim")?),
            "eal_packet" => self.service.packet_with_limits(
                string("assessment_id")?,
                optional("claim")?,
                packet_limits,
            ),
            "eal_grounded" => eal_core::dialectic::solve_grounded(
                &arguments["arguments"],
                &arguments["attacks"],
                self.service.limits(),
            ),
            "eal_sources" => {
                let limit = bounded_integer(arguments, "limit", 50, 1, 100)?;
                let offset = bounded_integer(arguments, "offset", 0, 0, 10_000)?;
                let catalogue = self
                    .catalogue
                    .as_ref()
                    .ok_or("Missing registered catalogue")?;
                let mut entries = self
                    .selected
                    .iter()
                    .map(|identifier| catalogue.get(identifier, false))
                    .collect::<Result<Vec<_>, _>>()?;
                entries.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
                Ok(
                    json!({"sources": entries.iter().skip(offset).take(limit).map(source_summary).collect::<Vec<_>>(), "limit": limit, "offset": offset}),
                )
            }
            "eal_find_claims" => {
                let limit = bounded_integer(arguments, "limit", 50, 1, 100)?;
                let catalogue = self
                    .catalogue
                    .as_ref()
                    .ok_or("Missing registered catalogue")?;
                let mut matches = Vec::new();
                for identifier in &self.selected {
                    let entry = catalogue.get(identifier, false)?;
                    matches.extend(catalogue.find(FindOptions {
                        query: optional("query")?,
                        claim: optional("claim")?,
                        path: entry["path"].as_str(),
                        limit: 1,
                        ..FindOptions::default()
                    })?);
                }
                matches.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
                Ok(
                    json!({"matches": matches.iter().take(limit).map(source_summary).collect::<Vec<_>>()}),
                )
            }
            "eal_assess_known" => {
                let identifier = string("entry_id")?;
                if !self.selected.iter().any(|selected| selected == identifier) {
                    return Err("Registered source is not exposed by this server".into());
                }
                self.host
                    .as_ref()
                    .ok_or("Missing registered assessment host")?
                    .assess_with_lease(
                        identifier,
                        string("claim")?,
                        RegisteredAssessmentOptions {
                            packet_limits: Some(&packet_limits),
                            ..Default::default()
                        },
                        lease.ok_or("Missing acquisition lease")?,
                    )
            }
            "eal_get_known_packet" => {
                let identifier = string("entry_id")?;
                if !self.selected.iter().any(|selected| selected == identifier) {
                    return Err("Registered source is not exposed by this server".into());
                }
                self.host
                    .as_ref()
                    .ok_or("Missing registered assessment host")?
                    .retrieve_packet(
                        identifier,
                        string("assessment_id")?,
                        string("claim")?,
                        packet_limits,
                    )
            }
            _ => Err(format!("Unknown tool: '{name}'")),
        }
    }
}
fn required_string<'a>(arguments: &'a Value, key: &str) -> Result<&'a str, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))
}
fn optional_string<'a>(arguments: &'a Value, key: &str) -> Result<Option<&'a str>, String> {
    match arguments.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .map(Some)
            .ok_or_else(|| format!("{key} must be a string or null")),
    }
}
fn bounded_integer(
    arguments: &Value,
    key: &str,
    default: usize,
    minimum: usize,
    maximum: usize,
) -> Result<usize, String> {
    let integer = match arguments.get(key) {
        None => Some(default),
        Some(value) => value
            .as_u64()
            .and_then(|number| usize::try_from(number).ok()),
    };
    integer
        .filter(|number| (minimum..=maximum).contains(number))
        .ok_or_else(|| format!("{key} must be an integer from {minimum} through {maximum}"))
}
fn source_summary(entry: &Value) -> Value {
    let mut result = serde_json::Map::new();
    for key in [
        "entry_id",
        "path",
        "label",
        "source_digest",
        "claims",
        "claim_metadata",
    ] {
        result.insert(key.into(), entry[key].clone());
    }
    Value::Object(result)
}

/// MCP exposes byte and statement budgets only. Dependency, output and traversal
/// ceilings retain the packet contract. The budget bounds packet JSON, not the
/// surrounding registered result or MCP envelope.
pub(crate) fn requested_packet_limits(arguments: &Value) -> Result<PacketLimits, String> {
    let Some(value) = arguments.get("packet_limits") else {
        return Ok(PacketLimits::default());
    };
    let object = value.as_object().ok_or("packet_limits must be an object")?;
    if object
        .keys()
        .any(|key| !["max_bytes", "max_statement_bytes"].contains(&key.as_str()))
    {
        return Err("packet_limits accepts only max_bytes and max_statement_bytes".into());
    }
    for (key, value) in object {
        if value
            .as_number()
            .is_some_and(|number| number.as_str().contains(['.', 'e', 'E']))
        {
            return Err(format!(
                "Tool input requires a strict integer for packet_limits.{key}"
            ));
        }
    }
    let defaults = PacketLimits::default();
    let limits = PacketLimits {
        max_bytes: bounded_integer(value, "max_bytes", defaults.max_bytes, 4096, 65536)?,
        max_statement_bytes: bounded_integer(
            value,
            "max_statement_bytes",
            defaults.max_statement_bytes,
            1,
            4096,
        )?,
        ..defaults
    };
    limits.validate()?;
    Ok(limits)
}
