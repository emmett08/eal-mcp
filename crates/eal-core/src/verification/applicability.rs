use super::evaluate::obligation;
use super::{Applicability, Context, Contract, ObligationResult, ObservationStatus, Role, Status};
use crate::{ExecutionLimits, canonical, validation::parse_time};
use serde_json::json;

pub(super) fn evaluate(
    source: &str,
    contract: &Contract,
    context: &Context,
    limits: &ExecutionLimits,
) -> Result<Vec<ObligationResult>, String> {
    let Some(expected) = contract
        .applicability
        .iter()
        .find(|item| item.observation_id == source)
    else {
        return Ok(vec![obligation(
            &format!("applicability:{source}"),
            Role::Applicability,
            Status::Unknown,
            "A relied-upon source lacks explicit identity, unit, scope and time obligations",
            json!({"observation_id":source}),
        )]);
    };
    let Some(observation) = context
        .observations
        .get(source)
        .filter(|item| item.status == ObservationStatus::Available)
    else {
        return Ok(vec![obligation(
            &format!("applicability:{source}"),
            Role::Applicability,
            Status::Unknown,
            "Applicability requires an available acquired observation",
            json!({"observation_id":source}),
        )]);
    };
    let mut results = vec![];
    for (facet, pointer, wanted) in [
        (
            "identity",
            &expected.identity_pointer,
            expected.identity.clone(),
        ),
        ("unit", &expected.unit_pointer, json!(expected.unit)),
        ("scope", &expected.scope_pointer, expected.scope.clone()),
    ] {
        let actual = observation.value.pointer(pointer);
        let status = match actual {
            _ if !specified(&wanted) => Status::Unknown,
            None => Status::Unknown,
            Some(actual) if !specified(actual) => Status::Unknown,
            Some(actual) if canonical::same_json(actual, &wanted, limits) => Status::Pass,
            Some(_) => Status::Fail,
        };
        let reason = match status {
            Status::Unknown => "The source omitted a required applicability facet",
            Status::Fail => "The source applicability facet differs from the required target",
            _ => "The source applicability facet matches the required target",
        };
        results.push(obligation(&format!("applicability:{source}:{facet}"),Role::Applicability,status,reason,
            json!({"observation_id":source,"facet":facet,"pointer":pointer,"expected":wanted,"actual":actual})));
    }
    // Units are compared exactly; a source must perform and identify any allowed
    // conversion before it can be used in a different unit.
    results.push(time(source, expected, observation, context));
    Ok(results)
}

fn specified(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::String(value) => !value.trim().is_empty(),
        serde_json::Value::Object(value) => !value.is_empty() && value.values().all(specified),
        serde_json::Value::Array(value) => !value.is_empty() && value.iter().all(specified),
        _ => true,
    }
}
fn time(
    source: &str,
    expected: &Applicability,
    observation: &super::Observation,
    context: &Context,
) -> ObligationResult {
    let id = format!("applicability:{source}:time");
    let details = json!({"observation_id":source,"observed_at":observation.observed_at,"assessed_at":context.assessed_at,
        "expected_from":expected.valid_from,"expected_until":expected.valid_until,"max_age_seconds":expected.max_age_seconds,
        "actual_from":observation.value.pointer(&expected.valid_from_pointer),"actual_until":observation.value.pointer(&expected.valid_until_pointer),"source_digest":observation.source_digest});
    let parse = || {
        Some((
            parse_time(&context.assessed_at).ok()?,
            parse_time(observation.observed_at.as_deref()?).ok()?,
            parse_time(&expected.valid_from).ok()?,
            parse_time(&expected.valid_until).ok()?,
            parse_time(
                observation
                    .value
                    .pointer(&expected.valid_from_pointer)?
                    .as_str()?,
            )
            .ok()?,
            parse_time(
                observation
                    .value
                    .pointer(&expected.valid_until_pointer)?
                    .as_str()?,
            )
            .ok()?,
        ))
    };
    let Some((now, observed, from, until, actual_from, actual_until)) = parse() else {
        return obligation(
            &id,
            Role::Applicability,
            Status::Unknown,
            "Required original observation time or applicability interval is missing or invalid",
            details,
        );
    };
    if observation
        .source_digest
        .as_deref()
        .is_none_or(|digest| digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return obligation(
            &id,
            Role::Applicability,
            Status::Unknown,
            "Original source digest is missing or invalid",
            details,
        );
    }
    let age = now.signed_duration_since(observed);
    let max_age = i64::try_from(expected.max_age_seconds).unwrap_or(i64::MAX);
    let holds = from < until
        && actual_from < actual_until
        && actual_from <= from
        && actual_until >= until
        && now >= from
        && now < until
        && observed >= from
        && observed < until
        && observed >= actual_from
        && observed < actual_until
        && observed <= now
        && chrono::TimeDelta::try_seconds(max_age).is_some_and(|allowance| age <= allowance);
    obligation(
        &id,
        Role::Applicability,
        if holds { Status::Pass } else { Status::Fail },
        if holds {
            "Original source age and the containing half-open applicability interval satisfy the required target"
        } else {
            "The source is stale, future-dated, expired or does not cover the required interval"
        },
        details,
    )
}
