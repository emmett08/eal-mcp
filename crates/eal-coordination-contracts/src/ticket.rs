//! Versioned application-stage and artifact contracts for a retained ticket.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

pub const TICKET_SCHEMA: &str = "eal.ticket/1";
pub const STAGE_REQUEST_SCHEMA: &str = "eal.ticket-stage/1";
pub const STAGE_RESULT_SCHEMA: &str = "eal.ticket-stage-result/1";

/// A usable measurement can refute its predicate while completing an investigation.
/// Acquisition failure, expiry and missing observations remain unresolved outcomes.
pub fn evidence_is_measured(verdict: &Value) -> bool {
    verdict["status"] == "available"
        || (verdict["status"] == "unavailable"
            && verdict["availability_issues"]
                .as_array()
                .is_some_and(|issues| {
                    !issues.is_empty() && issues.iter().all(|issue| issue == "predicate_not_met")
                }))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub kind: String,
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Author,
    Integrate,
    Export,
    Execute,
}
impl Stage {
    pub const ALL: [Self; 4] = [Self::Author, Self::Integrate, Self::Export, Self::Execute];
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Author => "author",
            Self::Integrate => "integrate",
            Self::Export => "export",
            Self::Execute => "execute",
        }
    }
    pub const fn required_artifacts(self) -> &'static [&'static str] {
        match self {
            Self::Author => &["analysis"],
            Self::Integrate => &["analysis_ready", "planning", "connections", "registration"],
            Self::Export => &["source", "tools", "bundle"],
            Self::Execute => &[
                "source_registration",
                "collection",
                "assessment",
                "registered_assessment",
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Completed,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TicketSpec {
    pub schema_version: String,
    pub ticket_id: String,
    pub workspace: PathBuf,
    pub worker_argv: Vec<String>,
    pub environment: Vec<String>,
    pub timeout_seconds: u64,
    pub stage_timeout_seconds: u64,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub context: Value,
    pub inputs: Vec<Artifact>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageRequest {
    pub schema_version: String,
    pub ticket_id: String,
    pub stage: Stage,
    pub workspace: PathBuf,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub context: Value,
    pub previous: Vec<Artifact>,
    pub call_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageResult {
    pub schema_version: String,
    pub ticket_id: String,
    pub stage: Stage,
    pub status: StageStatus,
    pub artifacts: Vec<Artifact>,
    #[serde(
        default,
        deserialize_with = "eal_collector_contracts::deserialize_optional_value_exact"
    )]
    pub outcome: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn measured_negatives_are_complete_but_failed_expired_and_malformed_verdicts_are_not() {
        assert!(evidence_is_measured(&json!({"status":"available"})));
        assert!(evidence_is_measured(
            &json!({"status":"unavailable","availability_issues":["predicate_not_met"]})
        ));
        for value in [
            json!({}),
            json!({"status":"false"}),
            json!({"status":"unavailable"}),
            json!({"status":"unavailable","availability_issues":[]}),
            json!({"status":"unavailable","availability_issues":["expired"]}),
            json!({"status":"unavailable","availability_issues":["acquisition_failed"]}),
            json!({"status":"unavailable","availability_issues":["predicate_not_met","missing_observation"]}),
        ] {
            assert!(!evidence_is_measured(&value));
        }
    }
}
