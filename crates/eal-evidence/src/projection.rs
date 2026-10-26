//! Normal command-collector projection of a pinned requirement evaluation.
use crate::{
    execution::{EvaluationStatus, ExecutionReport},
    plan::{EvidenceExecutionPlan, canonical_digest, valid_digest},
};
use eal_collector_contracts::{AcquisitionRequest, ObservationEnvelope};
use serde::{Deserialize, Serialize};
use serde_json::json;

mod reasoning;
pub use reasoning::{
    REASONING_INPUT_DENIED_SCHEMA, REASONING_INPUT_SCHEMA, ReasoningInputAcquisition,
    ReasoningPremiseBinding, ReasoningPremiseCorrespondence, graph_request_requirements,
    project_graph_request, validate_graph_request, validate_reasoning_envelope,
};

pub const SUPPORT_ACQUISITION_SCHEMA: &str = "engarg.support-acquisition/1";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportAcquisitionInput {
    pub schema: String,
    pub case_digest: String,
    pub execution_plan_sha256: String,
    pub requirement_id: String,
}
fn input(request: &AcquisitionRequest) -> Result<SupportAcquisitionInput, String> {
    request.validate()?;
    let input: SupportAcquisitionInput = serde_json::from_value(request.input.clone())
        .map_err(|_| "invalid support acquisition input fields")?;
    if input.schema != SUPPORT_ACQUISITION_SCHEMA
        || !valid_digest(&input.case_digest)
        || !valid_digest(&input.execution_plan_sha256)
        || input.requirement_id.trim().is_empty()
        || input.requirement_id.len() > 256
    {
        return Err(
            "support acquisition requires versioned case/plan identities and a requirement ID"
                .into(),
        );
    }
    if request
        .context
        .get("engarg_export_digest")
        .and_then(serde_json::Value::as_str)
        != Some(input.case_digest.as_str())
    {
        return Err("support acquisition case identity differs from its EAL export context".into());
    }
    if request
        .context
        .get("engarg_execution_plan_sha256")
        .is_some_and(|expected| expected.as_str() != Some(input.execution_plan_sha256.as_str()))
    {
        return Err("support acquisition plan identity differs from its EAL export context".into());
    }
    Ok(input)
}
/// Check case correspondence, the exact canonical graph and requirement before IO.
pub fn validate_support_request(
    request: &AcquisitionRequest,
    plan: &EvidenceExecutionPlan,
) -> Result<SupportAcquisitionInput, String> {
    plan.validate()?;
    let input = input(request)?;
    if input.execution_plan_sha256 != canonical_digest(plan)? {
        return Err("support acquisition execution plan digest differs".into());
    }
    if plan
        .evaluations
        .iter()
        .filter(|evaluation| evaluation.requirement_id == input.requirement_id)
        .count()
        != 1
    {
        return Err("support acquisition selects an absent or ambiguous requirement".into());
    }
    Ok(input)
}
/// Pending judgements, stale sources and missing observations are unavailable;
/// a measured negative remains a collected false criterion result.
pub fn project_requirement(
    request: &AcquisitionRequest,
    report: &ExecutionReport,
) -> Result<ObservationEnvelope, String> {
    let input = input(request)?;
    if input.execution_plan_sha256 != report.plan_sha256 {
        return Err("support execution report differs from its pinned plan".into());
    }
    let mut selected = report
        .evaluations
        .iter()
        .filter(|evaluation| evaluation.requirement_id == input.requirement_id);
    let evaluation = selected
        .next()
        .ok_or("support requirement evaluation is absent")?;
    if selected.next().is_some() {
        return Err("support requirement evaluation is ambiguous".into());
    }
    if evaluation.status == EvaluationStatus::Pending {
        return Err(evaluation
            .diagnostic
            .clone()
            .unwrap_or_else(|| "support requirement remains unavailable".into()));
    }
    let observed_at = evaluation
        .observed_at
        .clone()
        .filter(|_| !evaluation.source_ids.is_empty())
        .ok_or("support evaluation has no original source observation time")?;
    chrono::DateTime::parse_from_rfc3339(&observed_at)
        .map_err(|_| "support evaluation has an invalid original observation time")?;
    if evaluation
        .source_ids
        .iter()
        .any(|id| !report.sources.contains_key(id))
    {
        return Err("support evaluation source lineage is absent".into());
    }
    if evaluation.observation_ids.is_empty() {
        return Err("support evaluation observation lineage is absent".into());
    }
    let observations=evaluation.observation_ids.iter().map(|id|{
        let record=report.observations.get(id).ok_or("support evaluation immutable observation is absent")?;
        if record.observation_id!=*id || record.observation_identity()?!=*id || !evaluation.source_ids.contains(&record.source_id){return Err("support evaluation observation correspondence differs".into());}
        Ok(json!({"observation_id":id,"source_id":record.source_id,"value_sha256":record.value_sha256,"observed_at":record.observed_at,"snapshot":format!("observations/{id}.json")}))
    }).collect::<Result<Vec<_>,String>>()?;
    Ok(ObservationEnvelope {
        value: json!({"execution_plan_sha256":report.plan_sha256,"collected":true,"execution_complete":report.complete,"budget_exhausted":report.budget_exhausted,"criterion_satisfied":evaluation.status==EvaluationStatus::Satisfied,"requirement_id":evaluation.requirement_id,"evaluation_id":evaluation.id,"status":evaluation.status,"source_ids":evaluation.source_ids,"observation_ids":evaluation.observation_ids,"evaluation":evaluation}),
        observed_at: Some(observed_at),
        context: Some(request.context.clone()),
        request: Some(request.acquisition()),
        details: Some(
            json!({"execution_plan_sha256":report.plan_sha256,"evaluated_at":report.evaluated_at,"execution_complete":report.complete,"budget_exhausted":report.budget_exhausted,"metric_scope":report.metric_scope,"metrics":report.metrics,"observations":observations}),
        ),
    })
}
