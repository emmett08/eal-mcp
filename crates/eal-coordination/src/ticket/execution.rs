//! Correspondence between EAL execution artifacts and the terminal ticket summary.
use super::artifacts::{self, MAX_ARTIFACT_BYTES};
use eal_coordination_contracts::ticket::{Artifact, StageResult, evidence_is_measured};
use serde_json::Value;

fn artifact<'a>(artifacts: &'a [Artifact], kind: &str) -> Result<&'a Artifact, String> {
    artifacts
        .iter()
        .find(|artifact| artifact.kind == kind)
        .ok_or_else(|| format!("execution correspondence requires {kind} artifact"))
}
fn json(artifacts: &[Artifact], kind: &str) -> Result<Value, String> {
    let bytes = artifacts::read(&artifact(artifacts, kind)?.path, MAX_ARTIFACT_BYTES)?;
    eal_collector_contracts::strict_json_exact(
        &bytes,
        eal_collector_contracts::JsonLimits {
            max_bytes: MAX_ARTIFACT_BYTES as usize,
            max_depth: 128,
            max_nodes: 1_000_000,
        },
    )
}
fn result(artifacts: &[Artifact], kind: &str) -> Result<Value, String> {
    let value = json(artifacts, kind)?;
    if value["is_error"] != false || !value["result"].is_object() {
        return Err(format!("{kind} must retain a successful EAL host response"));
    }
    Ok(value["result"].clone())
}
fn identifier<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 256 && s.trim() == *s)
        .ok_or_else(|| format!("execution outcome requires bounded nonempty {key}"))
}

pub(super) fn validate(response: &StageResult, previous: &[Artifact]) -> Result<(), String> {
    let outcome = response
        .outcome
        .as_ref()
        .ok_or("completed execution requires an outcome")?;
    let entry = identifier(outcome, "entry_id")?;
    let assessment_id = identifier(outcome, "assessment_id")?;
    let collection_id = identifier(outcome, "collection_id")?;
    let initial_collection = identifier(outcome, "initial_collection_id")?;
    let source_digest = identifier(outcome, "source_digest")?;
    if source_digest.len() != 64
        || !source_digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("execution source_digest must be a lowercase SHA-256 digest".into());
    }
    let status = identifier(outcome, "claim_status")?;
    if !["supported", "contested", "unsupported"].contains(&status) {
        return Err("execution claim status is outside the EAL assessment contract".into());
    }
    let source = artifact(previous, "source")?;
    if source.sha256 != source_digest {
        return Err("executed source identity differs from the exported ticket source".into());
    }
    let bundle = json(previous, "bundle")?;
    let source_bytes = artifacts::read(&source.path, MAX_ARTIFACT_BYTES)?;
    if bundle["source"].as_str().map(str::as_bytes) != Some(source_bytes.as_slice()) {
        return Err("exported bundle and source artifact differ".into());
    }
    let registration = json(&response.artifacts, "source_registration")?;
    let collection = result(&response.artifacts, "collection")?;
    let registered = result(&response.artifacts, "registered_assessment")?;
    let assessment = result(&response.artifacts, "assessment")?;
    if registration["entry_id"] != entry
        || registered["entry_id"] != entry
        || registered["schema"] != "EAL/registered-assessment/1"
    {
        return Err("registered ticket entry identity differs across execution artifacts".into());
    }
    for artifact in [&registration, &collection, &registered, &assessment] {
        if artifact["source_digest"] != source_digest {
            return Err("EAL execution artifact has a different source digest".into());
        }
    }
    if collection["collection_id"] != initial_collection
        || registered["collection_id"] != collection_id
        || assessment["collection_id"] != collection_id
        || registered["assessment_id"] != assessment_id
        || assessment["assessment_id"] != assessment_id
    {
        return Err("collection or assessment identities differ across execution artifacts".into());
    }
    let claim = registered["claim"]
        .as_str()
        .ok_or("registered assessment lacks its selected claim")?;
    if bundle["claim_names"]["current_claim"] != claim
        || assessment["claims"][claim]["status"] != status
        || registered["status"] != status
    {
        return Err(
            "selected claim or status differs across exported and assessed artifacts".into(),
        );
    }
    if outcome["valid"] != true
        || outcome["complete"] != true
        || assessment["valid"] != true
        || assessment["complete"] != true
    {
        return Err("completed ticket requires the same valid complete native assessment".into());
    }
    let source_text =
        std::str::from_utf8(&source_bytes).map_err(|_| "ticket source requires UTF-8")?;
    let program = eal_core::composition::compile(
        source_text,
        None,
        "ticket.eal",
        eal_core::ExecutionLimits::default(),
    )
    .map_err(|error| error.to_string())?;
    let plan = eal_core::planning::EvidencePlanner::new(&program)?.plan(claim)?;
    let initial_records = collection["records"]
        .as_object()
        .ok_or("initial collection lacks its records")?;
    let evaluated = assessment["evidence"]
        .as_object()
        .ok_or("assessment lacks its evidence verdicts")?;
    for evidence in plan.evidence_ids() {
        if !initial_records.contains_key(&evidence)
            || !evaluated.get(&evidence).is_some_and(evidence_is_measured)
        {
            return Err("selected ticket claim has an absent collection record or unresolved final measurement".into());
        }
    }
    Ok(())
}
