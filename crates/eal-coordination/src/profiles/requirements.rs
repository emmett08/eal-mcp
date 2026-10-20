use super::{CRITERIA, PROFILE_NAMES, WORKLOAD_FIELDS, exact_keys, integer};
use crate::contracts::{json_object, text};
use eal_core::expressions::numbers;
use serde_json::{Value, json};
use std::cmp::Ordering;

pub fn validate_requirements(
    profile: &str,
    claims: &[String],
    value: &Value,
) -> Result<(), String> {
    if !PROFILE_NAMES.contains(&profile) {
        return Err("Unknown installed coordination profile".into());
    }
    let fields: &[&str] = if profile == "registered_claims/1" {
        &["context"]
    } else {
        &["context", "workload", "criteria", "claim_outputs"]
    };
    if !exact_keys(value, fields) {
        return Err("Profile requires exactly its declared requirement fields".into());
    }
    json_object(&value["context"], "context", 16384)?;
    if profile == "registered_claims/1" {
        return Ok(());
    }
    let workload = &value["workload"];
    if !exact_keys(workload, &WORKLOAD_FIELDS) {
        return Err("API workload requires all five identity fields".into());
    }
    for key in WORKLOAD_FIELDS
        .iter()
        .filter(|key| **key != "concurrent_clients")
    {
        text(&workload[*key], key, 128)?;
    }
    if !integer(&workload["concurrent_clients"])
        || !matches!(
            numbers::compare(&workload["concurrent_clients"], &json!(1)),
            Some(Ordering::Equal | Ordering::Greater)
        )
    {
        return Err("Concurrent clients requires a positive integer".into());
    }
    if !exact_keys(&value["context"], &["service", "build_id", "dataset"]) {
        return Err("API context requires service, build_id and dataset".into());
    }
    let criteria = &value["criteria"];
    if !exact_keys(
        criteria,
        &[
            CRITERIA[0].0,
            CRITERIA[1].0,
            CRITERIA[2].0,
            "evidence_max_age_seconds",
        ],
    ) {
        return Err("API criteria require the complete criterion and freshness contract".into());
    }
    if criteria.as_object().unwrap().values().any(|number| {
        !matches!(
            numbers::compare(number, &json!(0)),
            Some(Ordering::Equal | Ordering::Greater)
        ) || !matches!(
            numbers::compare(number, &json!(1e100)),
            Some(Ordering::Equal | Ordering::Less)
        )
    }) || !integer(&criteria["minimum_requests"])
    {
        return Err(
            "API criteria require finite nonnegative numbers and integer request count".into(),
        );
    }
    if !value["claim_outputs"].as_object().is_some_and(|outputs| {
        outputs.len() == claims.len()
            && claims.iter().all(|claim| {
                outputs
                    .get(claim)
                    .and_then(Value::as_str)
                    .is_some_and(|output| matches!(output, "passes" | "fails"))
            })
    }) {
        return Err("API claim_outputs must bind each selected claim to passes or fails".into());
    }
    Ok(())
}
