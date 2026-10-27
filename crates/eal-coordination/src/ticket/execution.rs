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
    let assessment_id = identifier(outcome, "assessment_id")?;
    let collection_id = identifier(outcome, "collection_id")?;
    let source_digest = identifier(outcome, "source_digest")?;
    let executable_digest = identifier(outcome, "executable_digest")?;
    let claim = identifier(outcome, "claim")?;
    let status = identifier(outcome, "claim_status")?;
    if !["supported", "contested", "unsupported"].contains(&status) {
        return Err("execution claim status is outside the EAL assessment contract".into());
    }
    let source = artifact(previous, "source")?;
    if source.sha256 != source_digest {
        return Err("executed source identity differs from the exported ticket source".into());
    }
    let receipt = json(previous, "build")?;
    let tools = artifact(previous, "tools")?;
    if receipt["schema"] != "engarg.executable-package/1"
        || receipt["files"]["case.eal"] != source.sha256
        || receipt["files"]["tools.toml"] != tools.sha256
        || receipt["claims"]["current_claim"] != claim
        || receipt["executable_digest"] != executable_digest
    {
        return Err(
            "package build identity differs from its exported source, tools or selected claim"
                .into(),
        );
    }
    let execution = result(&response.artifacts, "execution")?;
    let collection = json(&response.artifacts, "collection")?;
    let assessment = json(&response.artifacts, "assessment")?;
    if execution["schema"] != "EAL/execution/1"
        || execution["claim"] != claim
        || execution["collection"] != collection
        || execution["assessment"] != assessment
        || execution["executable_digest"] != executable_digest
    {
        return Err(
            "retained collection or assessment differs from its EAL execution envelope".into(),
        );
    }
    for artifact in [&execution, &collection, &assessment] {
        if artifact["source_digest"] != source_digest {
            return Err("EAL execution artifact has a different source digest".into());
        }
    }
    if collection["collection_id"] != collection_id
        || assessment["collection_id"] != collection_id
        || assessment["assessment_id"] != assessment_id
        || assessment["claims"][claim]["status"] != status
    {
        return Err("selected claim or collection and assessment identities differ across execution artifacts".into());
    }
    if outcome["valid"] != true
        || outcome["complete"] != true
        || assessment["valid"] != true
        || assessment["complete"] != true
    {
        return Err("completed ticket requires the same valid complete native assessment".into());
    }
    let tools_bytes = artifacts::read(&tools.path, MAX_ARTIFACT_BYTES)?;
    let manifest = eal_collector_contracts::RegistryDocument::from_toml(
        std::str::from_utf8(&tools_bytes).map_err(|_| "ticket tools require UTF-8")?,
    )?;
    let context = serde_json::to_value(manifest.context).map_err(|error| error.to_string())?;
    let fingerprint =
        eal_core::canonical::canonical_digest(&context, &eal_core::ExecutionLimits::default())?;
    if collection["context"] != context || assessment["context_fingerprint"] != fingerprint {
        return Err("execution context differs from the operator's packaged tool context".into());
    }
    let source_bytes = artifacts::read(&source.path, MAX_ARTIFACT_BYTES)?;
    let source_text =
        std::str::from_utf8(&source_bytes).map_err(|_| "ticket source requires UTF-8")?;
    let program = eal_core::composition::compile(
        source_text,
        None,
        "ticket.eal",
        eal_core::ExecutionLimits::default(),
    )
    .map_err(|error| error.to_string())?;
    if eal_core::semantic_format::executable_digest(&program, eal_core::methods::default_registry())
        .map_err(|error| error.to_string())?
        != executable_digest
    {
        return Err("execution semantics differ from the exported EAL program".into());
    }
    let plan = eal_core::planning::EvidencePlanner::new(&program)?.plan(claim)?;
    let records = collection["records"]
        .as_object()
        .ok_or("collection lacks its records")?;
    let evaluated = assessment["evidence"]
        .as_object()
        .ok_or("assessment lacks its evidence verdicts")?;
    for evidence in plan.evidence_ids() {
        if !records
            .get(&evidence)
            .is_some_and(|record| record["status"] == "ok")
            || !evaluated.get(&evidence).is_some_and(evidence_is_measured)
        {
            return Err("selected ticket claim has an absent collection record or unresolved final measurement".into());
        }
    }
    Ok(())
}
