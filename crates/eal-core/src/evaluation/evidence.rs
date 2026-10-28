//! Check scoped observation correspondence, freshness and declared predicates.
use crate::evaluation::{AvailabilityIssue, PredicateIssue};
mod failure;
mod scoped_review;
use super::{
    entry, environment_context, environment_fingerprint, general_number, name_repr, py_repr,
};
use crate::{
    ExecutionLimits, canonical, expressions,
    semantic::{Environment, Predicate, Program},
    validation,
};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceVerdict {
    pub entry: Value,
    pub complete: bool,
    /// The observation passed record integrity, scope and the selected time checks.
    /// Authored support predicates do not determine whether its fields are usable data.
    pub data_available: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct PredicateCheck {
    pub holds: bool,
    pub comparable: bool,
    pub reason: String,
    pub issue: Option<PredicateIssue>,
}

pub(crate) fn predicate_check(
    predicate: &Predicate,
    value: &Value,
    limits: &ExecutionLimits,
) -> PredicateCheck {
    if let Some(expression) = &predicate.expression {
        let evaluated = expressions::evaluate_expression_with_limits(expression, value, limits);
        let Some(holds) = evaluated.value.as_ref().and_then(Value::as_bool) else {
            return PredicateCheck {
                holds: false,
                comparable: false,
                reason: if evaluated.reason.is_empty() {
                    "Expression requires a Boolean result".into()
                } else {
                    evaluated.reason
                },
                issue: Some(match evaluated.issue.as_deref() {
                    Some("missing_field") => PredicateIssue::MissingField,
                    Some("invalid_expression") => PredicateIssue::InvalidExpression,
                    _ => PredicateIssue::IncompatibleType,
                }),
            };
        };
        let text = match expressions::format_expression_with_limits(expression, limits) {
            Ok(text) => text,
            Err(reason) => {
                return PredicateCheck {
                    holds: false,
                    comparable: false,
                    reason,
                    issue: Some(PredicateIssue::InvalidExpression),
                };
            }
        };
        return PredicateCheck {
            holds,
            comparable: true,
            reason: format!("{} {}", text, if holds { "holds" } else { "does not hold" }),
            issue: if holds {
                None
            } else {
                Some(PredicateIssue::NotMet)
            },
        };
    }
    let mut actual = value;
    for field in predicate.path.split('.') {
        let Some(child) = actual.as_object().and_then(|v| v.get(field)) else {
            return PredicateCheck {
                holds: false,
                comparable: false,
                reason: format!("Field {} is missing", name_repr(&predicate.path)),
                issue: Some(PredicateIssue::MissingField),
            };
        };
        actual = child;
    }
    let expected = &predicate.expected;
    let same_type = matches!(
        (actual, expected),
        (Value::Null, Value::Null)
            | (Value::Bool(_), Value::Bool(_))
            | (Value::Number(_), Value::Number(_))
            | (Value::String(_), Value::String(_))
            | (Value::Array(_), Value::Array(_))
            | (Value::Object(_), Value::Object(_))
    );
    if !same_type || actual.is_array() || actual.is_object() {
        return PredicateCheck {
            holds: false,
            comparable: false,
            reason: format!(
                "Field {} has an incompatible type",
                name_repr(&predicate.path)
            ),
            issue: Some(PredicateIssue::IncompatibleType),
        };
    }
    if actual.is_number()
        && actual
            .as_number()
            .is_some_and(|n| n.to_string().contains(['.', 'e', 'E']))
        && actual.as_f64().is_none_or(|n| !n.is_finite())
    {
        return PredicateCheck {
            holds: false,
            comparable: false,
            reason: format!("Field {} is not finite", name_repr(&predicate.path)),
            issue: Some(PredicateIssue::NonFinite),
        };
    }
    let expression = expressions::predicate_expression(predicate);
    let evaluated = expressions::evaluate_expression_with_limits(&expression, value, limits);
    let Some(holds) = evaluated.value.as_ref().and_then(Value::as_bool) else {
        return PredicateCheck {
            holds: false,
            comparable: false,
            reason: format!(
                "Field {} cannot use {}",
                name_repr(&predicate.path),
                predicate.operator
            ),
            issue: Some(PredicateIssue::Incomparable),
        };
    };
    PredicateCheck {
        holds,
        comparable: true,
        reason: format!(
            "Field {} {} {} {} (observed {})",
            name_repr(&predicate.path),
            predicate.operator,
            py_repr(expected),
            if holds { "holds" } else { "does not hold" },
            py_repr(actual)
        ),
        issue: if holds {
            None
        } else {
            Some(PredicateIssue::NotMet)
        },
    }
}
pub(crate) fn predicate(
    predicate: &Predicate,
    value: &Value,
    limits: &ExecutionLimits,
) -> (bool, String) {
    let check = predicate_check(predicate, value, limits);
    (check.holds, check.reason)
}

/// Produce the same scope entry used by complete evaluations and evidence checks.
pub fn assess_environment(environment: &Environment, context: &Value) -> Result<Value, String> {
    assess_environment_with_limits(environment, context, &ExecutionLimits::default())
}

/// Assess scope with the same resource policy captured by the compiled programme.
pub fn assess_environment_with_limits(
    environment: &Environment,
    context: &Value,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    let context = environment_context(&environment.name, context)?;
    let outcomes: Vec<_> = environment
        .predicates
        .iter()
        .map(|p| predicate(p, &context, limits))
        .collect();
    Ok(entry(
        if outcomes.iter().all(|(ok, _)| *ok) {
            "matched"
        } else {
            "out_of_scope"
        },
        outcomes.into_iter().map(|(_, reason)| reason).collect(),
    ))
}

/// Assess an independent retained record. Bound acquisitions require the complete
/// dependency set; callers must use `assess_evidence_records` for those records.
#[allow(clippy::too_many_arguments)]
pub fn assess_evidence_record(
    program: &Program,
    name: &str,
    record: Option<&Value>,
    instant: DateTime<Utc>,
    context: &Value,
    environment_matched: bool,
    expected_tool_binding_digest: Option<&Value>,
    check_current_binding: bool,
) -> Result<EvidenceVerdict, String> {
    let evidence = program
        .evidence
        .get(name)
        .ok_or_else(|| format!("Unknown evidence {name}"))?;
    if !evidence.bindings.is_empty() {
        return Ok(super::acquisitions::blocked(
            "Bound acquisition assessment requires its parent observations".into(),
        ));
    }
    assess_resolved_evidence_record(
        program,
        name,
        record,
        instant,
        context,
        environment_matched,
        expected_tool_binding_digest,
        check_current_binding,
        true,
        &crate::acquisition::ResolvedInput {
            input: evidence.input.clone(),
            dependencies: json!([]),
        },
    )
}

/// Assess one retained record. Integrity fields are correspondence checks, not signatures.
#[allow(clippy::too_many_arguments)]
pub(super) fn assess_resolved_evidence_record(
    program: &Program,
    name: &str,
    record: Option<&Value>,
    instant: DateTime<Utc>,
    context: &Value,
    environment_matched: bool,
    expected_tool_binding_digest: Option<&Value>,
    check_current_binding: bool,
    check_time: bool,
    resolved: &crate::acquisition::ResolvedInput,
) -> Result<EvidenceVerdict, String> {
    let evidence = program
        .evidence
        .get(name)
        .ok_or_else(|| format!("Unknown evidence {name}"))?;
    let context = environment_context(&evidence.environment, context)?;
    let mut reasons = vec![];
    let mut issues = BTreeSet::new();
    if !environment_matched {
        reasons.push(format!(
            "Environment {} does not match supplied context",
            name_repr(&evidence.environment)
        ));
        issues.insert(AvailabilityIssue::OutOfScope);
    }
    let Some(record) = record.filter(|v| v.is_object()) else {
        reasons.push("No evidence record is available".into());
        issues.insert(AvailabilityIssue::MissingObservation);
        return Ok(EvidenceVerdict {
            entry: json!({"status":"unavailable","reasons":reasons,"availability_issues":issues}),
            complete: false,
            data_available: false,
        });
    };
    if record["status"] == "error" {
        reasons.push(failure::reason(record, &program.limits));
    }
    let tool = evidence
        .tool
        .as_ref()
        .map(|name| {
            program
                .tools
                .get(name)
                .ok_or_else(|| format!("Unknown tool {name}"))
        })
        .transpose()?;
    let tool_name = tool.map(|tool| &tool.name);
    let tool_version = tool.map(|tool| &tool.version);
    let expected = vec![
        ("evidence_id", json!(name)),
        ("source_digest", json!(program.source_digest)),
        ("tool", json!(tool_name)),
        ("tool_version", json!(tool_version)),
        ("evidence_kind", json!(evidence.kind)),
        ("environment", json!(evidence.environment)),
        (
            "environment_fingerprint",
            json!(environment_fingerprint(
                &evidence.environment,
                &context,
                &program.limits
            )?),
        ),
        (
            "input_digest",
            json!(canonical::canonical_digest(
                &resolved.input,
                &program.limits
            )?),
        ),
    ];
    for (key, wanted) in expected {
        if record.get(key) != Some(&wanted) {
            reasons.push(format!(
                "Record {key} does not match the declared evidence request"
            ));
            issues.insert(AvailabilityIssue::InvalidObservation);
        }
    }
    if !evidence.bindings.is_empty()
        && !record.get("dependencies").is_some_and(|actual| {
            canonical::same_json(actual, &resolved.dependencies, &program.limits)
        })
    {
        reasons.push("Record dependencies differ from the selected parent observations".into());
        issues.insert(AvailabilityIssue::InvalidObservation);
    }
    if !evidence.bindings.is_empty() {
        let lineage_time = record["computed_at"]
            .as_str()
            .and_then(|time| validation::parse_time(time).ok());
        let mut expected_time = lineage_time;
        for parent in resolved.dependencies.as_array().into_iter().flatten() {
            let measured = parent["collected_at"]
                .as_str()
                .and_then(|time| validation::parse_time(time).ok());
            expected_time = expected_time
                .zip(measured)
                .map(|(computed, measured)| computed.min(measured));
        }
        if evidence.kind == "scoped_review" {
            expected_time = expected_time
                .zip(scoped_review::review_time(record))
                .map(|(retained, reviewed)| retained.min(reviewed));
        }
        let retained_time = record["collected_at"]
            .as_str()
            .and_then(|time| validation::parse_time(time).ok());
        if expected_time.is_none()
            || expected_time != retained_time
            || (check_time
                && evidence.derivation.is_none()
                && lineage_time.is_some_and(|time| time > instant))
        {
            reasons.push(
                "Bound observation does not preserve the oldest input measurement time".into(),
            );
            issues.insert(AvailabilityIssue::InvalidObservation);
        }
    }
    let binding_digest = record
        .get("tool_binding_digest")
        .cloned()
        .unwrap_or(Value::Null);
    if evidence.derivation.is_some() {
        let expected = crate::acquisition::derivation_digest(program, name)?;
        if binding_digest != expected {
            reasons.push("Derivation identity differs from the declared expression".into());
            issues.insert(AvailabilityIssue::InvalidObservation);
        }
        let expression = evidence.derivation.as_ref().expect("checked derivation");
        let recomputed = expressions::evaluate_expression_with_limits(
            expression,
            &resolved.input,
            &program.limits,
        );
        if recomputed.value.as_ref().is_none_or(|value| {
            !record
                .get("value")
                .is_some_and(|actual| canonical::same_json(actual, value, &program.limits))
        }) {
            reasons.push("Derived value differs from the declared pure expression".into());
            issues.insert(AvailabilityIssue::InvalidObservation);
        }
    }
    let digest_valid = binding_digest.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    });
    if !digest_valid {
        reasons.push("Record requires a tool binding configuration digest".into());
        issues.insert(AvailabilityIssue::InvalidObservation);
    }
    if check_current_binding && Some(&binding_digest) != expected_tool_binding_digest {
        reasons.push("Current operator tool binding differs from the collected binding".into());
        issues.insert(AvailabilityIssue::InvalidObservation);
    }
    let acquisition = json!({"tool":tool_name,"tool_version":tool_version,"input":resolved.input,"context":context});
    let full_request = json!({"evidence_id":name,"environment":evidence.environment,"tool":tool_name,"tool_version":tool_version,"input":resolved.input,"context":context});
    for (key, wanted, message) in [
        (
            "input",
            &resolved.input,
            "Record input differs from the declared evidence input",
        ),
        (
            "context",
            &context,
            "Record context differs from the supplied assessment context",
        ),
        (
            "acquisition_request",
            &acquisition,
            "Record acquisition request differs from the declared request",
        ),
    ] {
        if !record
            .get(key)
            .is_some_and(|actual| canonical::same_json(actual, wanted, &program.limits))
        {
            reasons.push(message.into());
            issues.insert(AvailabilityIssue::InvalidObservation);
        }
    }
    if record
        .get("acquisition_request_digest")
        .and_then(Value::as_str)
        != Some(canonical::canonical_digest(&acquisition, &program.limits)?.as_str())
    {
        reasons
            .push("Record acquisition request digest does not match the declared request".into());
        issues.insert(AvailabilityIssue::InvalidObservation);
    }
    if record.get("request_digest").and_then(Value::as_str)
        != Some(canonical::canonical_digest(&full_request, &program.limits)?.as_str())
    {
        reasons.push("Record request digest does not match the declared evidence request".into());
        issues.insert(AvailabilityIssue::InvalidObservation);
    }
    if record["status"] != "ok" {
        reasons.push("Tool execution did not produce an ok observation".into());
        issues.insert(AvailabilityIssue::ToolError);
    }
    if record
        .get("run_id")
        .and_then(Value::as_str)
        .is_none_or(|s| {
            s.trim_matches(|c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c))
                .is_empty()
        })
    {
        reasons.push("Record requires a nonempty run_id".into());
        issues.insert(AvailabilityIssue::InvalidObservation);
    }
    if let Some(collected) = record
        .get("collected_at")
        .and_then(Value::as_str)
        .and_then(|s| validation::parse_time(s).ok())
    {
        let age = (instant - collected)
            .num_microseconds()
            .ok_or("Observation duration is outside supported range")? as f64
            / 1_000_000.0;
        if check_time && age < 0.0 {
            reasons.push("Observation is dated after the assessment time".into());
            issues.insert(AvailabilityIssue::InvalidObservation);
        } else if check_time && age > evidence.max_age {
            reasons.push(format!(
                "Observation age {}s exceeds max_age {}s",
                general_number(age),
                general_number(evidence.max_age)
            ));
            issues.insert(AvailabilityIssue::StaleObservation);
        }
    } else {
        reasons.push("Record collected_at must be an ISO-8601 timestamp with timezone".into());
        issues.insert(AvailabilityIssue::InvalidObservation);
    }
    let value_bytes: Result<Vec<u8>, String> = record
        .get("value")
        .ok_or_else(|| "Record has no JSON value".to_owned())
        .and_then(|value| canonical::canonical_json_bytes(value, &program.limits))
        .and_then(|bytes| {
            if bytes.len() > program.limits.record_bytes {
                Err("Record value exceeds byte limit".into())
            } else {
                Ok(bytes)
            }
        });
    match value_bytes {
        Ok(bytes) => {
            use sha2::{Digest, Sha256};
            let digest = format!("{:x}", Sha256::digest(bytes));
            if record.get("data_digest").and_then(Value::as_str) != Some(&digest) {
                reasons.push("Record data_digest does not match its JSON value".into());
                issues.insert(AvailabilityIssue::InvalidObservation);
            }
        }
        Err(error) => {
            reasons.push(format!("Invalid observation value: {error}"));
            issues.insert(AvailabilityIssue::InvalidObservation);
        }
    }
    if evidence.kind == "scoped_review" {
        scoped_review::assess(
            evidence,
            record,
            instant,
            check_time,
            &mut reasons,
            &mut issues,
        );
    }
    let data_available = reasons.is_empty();
    let mut complete = data_available;
    let mut failures = vec![];
    let available = if complete {
        let outcomes: Vec<_> = evidence
            .predicates
            .iter()
            .map(|p| predicate_check(p, &record["value"], &program.limits))
            .collect();
        failures = evidence
            .predicates
            .iter()
            .zip(&outcomes)
            .filter_map(|(p, o)| o.issue.map(|issue| json!({"path":p.path,"issue":issue})))
            .collect();
        reasons = outcomes.iter().map(|o| o.reason.clone()).collect();
        complete = outcomes.iter().all(|o| o.comparable);
        if outcomes.iter().any(|o| !o.comparable) {
            issues.insert(AvailabilityIssue::PredicateUnresolved);
        }
        if outcomes.iter().any(|o| o.comparable && !o.holds) {
            issues.insert(AvailabilityIssue::PredicateNotMet);
        }
        outcomes.iter().all(|o| o.holds)
    } else {
        false
    };
    let mut result = json!({"status":if available{"available"}else{"unavailable"},"reasons":reasons,"availability_issues":issues,"tool":tool_name,"tool_version":tool_version,"tool_binding_digest":binding_digest,"run_id":record.get("run_id").unwrap_or(&Value::Null)});
    if !failures.is_empty() {
        result["predicate_failures"] = json!(failures);
    }
    Ok(EvidenceVerdict {
        entry: result,
        complete,
        data_available,
    })
}
