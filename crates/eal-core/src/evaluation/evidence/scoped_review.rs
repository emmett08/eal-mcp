//! Review identity and original clock are admission conditions, not support predicates.
use crate::{semantic::Evidence, validation};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn review_time(record: &Value) -> Option<DateTime<Utc>> {
    record["value"]["reviewed_at"]
        .as_str()
        .and_then(|time| validation::parse_time(time).ok())
}

pub(super) fn assess(
    evidence: &Evidence,
    record: &Value,
    instant: DateTime<Utc>,
    check_time: bool,
    reasons: &mut Vec<String>,
    issues: &mut BTreeSet<&'static str>,
) {
    // Identity comes from the declaration, never from bound reviewer output.
    let reviewed =
        match eal_collector_contracts::scoped_review::validate(&evidence.input, &record["value"]) {
            Ok(time) => validation::parse_time(&time),
            Err(reason) => Err(reason),
        };
    let reviewed = match reviewed {
        Ok(reviewed) => reviewed,
        Err(reason) => {
            reasons.push(format!("Invalid scoped review: {reason}"));
            issues.insert("invalid_observation");
            return;
        }
    };
    if record["collected_at"]
        .as_str()
        .and_then(|time| validation::parse_time(time).ok())
        .is_none_or(|time| time > reviewed)
    {
        reasons
            .push("Scoped review observation must preserve the original reviewed_at clock".into());
        issues.insert("invalid_observation");
    }
    if check_time {
        let Some(microseconds) = (instant - reviewed).num_microseconds() else {
            reasons.push("Scoped review age is outside the supported range".into());
            issues.insert("invalid_observation");
            return;
        };
        let age = microseconds as f64 / 1_000_000.0;
        if age < 0.0 {
            reasons.push("Scoped review is dated after the assessment time".into());
            issues.insert("invalid_observation");
        } else if age > evidence.max_age {
            reasons.push(format!(
                "Scoped review age {age}s exceeds max_age {}s",
                evidence.max_age
            ));
            issues.insert("stale_observation");
        }
    }
}
