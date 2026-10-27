//! Pure admission of an explicitly scoped reviewer record, without certifying its truth.
use crate::capability::canonical_identity;
use serde_json::Value;

fn invalid(message: &str) -> String {
    message.into()
}
fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
/// Expected identity is authored input. Bound record data cannot replace it.
pub fn validate_expectation(input: &Value) -> Result<(), String> {
    for key in ["criterion", "target"] {
        if input[key].as_str().is_none_or(|value| !text(value, 4096)) {
            return Err(invalid(
                "review request requires an explicit criterion and target",
            ));
        }
    }
    if !scoped(&input["scope"]) {
        return Err(invalid(
            "review request requires an explicit nonempty scope",
        ));
    }
    Ok(())
}

fn scoped(value: &Value) -> bool {
    value.as_object().is_some_and(|fields| !fields.is_empty())
        || value.as_str().is_some_and(|scope| text(scope, 4096))
}

pub fn validate(input: &Value, value: &Value) -> Result<String, String> {
    validate_expectation(input)?;
    let object = value
        .as_object()
        .ok_or_else(|| invalid("retained review value must be an object"))?;
    let required = [
        "criterion",
        "target",
        "scope",
        "reviewer",
        "verdict",
        "reasons",
        "reviewed_at",
    ];
    if required.iter().any(|field| !object.contains_key(*field))
        || object.keys().any(|field| {
            !required.contains(&field.as_str())
                && !["provenance", "evidence_refs"].contains(&field.as_str())
        })
    {
        return Err(invalid(
            "retained review requires criterion, target, scope, reviewer, verdict, reasons and reviewed_at; accepted booleans are not scoped reviews",
        ));
    }
    for field in ["criterion", "target", "scope"] {
        if canonical_identity(&value[field])? != canonical_identity(&input[field])? {
            return Err(invalid(
                "retained review criterion, target or scope differs from the exact request",
            ));
        }
    }
    if value["reviewer"]
        .as_str()
        .is_none_or(|reviewer| !text(reviewer, 1024))
    {
        return Err(invalid(
            "retained review requires a nonempty reviewer identity",
        ));
    }
    if !matches!(
        value["verdict"].as_str(),
        Some("supported" | "rejected" | "inconclusive")
    ) {
        return Err(invalid(
            "retained review verdict must be supported, rejected or inconclusive",
        ));
    }
    if value["reasons"].as_array().is_none_or(|reasons| {
        reasons.is_empty()
            || reasons.len() > 64
            || reasons
                .iter()
                .any(|reason| reason.as_str().is_none_or(|reason| !text(reason, 4096)))
    }) {
        return Err(invalid(
            "retained review requires 1 to 64 nonempty bounded reasons",
        ));
    }
    let reviewed_at = value["reviewed_at"]
        .as_str()
        .ok_or_else(|| invalid("retained review requires original reviewed_at"))?;
    let time = chrono::DateTime::parse_from_rfc3339(reviewed_at)
        .map_err(|_| "reviewed_at requires a retained UTC RFC3339 timestamp".to_owned())?;
    use chrono::{Datelike, Timelike};
    if !(20..=30).contains(&reviewed_at.len())
        || reviewed_at.as_bytes().get(10) != Some(&b'T')
        || !reviewed_at.ends_with('Z')
        || time.year() < 1
        || time.nanosecond() >= 1_000_000_000
    {
        return Err("reviewed_at requires a retained UTC RFC3339 timestamp".into());
    }
    Ok(reviewed_at.to_owned())
}
