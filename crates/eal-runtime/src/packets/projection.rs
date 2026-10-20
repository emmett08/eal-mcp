use super::shape::{
    bounded, digest, host_limits, identifier, ids, record_identifier, scalar, state,
};
use super::{AssessmentPacketBuilder, CLAIM_STATES, prefix};
use chrono::{DateTime, Utc};
use eal_core::{canonical::canonical_digest, methods::default_registry, validation::parse_time};
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

impl AssessmentPacketBuilder {
    pub(super) fn argument(&self, item: &Map<String, Value>) -> Result<Value, String> {
        let empty = Map::new();
        let computation = item
            .get("reasoning_result")
            .and_then(Value::as_object)
            .unwrap_or(&empty);
        let method = bounded(computation.get("method"));
        let mut result = json!({"status": state(item.get("status"), CLAIM_STATES)?,
            "conclusion": identifier(item.get("conclusion").unwrap_or(&Value::Null))?,
            "method_result": {"status": state(computation.get("status"), &["supported", "unsupported", "not_run"])?, "method": method}});
        let projected = result["method_result"]
            .as_object_mut()
            .expect("constructed object");
        for key in ["evidence_id", "input_digest", "bound_observation_digest"] {
            let value = computation.get(key).unwrap_or(&Value::Null);
            let selected = if key.ends_with("digest") {
                digest(value)
            } else if value.is_string() {
                Some(json!(identifier(value)?))
            } else {
                None
            };
            if let Some(value) = selected {
                projected.insert(key.into(), value);
            }
        }
        if let Some(details) = computation.get("details").and_then(Value::as_object) {
            let mut outputs = self.outputs(method.as_ref().and_then(Value::as_str), details);
            let safe: &[&str] = match method.as_ref().and_then(Value::as_str) {
                Some("structured/1") => &["authored", "mechanically_proved"],
                Some("deductive/1") => &["valuations", "satisfying_premise_valuations"],
                Some("inductive/1") => &["sample_size", "successes", "confidence"],
                Some("causal/1") => &["sample_size", "treatment_size", "control_size"],
                Some("analogical/1") => &["feature_count", "matched"],
                _ => &[],
            };
            for key in safe {
                if let Some(value) = details.get(*key).filter(|v| scalar(v))
                    && outputs.len() < self.limits.max_outputs
                {
                    outputs.insert((*key).into(), value.clone());
                }
            }
            if !outputs.is_empty() {
                projected.insert("outputs".into(), Value::Object(outputs));
            }
            if let Some(value) = details
                .get("assumptions_verified")
                .filter(|v| v.is_boolean())
            {
                projected.insert("assumptions_verified".into(), value.clone());
            }
        }
        if let Some(predicates) = computation.get("predicates").and_then(Value::as_array) {
            projected.insert(
                "require_holds".into(),
                json!(
                    predicates
                        .iter()
                        .take(16)
                        .filter_map(|v| v.get("holds").filter(|v| v.is_boolean()))
                        .collect::<Vec<_>>()
                ),
            );
            if predicates.len() > 16 {
                projected.insert("require_omitted".into(), json!(predicates.len() - 16));
            }
        }
        if let Some(binding) = computation.get("binding").and_then(Value::as_object) {
            let evidence_id = binding
                .get("evidence_id")
                .filter(|v| v.is_string())
                .map(identifier)
                .transpose()?;
            let mut trace =
                json!({"status": bounded(binding.get("status")), "evidence_id": evidence_id});
            for key in [
                "holds",
                "actual",
                "prose_verified",
                "physical_interpretation_verified",
            ] {
                if let Some(value) = binding.get(key).filter(|v| scalar(v)) {
                    trace[key] = value.clone();
                }
            }
            if let Some(unit) = bounded(binding.get("output_unit")) {
                trace["output_unit"] = unit;
            }
            if let Some(digest) =
                digest(binding.get("input_payload_digest").unwrap_or(&Value::Null))
            {
                trace["input_payload_digest"] = digest;
            }
            projected.insert("binding".into(), trace);
        }
        if let Some(deps) = item.get("dependencies").and_then(Value::as_object) {
            result["premises"] = json!(prefix(ids(deps.get("premises"), false)?, 16));
        }
        Ok(result)
    }

    fn outputs(&self, method: Option<&str>, details: &Map<String, Value>) -> Map<String, Value> {
        let Some(method) = method else {
            return Map::new();
        };
        let contract = self
            .method_registry
            .as_ref()
            .and_then(|r| r.get(method))
            .or_else(|| default_registry().get(method));
        let mut output = Map::new();
        if let Some(contract) = contract {
            for path in contract.outputs.keys() {
                if path.ends_with(".*") {
                    continue;
                }
                let mut value = None;
                for (i, component) in path.split('.').enumerate() {
                    value = if i == 0 {
                        details.get(component)
                    } else {
                        value
                            .and_then(|v: &Value| v.as_object())
                            .and_then(|v| v.get(component))
                    };
                }
                if let Some(value) = value.filter(|v| scalar(v)) {
                    output.insert(path.clone(), value.clone());
                }
                if output.len() == self.limits.max_outputs {
                    break;
                }
            }
        }
        output
    }

    pub(super) fn evidence(&self, item: &Map<String, Value>) -> Result<Value, String> {
        let empty = Vec::new();
        let issues = match item.get("availability_issues") {
            None => &empty,
            Some(v) => v
                .as_array()
                .ok_or("Evidence availability issues must be a list")?,
        };
        let known: BTreeSet<_> = issues
            .iter()
            .filter_map(Value::as_str)
            .filter(|s| {
                [
                    "out_of_scope",
                    "missing_observation",
                    "invalid_observation",
                    "tool_error",
                    "stale_observation",
                    "predicate_not_met",
                ]
                .contains(s)
            })
            .collect();
        let shown: Vec<_> = known
            .into_iter()
            .take(self.limits.max_availability_issues)
            .collect();
        let mut result = json!({"status": state(item.get("status"), &["available", "unavailable"])?,
            "availability_issues": shown, "issues_omitted": issues.len() - shown.len(),
            "observation_id": record_identifier(item.get("run_id").unwrap_or(&Value::Null)), "age_status": "unknown"});
        let failures = match item.get("predicate_failures") {
            None => &empty,
            Some(v) => v.as_array().ok_or("Predicate failures must be a list")?,
        };
        let safe: Vec<_> = failures
            .iter()
            .filter_map(|f| {
                let path = bounded(f.get("path"))?;
                let issue = f.get("issue")?.as_str()?;
                [
                    "missing_field",
                    "incompatible_type",
                    "non_finite",
                    "incomparable",
                    "not_met",
                ]
                .contains(&issue)
                .then(|| json!({"path": path, "issue": issue}))
            })
            .collect();
        let retained = safe.len().min(self.limits.max_availability_issues);
        if !safe.is_empty() {
            result["predicate_failures"] = json!(prefix(safe, self.limits.max_availability_issues));
        }
        result["predicate_failures_omitted"] = json!(failures.len() - retained);
        Ok(result)
    }
}

pub(super) fn objection(item: &Map<String, Value>) -> Result<Value, String> {
    Ok(
        json!({"status": state(item.get("status"), &["active", "undecided", "defeated", "inactive"])?,
        "target_kind": bounded(item.get("target_kind")),
        "target": item.get("target").filter(|v| v.is_string()).map(identifier).transpose()?,
        "attackers": prefix(ids(item.get("attackers"), true)?, 16), "premises": prefix(ids(item.get("premises"), false)?, 16)}),
    )
}
pub(super) fn assumption(item: &Map<String, Value>) -> Result<Value, String> {
    let mut result = json!({"status": state(item.get("status"), CLAIM_STATES)?,
        "validation": item.get("validation").filter(|v| v.is_string()).map(identifier).transpose()?});
    if let Some(status) = item
        .get("time_status")
        .and_then(Value::as_str)
        .filter(|s| ["not_started", "expired", "within_interval"].contains(s))
    {
        result["time_status"] = json!(status);
        for key in ["valid_from", "valid_until"] {
            if let Some(value) = item.get(key).filter(|v| !v.is_null()) {
                result[key] = json!(iso(
                    parse_time(value.as_str().ok_or("Timestamp must be a string")?)?,
                    false
                ));
            }
        }
    }
    Ok(result)
}

pub(super) fn collection_records<'a>(
    assessment: &Value,
    collection: Option<&'a Value>,
) -> Result<Option<&'a Map<String, Value>>, String> {
    let Some(collection) = collection else {
        return Ok(None);
    };
    if !collection.is_object() || assessment["collection_id"].is_null() {
        return Err("Collection must belong to a collected assessment".into());
    }
    if collection["collection_id"] != assessment["collection_id"] {
        return Err("Collection ID differs from the assessment".into());
    }
    if collection["source_digest"] != assessment["source_digest"] {
        return Err("Collection source differs from the assessment".into());
    }
    if Some(canonical_digest(&collection["context"], &host_limits())?)
        != assessment["context_fingerprint"]
            .as_str()
            .map(str::to_owned)
    {
        return Err("Collection context differs from the assessment".into());
    }
    let records = collection["records"]
        .as_object()
        .filter(|r| r.values().all(Value::is_object))
        .ok_or("Collection records must map evidence IDs to observations")?;
    Ok(Some(records))
}

pub(super) fn observation_time(
    projected: &mut Map<String, Value>,
    assessment: &Value,
    assessed_evidence: &Value,
    record: &Value,
) -> Result<(), String> {
    if record["run_id"] != assessed_evidence["run_id"] {
        return Err("Observation ID differs from the assessed evidence".into());
    }
    let parsed = record["collected_at"]
        .as_str()
        .and_then(|s| parse_time(s).ok())
        .zip(
            assessment["assessed_at"]
                .as_str()
                .and_then(|s| parse_time(s).ok()),
        );
    let Some((measured, assessed)) = parsed else {
        projected.insert("age_status".into(), json!("invalid_timestamp"));
        return Ok(());
    };
    let duration = assessed.signed_duration_since(measured);
    let age = duration.num_seconds() as f64
        + (duration - chrono::Duration::seconds(duration.num_seconds()))
            .num_microseconds()
            .unwrap_or(0) as f64
            / 1_000_000.0;
    if record["status"] != "ok" {
        projected.insert("attempted_at".into(), json!(iso(measured, true)));
        projected.insert("age_status".into(), json!("no_observation"));
    } else {
        projected.insert("observed_at".into(), json!(iso(measured, true)));
        projected.insert("age_seconds".into(), json!(age));
        let stale = assessed_evidence["availability_issues"]
            .as_array()
            .is_some_and(|v| v.iter().any(|i| i == "stale_observation"));
        projected.insert(
            "age_status".into(),
            json!(if age < 0.0 {
                "future"
            } else if stale {
                "stale"
            } else {
                "within_max_age"
            }),
        );
    }
    Ok(())
}
fn iso(time: DateTime<Utc>, z: bool) -> String {
    let format = if time.timestamp_subsec_micros() == 0 {
        "%Y-%m-%dT%H:%M:%S"
    } else {
        "%Y-%m-%dT%H:%M:%S%.6f"
    };
    format!("{}{}", time.format(format), if z { "Z" } else { "+00:00" })
}
