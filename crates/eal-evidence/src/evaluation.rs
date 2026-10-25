//! Pure transformations and requirement-specific checks over retained source values.
use crate::{
    execution::{EvaluationResult, EvaluationStatus, SourceRecord, StepResult},
    numeric,
    plan::*,
};
use num_rational::BigRational;
use num_traits::Zero;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn referenced(
    reference: &OutputReference,
    steps: &BTreeMap<String, StepResult>,
) -> Result<Value, String> {
    let step = steps
        .get(&reference.step_id)
        .ok_or("step result is unavailable")?;
    let value = step
        .value
        .as_ref()
        .ok_or("upstream source or derivation is unavailable")?;
    value
        .pointer(&reference.pointer)
        .cloned()
        .ok_or_else(|| "upstream result does not contain the selected JSON pointer".into())
}

pub(crate) fn derive(operation: &DerivationOperation, values: &[Value]) -> Result<Value, String> {
    match operation {
        DerivationOperation::Collect => Ok(json!(values)),
        DerivationOperation::Extract { pointer } => match values {
            [value] => value
                .pointer(pointer)
                .cloned()
                .ok_or_else(|| "derivation output pointer is absent".into()),
            _ => Err("extraction requires exactly one selected input".into()),
        },
        DerivationOperation::Project { fields } => project(values, fields, true),
        DerivationOperation::ProjectValue { fields } => project(values, fields, false),
        DerivationOperation::Count => Ok(numeric::value(
            &BigRational::from_integer(numeric::flat_values(values).len().into()),
            Some("1"),
        )),
        DerivationOperation::AnyIdentityEqual=>match values {
            [identity,Value::Array(candidates)] if !identity.is_array()&&!identity.is_null() && candidates.iter().all(|candidate|!candidate.is_array()&&!candidate.is_null())=>Ok(json!(candidates.iter().any(|candidate|candidate==identity))),
            _=>Err("identity membership requires one known individual identity and an array of known individual identities".into()),
        },
        DerivationOperation::IdentityEqual => {
            if values.iter().any(|value|value.is_array()!=values[0].is_array()){return Err("identity equality cannot mix individual and collection shapes".into());}
            if values.len() < 2 {
                Err("identity comparison requires at least two inputs".into())
            } else {
                Ok(json!(values.windows(2).all(|pair| pair[0] == pair[1])))
            }
        }
        DerivationOperation::Scale {
            numerator,
            denominator,
            from_unit,
            to_unit,
        } => {
            let [selected] = values else {
                return Err("scaling requires exactly one input".into());
            };
            if selected
                .get("unit")
                .and_then(Value::as_str)
                .is_some_and(|unit| unit != from_unit)
                || from_unit.trim().is_empty()
                || to_unit.trim().is_empty()
            {
                return Err("scaling requires matching explicit input/output units".into());
            }
            let denominator = numeric::integer(denominator)?;
            if denominator.is_zero() {
                return Err("scale denominator must be nonzero".into());
            }
            let factor = BigRational::new(numeric::integer(numerator)?, denominator);
            Ok(numeric::value(
                &(numeric::rational(selected)? * factor),
                Some(to_unit),
            ))
        }
        operation => {
            let flattened = numeric::flat_values(values);
            if flattened.is_empty() {
                return Err("numeric derivation requires a nonempty sample".into());
            }
            let unit = numeric::same_unit(&flattened)?;
            let numbers = flattened
                .into_iter()
                .map(numeric::rational)
                .collect::<Result<Vec<_>, _>>()?;
            let result = match operation {
                DerivationOperation::Sum => numbers.iter().cloned().sum(),
                DerivationOperation::Mean => {
                    numbers.iter().cloned().sum::<BigRational>()
                        / BigRational::from_integer(numbers.len().into())
                }
                DerivationOperation::Minimum => numbers.into_iter().min().expect("nonempty"),
                DerivationOperation::Maximum => numbers.into_iter().max().expect("nonempty"),
                _ => unreachable!("closed derivation operation"),
            };
            Ok(numeric::value(&result, unit.as_deref()))
        }
    }
}

fn project(
    values: &[Value],
    fields: &BTreeMap<String, String>,
    fanout: bool,
) -> Result<Value, String> {
    let [value] = values else {
        return Err("projection requires exactly one selected input".into());
    };
    fields
        .iter()
        .map(|(name, pointer)| {
            let select = |value: &Value| {
                value
                    .pointer(pointer)
                    .cloned()
                    .ok_or_else(|| "projection output pointer is absent".to_owned())
            };
            let selected = match value {
                Value::Array(records) if fanout => {
                    Value::Array(records.iter().map(select).collect::<Result<Vec<_>, _>>()?)
                }
                _ => select(value)?,
            };
            Ok((name.clone(), selected))
        })
        .collect::<Result<serde_json::Map<_, _>, _>>()
        .map(Value::Object)
}

pub(crate) fn evaluate(
    evaluation: &RequirementEvaluation,
    steps: &BTreeMap<String, StepResult>,
    source_records: &BTreeMap<String, SourceRecord>,
    now: &str,
) -> EvaluationResult {
    let mut result = EvaluationResult {
        id: evaluation.id.clone(),
        requirement_id: evaluation.requirement_id.clone(),
        status: EvaluationStatus::Pending,
        source_ids: Vec::new(),
        observation_ids: Vec::new(),
        observed_at: None,
        value: None,
        diagnostic: None,
    };
    let all_references: Vec<_> = evaluation
        .inputs
        .iter()
        .chain(&evaluation.applicability)
        .chain(&evaluation.challenges)
        .collect();
    result.source_ids = all_references
        .iter()
        .filter_map(|reference| steps.get(&reference.step_id))
        .flat_map(|step| step.source_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    result.observation_ids = all_references
        .iter()
        .filter_map(|reference| steps.get(&reference.step_id))
        .flat_map(|step| step.observation_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    result.observed_at = all_references
        .iter()
        .filter_map(|reference| steps.get(&reference.step_id))
        .filter_map(|step| step.observed_at.clone())
        .min_by_key(|at| chrono::DateTime::parse_from_rfc3339(at).ok());
    let outcome = (|| -> Result<bool, String> {
        let all_refs = evaluation
            .inputs
            .iter()
            .chain(&evaluation.applicability)
            .chain(&evaluation.challenges);
        let mut sources = BTreeSet::new();
        let mut oldest = None;
        for reference in all_refs {
            let step = steps
                .get(&reference.step_id)
                .ok_or("required step is unavailable")?;
            if step.value.is_none() {
                return Err(step
                    .diagnostic
                    .clone()
                    .unwrap_or_else(|| "required source is unavailable".into()));
            }
            sources.extend(step.source_ids.iter().cloned());
            if let Some(at) = &step.observed_at {
                let instant = chrono::DateTime::parse_from_rfc3339(at)
                    .map_err(|_| "invalid original observation time")?;
                let current = chrono::DateTime::parse_from_rfc3339(now)
                    .map_err(|_| "execution now requires RFC3339")?;
                if instant > current {
                    return Err("observation is later than the explicit assessment time".into());
                }
                if evaluation.max_age_seconds.is_some_and(|maximum| {
                    current
                        .signed_duration_since(instant)
                        .to_std()
                        .is_ok_and(|age| age > std::time::Duration::from_secs(maximum))
                }) {
                    return Err("source observation is stale for this requirement".into());
                }
                if oldest
                    .as_ref()
                    .is_none_or(|(selected, _)| instant < *selected)
                {
                    oldest = Some((instant, at.clone()));
                }
            }
            let sufficient = !evaluation.inputs.iter().any(|input| {
                input.step_id == reference.step_id && input.pointer == reference.pointer
            }) || match evaluation.required_coverage {
                RequiredCoverage::Unspecified => true,
                RequiredCoverage::Witness => step.coverage.supplies_witness_route(),
                RequiredCoverage::CompleteEnumeration => {
                    step.coverage.is_complete() && !step.truncated
                }
                RequiredCoverage::Statistical => step.coverage.sample_size().is_some(),
            };
            if !sufficient {
                return Err(
                    "source coverage does not establish this requirement's logical strength".into(),
                );
            }
        }
        result.source_ids = sources.into_iter().collect();
        result.observed_at = oldest.map(|(_, time)| time);
        for reference in &evaluation.applicability {
            if referenced(reference, steps)? != Value::Bool(true) {
                return Err("applicability check is unresolved or failed".into());
            }
        }
        for reference in &evaluation.challenges {
            if referenced(reference, steps)? != Value::Bool(true) {
                return Err("material challenge check is unresolved or failed".into());
            }
        }
        let values = evaluation
            .inputs
            .iter()
            .map(|reference| referenced(reference, steps))
            .collect::<Result<Vec<_>, _>>()?;
        let passed = match &evaluation.criterion {
            EvaluationCriterion::MethodInput { method } => {
                let [value] = values.as_slice() else {
                    return Err(
                        "method input admission requires exactly one collected input".into(),
                    );
                };
                crate::projection::validate_reasoning_envelope(value, method)?;
                result.value = Some(json!({
                    "method": method,
                    "input_contract_satisfied": true,
                    "method_result_evaluated": false,
                    "scientific_adequacy_verified": false
                }));
                true
            }
            EvaluationCriterion::Judgement { criterion, scope } => {
                let Some(scope) = scope else {
                    return Err(format!(
                        "independent judgement remains required: {criterion}"
                    ));
                };
                let input_steps = evaluation
                    .inputs
                    .iter()
                    .filter_map(|reference| steps.get(&reference.step_id))
                    .collect::<Vec<_>>();
                let retained = crate::judgement::correspondence(
                    criterion,
                    scope,
                    &values,
                    &input_steps,
                    source_records,
                )?;
                let verdict = retained.verdict;
                result.value = Some(
                    json!({"retained_judgement":retained,"judgement_record_correspondence_only":true}),
                );
                match verdict {
                    JudgementVerdict::Supported => true,
                    JudgementVerdict::Rejected => false,
                    JudgementVerdict::Inconclusive => {
                        return Err("retained reviewer judgement is inconclusive".into());
                    }
                }
            }
            EvaluationCriterion::Unavailable { reason } => {
                return Err(format!("required support is unavailable: {reason}"));
            }
            EvaluationCriterion::Equals { expected } => {
                matches!(values.as_slice(),[value] if value==expected)
            }
            EvaluationCriterion::AllTrue | EvaluationCriterion::AnyTrue => {
                let values = numeric::flat_values(&values);
                if values.iter().any(|value| !value.is_boolean()) {
                    return Err("boolean evaluation requires boolean measurements".into());
                }
                if matches!(evaluation.criterion, EvaluationCriterion::AllTrue) {
                    values.iter().all(|value| **value == Value::Bool(true))
                } else {
                    values.iter().any(|value| **value == Value::Bool(true))
                }
            }
            EvaluationCriterion::Numeric {
                comparison,
                threshold,
                value_unit,
                threshold_unit,
            } => {
                if value_unit != threshold_unit {
                    return Err(
                        "numeric threshold uses incompatible units; add an explicit conversion"
                            .into(),
                    );
                }
                let [selected] = values.as_slice() else {
                    return Err("numeric comparison requires exactly one selected quantity".into());
                };
                if selected
                    .get("unit")
                    .and_then(Value::as_str)
                    .is_some_and(|unit| unit != value_unit)
                {
                    return Err("derived quantity unit differs from the requirement".into());
                }
                if threshold
                    .get("unit")
                    .and_then(Value::as_str)
                    .is_some_and(|unit| unit != threshold_unit)
                {
                    return Err(
                        "threshold quantity unit differs from the explicit threshold unit".into(),
                    );
                }
                let observed = numeric::rational(selected)?;
                let expected = numeric::rational(threshold)?;
                match comparison {
                    NumericComparison::LessThan => observed < expected,
                    NumericComparison::AtMost => observed <= expected,
                    NumericComparison::Equal => observed == expected,
                    NumericComparison::AtLeast => observed >= expected,
                    NumericComparison::GreaterThan => observed > expected,
                }
            }
        };
        if !passed
            && matches!(evaluation.criterion, EvaluationCriterion::AnyTrue)
            && evaluation
                .inputs
                .iter()
                .filter_map(|reference| steps.get(&reference.step_id))
                .any(|step| step.truncated || !step.coverage.is_complete())
        {
            return Err(
                "no validated witness was found; incomplete coverage cannot establish absence"
                    .into(),
            );
        }
        match &mut result.value {
            Some(Value::Object(fields)) => {
                fields.insert("criterion_satisfied".into(), json!(passed));
            }
            _ => result.value = Some(json!({"criterion_satisfied":passed})),
        }
        Ok(passed)
    })();
    match outcome {
        Ok(true) => result.status = EvaluationStatus::Satisfied,
        Ok(false) => result.status = EvaluationStatus::NotSatisfied,
        Err(diagnostic) => result.diagnostic = Some(diagnostic),
    }
    result
}
