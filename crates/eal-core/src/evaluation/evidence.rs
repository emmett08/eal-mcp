//! Check scoped observation correspondence, freshness and declared predicates.
use super::{
    entry, environment_context, environment_fingerprint, general_number, name_repr, py_repr,
};
use crate::{
    canonical, expressions,
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
}
#[derive(Clone, Debug)]
pub(crate) struct PredicateCheck {
    pub holds: bool,
    pub comparable: bool,
    pub reason: String,
    pub issue: Option<&'static str>,
}

pub(crate) fn predicate_check(predicate: &Predicate, value: &Value) -> PredicateCheck {
    if let Some(expression) = &predicate.expression {
        let evaluated = expressions::evaluate_expression(expression, value);
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
                    Some("missing_field") => "missing_field",
                    Some("invalid_expression") => "invalid_expression",
                    _ => "incompatible_type",
                }),
            };
        };
        let text = match expressions::format_expression(expression) {
            Ok(text) => text,
            Err(reason) => {
                return PredicateCheck {
                    holds: false,
                    comparable: false,
                    reason,
                    issue: Some("invalid_expression"),
                };
            }
        };
        return PredicateCheck {
            holds,
            comparable: true,
            reason: format!("{} {}", text, if holds { "holds" } else { "does not hold" }),
            issue: if holds { None } else { Some("not_met") },
        };
    }
    let mut actual = value;
    for field in predicate.path.split('.') {
        let Some(child) = actual.as_object().and_then(|v| v.get(field)) else {
            return PredicateCheck {
                holds: false,
                comparable: false,
                reason: format!("Field {} is missing", name_repr(&predicate.path)),
                issue: Some("missing_field"),
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
            issue: Some("incompatible_type"),
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
            issue: Some("non_finite"),
        };
    }
    let expression = expressions::predicate_expression(predicate);
    let evaluated = expressions::evaluate_expression(&expression, value);
    let Some(holds) = evaluated.value.as_ref().and_then(Value::as_bool) else {
        return PredicateCheck {
            holds: false,
            comparable: false,
            reason: format!(
                "Field {} cannot use {}",
                name_repr(&predicate.path),
                predicate.operator
            ),
            issue: Some("incomparable"),
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
        issue: if holds { None } else { Some("not_met") },
    }
}
pub(crate) fn predicate(predicate: &Predicate, value: &Value) -> (bool, String) {
    let check = predicate_check(predicate, value);
    (check.holds, check.reason)
}

/// Produce the same scope entry used by complete evaluations and evidence checks.
pub fn assess_environment(environment: &Environment, context: &Value) -> Result<Value, String> {
    let context = environment_context(&environment.name, context)?;
    let outcomes: Vec<_> = environment
        .predicates
        .iter()
        .map(|p| predicate(p, &context))
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

/// Assess one retained record. Integrity fields are correspondence checks, not signatures.
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
    let context = environment_context(&evidence.environment, context)?;
    let mut reasons = vec![];
    let mut issues = BTreeSet::new();
    if !environment_matched {
        reasons.push(format!(
            "Environment {} does not match supplied context",
            name_repr(&evidence.environment)
        ));
        issues.insert("out_of_scope");
    }
    let Some(record) = record.filter(|v| v.is_object()) else {
        reasons.push("No evidence record is available".into());
        issues.insert("missing_observation");
        return Ok(EvidenceVerdict {
            entry: json!({"status":"unavailable","reasons":reasons,"availability_issues":issues}),
            complete: false,
        });
    };
    let tool = program
        .tools
        .get(&evidence.tool)
        .ok_or_else(|| format!("Unknown tool {}", evidence.tool))?;
    let expected = vec![
        ("evidence_id", json!(name)),
        ("source_digest", json!(program.source_digest)),
        ("tool", json!(tool.name)),
        ("tool_version", json!(tool.version)),
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
                &evidence.input,
                &program.limits
            )?),
        ),
    ];
    for (key, wanted) in expected {
        if record.get(key) != Some(&wanted) {
            reasons.push(format!(
                "Record {key} does not match the declared evidence request"
            ));
            issues.insert("invalid_observation");
        }
    }
    let binding_digest = record
        .get("tool_binding_digest")
        .cloned()
        .unwrap_or(Value::Null);
    let digest_valid = binding_digest.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    });
    if !digest_valid {
        reasons.push("Record requires a tool binding configuration digest".into());
        issues.insert("invalid_observation");
    }
    if check_current_binding && Some(&binding_digest) != expected_tool_binding_digest {
        reasons.push("Current operator tool binding differs from the collected binding".into());
        issues.insert("invalid_observation");
    }
    let acquisition = json!({"tool":tool.name,"tool_version":tool.version,"input":evidence.input,"context":context});
    let full_request = json!({"evidence_id":name,"environment":evidence.environment,"tool":tool.name,"tool_version":tool.version,"input":evidence.input,"context":context});
    for (key, wanted, message) in [
        (
            "input",
            &evidence.input,
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
            issues.insert("invalid_observation");
        }
    }
    if record
        .get("acquisition_request_digest")
        .and_then(Value::as_str)
        != Some(canonical::canonical_digest(&acquisition, &program.limits)?.as_str())
    {
        reasons
            .push("Record acquisition request digest does not match the declared request".into());
        issues.insert("invalid_observation");
    }
    if record.get("request_digest").and_then(Value::as_str)
        != Some(canonical::canonical_digest(&full_request, &program.limits)?.as_str())
    {
        reasons.push("Record request digest does not match the declared evidence request".into());
        issues.insert("invalid_observation");
    }
    if record["status"] != "ok" {
        reasons.push("Tool execution did not produce an ok observation".into());
        issues.insert("tool_error");
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
        issues.insert("invalid_observation");
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
        if age < 0.0 {
            reasons.push("Observation is dated after the assessment time".into());
            issues.insert("invalid_observation");
        } else if age > evidence.max_age {
            reasons.push(format!(
                "Observation age {}s exceeds max_age {}s",
                general_number(age),
                general_number(evidence.max_age)
            ));
            issues.insert("stale_observation");
        }
    } else {
        reasons.push("Record collected_at must be an ISO-8601 timestamp with timezone".into());
        issues.insert("invalid_observation");
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
                issues.insert("invalid_observation");
            }
        }
        Err(error) => {
            reasons.push(format!("Invalid observation value: {error}"));
            issues.insert("invalid_observation");
        }
    }
    let mut complete = reasons.is_empty();
    let mut failures = vec![];
    let available = if complete {
        let outcomes: Vec<_> = evidence
            .predicates
            .iter()
            .map(|p| predicate_check(p, &record["value"]))
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
            issues.insert("invalid_observation");
        }
        if outcomes.iter().any(|o| o.comparable && !o.holds) {
            issues.insert("predicate_not_met");
        }
        outcomes.iter().all(|o| o.holds)
    } else {
        false
    };
    let mut result = json!({"status":if available{"available"}else{"unavailable"},"reasons":reasons,"availability_issues":issues,"tool":tool.name,"tool_version":tool.version,"tool_binding_digest":binding_digest,"run_id":record.get("run_id").unwrap_or(&Value::Null)});
    if !failures.is_empty() {
        result["predicate_failures"] = json!(failures);
    }
    Ok(EvidenceVerdict {
        entry: result,
        complete,
    })
}
