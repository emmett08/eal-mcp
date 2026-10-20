use super::{exact_keys, same_json};
use crate::contracts::validate_anchor;
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Diagnose an authored answer without filling fields or changing EAL statuses.
pub fn validate_answer(
    anchor: &Value,
    finding: &Value,
    answer: &Value,
) -> Result<Vec<Value>, String> {
    let task = validate_anchor(anchor)?;
    if !exact_keys(
        answer,
        &[
            "task_digest",
            "finding_id",
            "claims",
            "scope",
            "conclusion",
            "qualification",
        ],
    ) {
        return Ok(vec![diagnostic(
            "answer",
            "Answer requires exactly its declared fields",
        )]);
    }
    let mut errors = Vec::new();
    for name in ["task_digest", "finding_id"] {
        let expected = finding
            .get(name)
            .ok_or("Finding lacks its trusted identity")?;
        if &answer[name] != expected
            || (name == "task_digest" && answer[name] != anchor["task_digest"])
        {
            errors.push(diagnostic(
                name,
                "Answer differs from the trusted task finding",
            ));
        }
    }
    if answer["qualification"] != "conditional_on_registered_argument" {
        errors.push(diagnostic(
            "qualification",
            "Assessment remains conditional on the registered argument; no proof of prose follows",
        ));
    }
    let expected_scope =
        json!({"context":task.requirements["context"], "requirements":task.requirements});
    match same_json(&answer["scope"], &expected_scope) {
        Ok(false) => errors.push(diagnostic(
            "scope",
            "Answer must retain the complete requested context and profile requirements",
        )),
        Err(_) => errors.push(diagnostic("scope", "Answer scope requires finite JSON")),
        Ok(true) => {}
    }
    let rows = finding
        .get("assessments")
        .and_then(Value::as_object)
        .ok_or("Finding assessments must be an object")?;
    let selected = task
        .claims
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let claims = answer["claims"].as_object();
    if !claims.is_some_and(|claims| {
        claims.keys().map(String::as_str).collect::<BTreeSet<_>>() == selected
    }) {
        errors.push(diagnostic(
            "claims",
            "Answer must include exactly the selected claims",
        ));
    } else {
        for claim in &task.claims {
            let row = rows.get(claim);
            let mut wanted = serde_json::Map::new();
            for key in ["assessment_id", "status", "assessed_at"] {
                wanted.insert(
                    key.into(),
                    match row {
                        None => Value::Null,
                        Some(row) => {
                            let value = row
                                .get(key)
                                .ok_or("Finding assessment lacks identity, status or time")?;
                            super::digest(value)?;
                            value.clone()
                        }
                    },
                );
            }
            let actual = &answer["claims"][claim];
            let path = format!("claims.{claim}");
            let matches = if actual.is_object() {
                same_json(actual, &Value::Object(wanted))
            } else {
                Ok(false)
            };
            match matches {
                Ok(false) => errors.push(diagnostic(
                    &path,
                    "Claim result must match the persisted assessment identity, status and time",
                )),
                Err(_) => errors.push(diagnostic(&path, "Claim result requires finite JSON")),
                Ok(true) => {}
            }
        }
    }
    let check = finding
        .get("check")
        .ok_or("Finding lacks its correspondence check")?;
    let ok = check
        .get("ok")
        .and_then(Value::as_bool)
        .ok_or("Finding check requires a Boolean result")?;
    let conclusion = if !ok {
        if matches!(
            check["code"].as_str(),
            Some("source_changed" | "context_mismatch" | "workload_mismatch" | "criteria_mismatch")
        ) {
            "out_of_scope"
        } else {
            "unresolved"
        }
    } else if rows.keys().map(String::as_str).collect::<BTreeSet<_>>() == selected {
        "assessed"
    } else {
        "unresolved"
    };
    if answer["conclusion"] != conclusion {
        errors.push(diagnostic("conclusion", "Conclusion must distinguish existing EAL assessments from scope mismatch and unavailable results"));
    }
    Ok(errors)
}

fn diagnostic(path: &str, detail: &str) -> Value {
    json!({"code":"answer_mismatch", "path":path, "detail":detail})
}
