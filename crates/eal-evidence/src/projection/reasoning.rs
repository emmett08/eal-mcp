//! Project a collected method input without turning an evaluation flag into evidence.
use super::{SUPPORT_ACQUISITION_SCHEMA, SupportAcquisitionInput};
use crate::{
    execution::{EvaluationResult, EvaluationStatus, ExecutionReport, StepStatus},
    plan::{
        EvaluationCriterion, EvidenceExecutionPlan, OutputReference, canonical_digest,
        valid_pointer,
    },
};
use eal_collector_contracts::{AcquisitionRequest, ObservationEnvelope};
use eal_core::{ExecutionLimits, methods, propositions};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub const REASONING_INPUT_SCHEMA: &str = "engarg.reasoning-input/1";
/// A collected refusal to admit a formal input whose empirical premise failed.
/// This observation deliberately does not satisfy EAL's typed-input predicate.
pub const REASONING_INPUT_DENIED_SCHEMA: &str = "engarg.reasoning-input-denied/1";

/// The exact declared evaluation input is the only selectable value. The value
/// must already contain the typed envelope; request metadata never supplies it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReasoningInputAcquisition {
    pub schema: String,
    pub case_digest: String,
    pub execution_plan_sha256: String,
    pub requirement_id: String,
    pub output: OutputReference,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub premise_bindings: Vec<ReasoningPremiseBinding>,
}

/// Presence of the formal operand and declaration of its separate evidence
/// obligation are checked. Truth and prose correspondence are not inferred.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReasoningPremiseBinding {
    pub premise_id: String,
    pub input_pointer: String,
    pub requirement_id: String,
    pub correspondence: ReasoningPremiseCorrespondence,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReasoningPremiseCorrespondence {
    IdentityValue {
        source_input_index: usize,
        source_pointer: String,
    },
    AcceptedPremise,
}

/// Admission verifies a bounded kernel input and its typed metadata. It does
/// not compute a conclusion or establish the adequacy of an empirical model.
pub fn validate_reasoning_envelope(value: &Value, method: &str) -> Result<(), String> {
    let contract = methods::default_registry()
        .get(method)
        .filter(|contract| contract.evidence_kind.is_some())
        .ok_or("method input requires an installed computational method")?;
    let fields: BTreeSet<_> = [
        "schema",
        "method",
        "subject",
        "quantity",
        "unit",
        "scope",
        "valid_from",
        "valid_until",
        "payload",
    ]
    .into_iter()
    .collect();
    let object = value
        .as_object()
        .filter(|object| object.keys().map(String::as_str).collect::<BTreeSet<_>>() == fields)
        .ok_or("method input requires exactly the EAL/typed-input/1 envelope fields")?;
    if object["schema"] != propositions::SCHEMA || object["method"] != method {
        return Err("method input schema or method identity differs".into());
    }
    for field in [
        "subject",
        "quantity",
        "unit",
        "scope",
        "valid_from",
        "valid_until",
    ] {
        if !object[field].as_str().is_some_and(|text| {
            !text.trim().is_empty() && text.len() <= 4096 && !text.chars().any(char::is_control)
        }) {
            return Err(format!(
                "method input {field} requires bounded nonblank text"
            ));
        }
    }
    let quantity = object["quantity"].as_str().expect("checked text");
    let unit = object["unit"].as_str().expect("checked text");
    if !contract
        .quantities
        .iter()
        .any(|accepted| accepted == quantity)
        || propositions::quantity_dimension(quantity).is_none()
        || propositions::unit_spec(unit).map(|(dimension, _)| dimension)
            != propositions::quantity_dimension(quantity)
    {
        return Err("method input quantity or unit is incompatible with the method".into());
    }
    let start =
        chrono::DateTime::parse_from_rfc3339(object["valid_from"].as_str().expect("checked text"))
            .map_err(|_| "method input interval requires RFC3339 timestamps")?;
    let end =
        chrono::DateTime::parse_from_rfc3339(object["valid_until"].as_str().expect("checked text"))
            .map_err(|_| "method input interval requires RFC3339 timestamps")?;
    if start >= end {
        return Err("method input valid_from must precede valid_until".into());
    }
    let payload = &object["payload"];
    let limits = ExecutionLimits::default();
    eal_core::validation::check_json_resources(value, &limits)?;
    if eal_core::canonical::canonical_json_bytes(payload, &limits)?.len() > contract.max_input_bytes
    {
        return Err("method input payload exceeds its registered byte allowance".into());
    }
    let errors = methods::schema_errors_with_limits(payload, &contract.input_schema, "$", &limits);
    if !errors.is_empty() {
        return Err(format!(
            "method input contract violation: {}",
            errors.join("; ")
        ));
    }
    Ok(())
}

fn selected_request(
    request: &AcquisitionRequest,
    plan: &EvidenceExecutionPlan,
) -> Result<(ReasoningInputAcquisition, AcquisitionRequest), String> {
    request.validate()?;
    let selected: ReasoningInputAcquisition = serde_json::from_value(request.input.clone())
        .map_err(|_| "invalid reasoning input acquisition fields")?;
    if selected.schema != REASONING_INPUT_SCHEMA {
        return Err("unsupported reasoning input acquisition schema".into());
    }
    let common = SupportAcquisitionInput {
        schema: SUPPORT_ACQUISITION_SCHEMA.into(),
        case_digest: selected.case_digest.clone(),
        execution_plan_sha256: selected.execution_plan_sha256.clone(),
        requirement_id: selected.requirement_id.clone(),
    };
    let mut ordinary = request.clone();
    ordinary.input = serde_json::to_value(common).map_err(|_| "invalid support identities")?;
    super::validate_support_request(&ordinary, plan)?;
    let evaluation = plan
        .evaluations
        .iter()
        .find(|evaluation| evaluation.requirement_id == selected.requirement_id)
        .ok_or("reasoning input requirement is absent")?;
    if !matches!(
        evaluation.criterion,
        EvaluationCriterion::MethodInput { .. }
    ) || !evaluation.inputs.iter().any(|reference| {
        reference.step_id == selected.output.step_id && reference.pointer == selected.output.pointer
    }) {
        return Err(
            "reasoning input must select the exact declared method-input evaluation reference"
                .into(),
        );
    }
    let mut premises = BTreeSet::new();
    if selected.premise_bindings.len() > 128
        || selected.premise_bindings.iter().any(|binding| {
            binding.premise_id.trim().is_empty()
                || binding.premise_id.len() > 256
                || binding.premise_id.chars().any(char::is_control)
                || !premises.insert((binding.premise_id.as_str(), binding.input_pointer.as_str()))
                || !valid_pointer(&binding.input_pointer)
                || plan
                    .evaluations
                    .iter()
                    .filter(|evaluation| evaluation.requirement_id == binding.requirement_id)
                    .count()
                    != 1
        })
    {
        return Err("reasoning premise bindings require distinct bounded premise/pointer pairs, valid payload pointers and exact declared requirements".into());
    }
    let EvaluationCriterion::MethodInput { method } = &evaluation.criterion else {
        return Err("reasoning input requires method input admission".into());
    };
    for binding in &selected.premise_bindings {
        let required = plan
            .evaluations
            .iter()
            .find(|evaluation| evaluation.requirement_id == binding.requirement_id)
            .ok_or("reasoning premise requirement is absent")?;
        if matches!(required.criterion, EvaluationCriterion::MethodInput { .. }) {
            return Err("method input admission cannot establish its own formal premises".into());
        }
        match &binding.correspondence {
            ReasoningPremiseCorrespondence::IdentityValue {
                source_input_index,
                source_pointer,
            } if *source_input_index >= required.inputs.len() || !valid_pointer(source_pointer) => {
                return Err(
                    "reasoning premise correspondence selects an absent input or invalid pointer"
                        .into(),
                );
            }
            ReasoningPremiseCorrespondence::AcceptedPremise => {
                let index = binding
                    .input_pointer
                    .strip_prefix("/premises/")
                    .and_then(|value| value.parse::<usize>().ok());
                if method != "deductive/1"
                    || index.is_none()
                    || binding.input_pointer != format!("/premises/{}", index.unwrap_or(0))
                {
                    return Err("accepted premise correspondence is confined to a deductive /premises/N operand".into());
                }
            }
            _ => {}
        }
    }
    Ok((selected, ordinary))
}

/// Validate either normal support collection or typed input projection before IO.
/// The returned identities have the ordinary support schema for compatibility.
pub fn validate_graph_request(
    request: &AcquisitionRequest,
    plan: &EvidenceExecutionPlan,
) -> Result<SupportAcquisitionInput, String> {
    if request.input["schema"] == REASONING_INPUT_SCHEMA {
        let (_, ordinary) = selected_request(request, plan)?;
        super::validate_support_request(&ordinary, plan)
    } else {
        super::validate_support_request(request, plan)
    }
}

/// All empirical premise requirements execute in the same bounded graph call
/// as the formal input. Declaring a requirement never substitutes for collecting it.
pub fn graph_request_requirements(
    request: &AcquisitionRequest,
    plan: &EvidenceExecutionPlan,
) -> Result<Vec<String>, String> {
    if request.input["schema"] == REASONING_INPUT_SCHEMA {
        let (selected, _) = selected_request(request, plan)?;
        let mut ids = BTreeSet::from([selected.requirement_id]);
        ids.extend(
            selected
                .premise_bindings
                .into_iter()
                .map(|binding| binding.requirement_id),
        );
        Ok(ids.into_iter().collect())
    } else {
        Ok(vec![
            super::validate_support_request(request, plan)?.requirement_id,
        ])
    }
}

/// Preserve the selected collected value, original clock and immutable lineage.
/// Pending support cannot produce a method observation. A measured false
/// premise produces a collected admission refusal, without a formal payload.
pub fn project_graph_request(
    request: &AcquisitionRequest,
    plan: &EvidenceExecutionPlan,
    report: &ExecutionReport,
) -> Result<ObservationEnvelope, String> {
    if request.input["schema"] != REASONING_INPUT_SCHEMA {
        return super::project_requirement(request, report);
    }
    let (selected, ordinary) = selected_request(request, plan)?;
    let mut envelope = super::project_requirement(&ordinary, report)?;
    let evaluation = report
        .evaluations
        .iter()
        .find(|evaluation| evaluation.requirement_id == selected.requirement_id)
        .ok_or("reasoning input evaluation is absent")?;
    if evaluation.status != EvaluationStatus::Satisfied {
        return Err("reasoning input admission is unresolved or not satisfied".into());
    }
    let step = report
        .steps
        .get(&selected.output.step_id)
        .filter(|step| matches!(step.status, StepStatus::Measured | StepStatus::Derived))
        .ok_or("reasoning input source or derivation is unavailable")?;
    if step.source_ids.is_empty()
        || step.observation_ids.is_empty()
        || step
            .source_ids
            .iter()
            .any(|id| !evaluation.source_ids.contains(id))
        || step
            .observation_ids
            .iter()
            .any(|id| !evaluation.observation_ids.contains(id))
    {
        return Err("reasoning input lineage differs from the admitted requirement".into());
    }
    verify_observations(evaluation, report, envelope.observed_at.as_deref())?;
    verify_value(&selected.output.step_id, plan, report)?;
    let value = step
        .value
        .as_ref()
        .and_then(|value| value.pointer(&selected.output.pointer))
        .ok_or("reasoning input selected value is absent")?;
    let definition = plan
        .evaluations
        .iter()
        .find(|evaluation| evaluation.requirement_id == selected.requirement_id)
        .ok_or("reasoning input requirement is absent")?;
    let EvaluationCriterion::MethodInput { method } = &definition.criterion else {
        return Err("reasoning input does not select method input admission".into());
    };
    validate_reasoning_envelope(value, method)?;
    let mut source_ids: BTreeSet<_> = evaluation.source_ids.iter().cloned().collect();
    let mut observation_ids: BTreeSet<_> = evaluation.observation_ids.iter().cloned().collect();
    let mut observations = envelope
        .details
        .as_ref()
        .and_then(|details| details["observations"].as_array())
        .cloned()
        .unwrap_or_default();
    let mut denied_premises = Vec::new();
    for binding in &selected.premise_bindings {
        let formal = value["payload"]
            .pointer(&binding.input_pointer)
            .filter(|value| !value.is_null())
            .ok_or_else(|| {
                format!(
                    "reasoning input formal premise {} is absent",
                    binding.premise_id
                )
            })?;
        let required = report
            .evaluations
            .iter()
            .find(|evaluation| evaluation.requirement_id == binding.requirement_id)
            .ok_or("reasoning premise evidence has not been acquired")?;
        if required.status == EvaluationStatus::Pending {
            return Err(format!(
                "reasoning premise evidence {} remains unresolved",
                binding.premise_id
            ));
        }
        let requirement = plan
            .evaluations
            .iter()
            .find(|evaluation| evaluation.requirement_id == binding.requirement_id)
            .ok_or("reasoning premise requirement is absent")?;
        let replayed = crate::evaluation::evaluate(
            requirement,
            &report.steps,
            &report.sources,
            &report.evaluated_at,
        );
        if replayed.status != required.status {
            return Err("reasoning premise evaluation differs from its collected inputs".into());
        }
        let mut premise_request = ordinary.clone();
        premise_request.input["requirement_id"] = json!(binding.requirement_id);
        let premise_envelope = super::project_requirement(&premise_request, report)?;
        verify_observations(required, report, premise_envelope.observed_at.as_deref())?;
        match &binding.correspondence {
            ReasoningPremiseCorrespondence::IdentityValue {
                source_input_index,
                source_pointer,
            } => {
                let reference = requirement
                    .inputs
                    .get(*source_input_index)
                    .ok_or("reasoning premise evidence input is absent")?;
                verify_value(&reference.step_id, plan, report)?;
                let observed = crate::evaluation::referenced(reference, &report.steps)?;
                let observed = observed
                    .pointer(source_pointer)
                    .filter(|value| !value.is_null())
                    .ok_or("reasoning premise evidence value is absent")?;
                if canonical_digest(observed)? != canonical_digest(formal)? {
                    return Err(format!(
                        "reasoning premise {} differs from its acquired evidence value",
                        binding.premise_id
                    ));
                }
            }
            ReasoningPremiseCorrespondence::AcceptedPremise => {
                // Restriction to deductive /premises/N and the independent
                // criterion is checked before IO. Prose mapping stays authored.
            }
        }
        if required.status == EvaluationStatus::NotSatisfied {
            denied_premises.push(json!({
                "premise_id":binding.premise_id,
                "requirement_id":binding.requirement_id,
                "evaluation_id":required.id,
                "status":"not_satisfied"
            }));
        }
        if premise_envelope
            .observed_at
            .as_deref()
            .and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok())
            < envelope
                .observed_at
                .as_deref()
                .and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok())
        {
            envelope.observed_at = premise_envelope.observed_at.clone();
        }
        source_ids.extend(required.source_ids.iter().cloned());
        observation_ids.extend(required.observation_ids.iter().cloned());
        if let Some(additional) = premise_envelope
            .details
            .as_ref()
            .and_then(|details| details["observations"].as_array())
        {
            for original in additional {
                if !observations
                    .iter()
                    .any(|record| record["observation_id"] == original["observation_id"])
                {
                    observations.push(original.clone());
                }
            }
        }
    }
    let premises_satisfied = denied_premises.is_empty();
    envelope.value = if premises_satisfied {
        value.clone()
    } else {
        json!({
            "schema":REASONING_INPUT_DENIED_SCHEMA,
            "method":method,
            "reason":"premise_requirement_unsatisfied",
            "denied_premises":denied_premises
        })
    };
    envelope.request = Some(request.acquisition());
    if let Some(details) = envelope.details.as_mut().and_then(Value::as_object_mut) {
        details.insert("observations".into(), json!(observations));
        details.insert(
            "reasoning_input".into(),
            json!({
                "requirement_id": selected.requirement_id,
                "output": selected.output,
                "method": method,
                "premise_bindings": selected.premise_bindings,
                "source_ids": source_ids,
                "observation_ids": observation_ids,
                "formal_input_correspondence_checked": true,
                "premise_truth_verified": false,
                "prose_correspondence_verified": false,
                "input_contract_satisfied": true,
                "formal_premises_satisfied": premises_satisfied,
                "method_result_evaluated": false,
                "scientific_adequacy_verified": false
            }),
        );
    }
    Ok(envelope)
}

fn verify_observations(
    evaluation: &EvaluationResult,
    report: &ExecutionReport,
    observed_at: Option<&str>,
) -> Result<(), String> {
    for id in &evaluation.observation_ids {
        let observation = report
            .observations
            .get(id)
            .ok_or("reasoning input immutable observation is absent")?;
        if canonical_digest(&observation.value)? != observation.value_sha256
            || canonical_digest(&(&observation.source, &observation.arguments))?
                != observation.source_id
        {
            return Err("reasoning input original observation identity differs".into());
        }
    }
    let oldest = evaluation
        .observation_ids
        .iter()
        .filter_map(|id| report.observations.get(id))
        .map(|record| chrono::DateTime::parse_from_rfc3339(&record.observed_at))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "reasoning input observation clock is invalid")?
        .into_iter()
        .min()
        .ok_or("reasoning input observation clock is absent")?;
    if observed_at.and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok()) != Some(oldest) {
        return Err("reasoning input does not retain its oldest original observation clock".into());
    }
    Ok(())
}

// Reports come from the executor, not a trusted arbitrary JSON import. Still
// check that projection cannot replace an observed payload with a report flag.
// Pure derivations are replayed, and acquisition values must occur in their
// retained original observations. Provider qualification remains separate.
fn verify_value(
    id: &str,
    plan: &EvidenceExecutionPlan,
    report: &ExecutionReport,
) -> Result<(), String> {
    let mut checked = BTreeSet::new();
    let mut pending = vec![(id, false)];
    while let Some((id, expanded)) = pending.pop() {
        if checked.contains(id) {
            continue;
        }
        if checked.len() >= 4096 || pending.len() > 16384 {
            return Err("reasoning input lineage exceeds the graph allowance".into());
        }
        let step = report
            .steps
            .get(id)
            .ok_or("reasoning input step is absent")?;
        let value = step
            .value
            .as_ref()
            .ok_or("reasoning input step has no collected value")?;
        if let Some(derivation) = plan
            .derivations
            .iter()
            .find(|definition| definition.id == id)
        {
            if !expanded {
                pending.push((id, true));
                for reference in &derivation.inputs {
                    if !checked.contains(reference.step_id.as_str()) {
                        pending.push((&reference.step_id, false));
                    }
                }
                continue;
            }
            let values = derivation
                .inputs
                .iter()
                .map(|reference| crate::evaluation::referenced(reference, &report.steps))
                .collect::<Result<Vec<_>, _>>()?;
            let replayed = crate::evaluation::derive(&derivation.operation, &values)?;
            if canonical_digest(&replayed)? != canonical_digest(value)? {
                return Err(
                    "reasoning input derivation differs from its collected operands".into(),
                );
            }
        } else {
            let acquisition = plan
                .acquisitions
                .iter()
                .find(|definition| definition.id == id)
                .ok_or("reasoning input step is not declared in its pinned plan")?;
            let originals = step
                .observation_ids
                .iter()
                .filter_map(|id| report.observations.get(id))
                .filter(|record| {
                    record.source.provider_id == acquisition.source.provider_id
                        && record.source.capability_id == acquisition.source.capability_id
                        && record.source.capability_version == acquisition.source.capability_version
                        && record.source.input_schema_sha256
                            == acquisition.source.input_schema_sha256
                        && record.source.output_schema_sha256
                            == acquisition.source.output_schema_sha256
                        && record.source.qualified_contract_sha256
                            == acquisition.source.qualified_contract_sha256
                        && record.source.historical_interval
                            == acquisition.source.historical_interval
                        && step.source_ids.contains(&record.source_id)
                        && acquisition
                            .source
                            .scope
                            .as_object()
                            .is_some_and(|expected| {
                                expected
                                    .iter()
                                    .all(|(key, value)| record.source.scope.get(key) == Some(value))
                            })
                })
                .map(|record| &record.value)
                .collect::<Vec<_>>();
            let matches = if acquisition.for_each.is_some() {
                value
                    .as_array()
                    .is_some_and(|values| values.iter().all(|value| originals.contains(&value)))
            } else {
                originals.contains(&value)
            };
            if !matches {
                return Err(
                    "reasoning input acquisition differs from its original observations".into(),
                );
            }
        }
        checked.insert(id.to_owned());
    }
    Ok(())
}
