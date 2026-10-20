use super::{
    API_METHOD, CRITERIA, WORKLOAD_FIELDS, exact_keys, integer, knowledge_with_host, same_json,
};
use crate::{
    contracts::{ServerSpec, validate_anchor},
    host::HostPaths,
};
use eal_core::{expressions::numbers, methods, semantic::Program};
use serde_json::{Value, json};
use std::{cmp::Ordering, collections::BTreeSet};

pub fn check_task(server: &ServerSpec, anchor: &Value) -> Result<Value, String> {
    check_task_with_host(server, anchor, &HostPaths::current()?)
}

/// Read and validate current identities without acquisition or reasoning.
pub fn check_task_with_host(
    server: &ServerSpec,
    anchor: &Value,
    host: &HostPaths,
) -> Result<Value, String> {
    let task = validate_anchor(anchor)?;
    let knowledge = match knowledge_with_host(server, host) {
        Ok(knowledge) => knowledge,
        Err(_) => {
            return Ok(result(
                "source_changed",
                "Current registered source or host configuration is unavailable or invalid",
            ));
        }
    };
    if knowledge.service.method_registry().fingerprint() != anchor["method_registry_fingerprint"] {
        return Ok(result(
            "criteria_mismatch",
            "Installed reasoning-method registry changed after task pinning",
        ));
    }
    let entry = match knowledge.catalogue.get(&task.entry_id, true) {
        Ok(entry) => entry,
        Err(error) if error.starts_with("Unknown catalogue entry or source revision ") => {
            return Ok(result(
                "evidence_unavailable",
                "Registered entry is unavailable",
            ));
        }
        Err(_) => {
            return Ok(result(
                "source_changed",
                "Current registered source or host configuration is unavailable or invalid",
            ));
        }
    };
    if entry["source_digest"] != anchor["source_digest"] {
        return Ok(result(
            "source_changed",
            "Registered source changed after task pinning",
        ));
    }
    if entry["method_registry_fingerprint"] != anchor["method_registry_fingerprint"] {
        return Ok(result(
            "criteria_mismatch",
            "Installed reasoning-method registry changed after task pinning",
        ));
    }
    let requirements = &task.requirements;
    if !same_json(&entry["context"], &anchor["registered_context"])?
        || !same_json(&entry["context"], &requirements["context"])?
    {
        return Ok(result(
            "context_mismatch",
            "Registered context differs from the pinned and requested context",
        ));
    }
    let available = entry["claims"]
        .as_array()
        .ok_or("Registered claims must be an array")?
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    if task
        .claims
        .iter()
        .any(|claim| !available.contains(claim.as_str()))
    {
        return Ok(result(
            "criteria_mismatch",
            "Selected task claims are absent from the registered entry",
        ));
    }
    if task.profile == "registered_claims/1" {
        return Ok(passed());
    }
    let workload = &requirements["workload"];
    let criteria = &requirements["criteria"];
    let context = json!({"service":workload["service"],"build_id":workload["build_id"],"dataset":workload["dataset"]});
    if !same_json(&requirements["context"], &context)? {
        return Ok(result(
            "context_mismatch",
            "Requested context differs from the requested API workload",
        ));
    }
    if CRITERIA.iter().any(|(key, expected)| {
        numbers::compare(&criteria[*key], &json!(expected)) != Some(Ordering::Equal)
    }) {
        return Ok(result(
            "criteria_mismatch",
            "Requested criteria differ from the installed API-load method's fixed limits",
        ));
    }
    let Some(installed) = knowledge.service.method_registry().get(API_METHOD) else {
        return Ok(result(
            "criteria_mismatch",
            "API-load reasoning method is not installed",
        ));
    };
    // Executor identity is independently pinned by the complete registry digest.
    // Compare the full compiled semantic contract across the supervised adapter.
    let expected_registry = methods::engineering_registry();
    let mut expected = expected_registry.get(API_METHOD).unwrap().describe();
    let mut actual = installed.describe();
    for description in [&mut expected, &mut actual] {
        let object = description.as_object_mut().unwrap();
        object.remove("execution");
        object.remove("executor_identity");
    }
    if !same_json(&actual, &expected)? {
        return Ok(result(
            "criteria_mismatch",
            "Installed API method differs from the profile contract",
        ));
    }
    let source = entry["source"]
        .as_str()
        .ok_or("Registered source must be text")?;
    let bindings = (|| -> Result<Value, String> {
        let program = knowledge.service.parse(source)?;
        for claim in &task.claims {
            let plan = knowledge.service.plan(source, claim)?;
            let check = check_plan(&program, &plan, workload, criteria, requirements)?;
            if check["ok"] != true {
                return Ok(check);
            }
        }
        Ok(passed())
    })();
    Ok(bindings.unwrap_or_else(|_| {
        result(
            "criteria_mismatch",
            "Selected API formal bindings cannot be checked",
        )
    }))
}

fn check_plan(
    program: &Program,
    plan: &Value,
    workload: &Value,
    criteria: &Value,
    requirements: &Value,
) -> Result<Value, String> {
    let evidence = plan["evidence_ids"]
        .as_array()
        .ok_or("Plan evidence must be an array")?;
    let reasoning = plan["dependencies"]["reasoning"]
        .as_array()
        .ok_or("Plan reasoning must be an array")?;
    if evidence.is_empty() || reasoning.is_empty() {
        return Ok(result(
            "criteria_mismatch",
            "Selected API claim lacks evidence or reasoning bindings",
        ));
    }
    for name in evidence {
        let declaration = program
            .evidence
            .get(name.as_str().ok_or("Invalid evidence identifier")?)
            .ok_or("Missing evidence declaration")?;
        if declaration.kind != "api_report" || !declaration.input.is_object() {
            return Ok(result(
                "workload_mismatch",
                "Selected API evidence lacks a report workload",
            ));
        }
        let mut fields = WORKLOAD_FIELDS.to_vec();
        fields.push("report_sha256");
        if !exact_keys(&declaration.input, &fields)
            || !declaration.input["report_sha256"]
                .as_str()
                .is_some_and(|digest| {
                    digest.len() == 64
                        && digest
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                })
        {
            return Ok(result(
                "workload_mismatch",
                "API evidence must pin the identified report bytes",
            ));
        }
        if workload
            .as_object()
            .ok_or("Workload must be an object")?
            .iter()
            .any(|(key, expected)| {
                let actual = &declaration.input[key];
                if actual.is_number() && expected.is_number() {
                    integer(actual) != integer(expected)
                        || numbers::compare(actual, expected) != Some(Ordering::Equal)
                } else {
                    actual != expected
                }
            })
        {
            return Ok(result(
                "workload_mismatch",
                "Evidence input differs from the complete requested workload",
            ));
        }
        if numbers::compare(
            &json!(declaration.max_age),
            &criteria["evidence_max_age_seconds"],
        ) != Some(Ordering::Equal)
        {
            return Ok(result(
                "criteria_mismatch",
                "Evidence freshness differs from the requested contract",
            ));
        }
    }
    for name in reasoning {
        if program
            .reasoning
            .get(name.as_str().ok_or("Invalid reasoning identifier")?)
            .ok_or("Missing reasoning declaration")?
            .method
            != API_METHOD
        {
            return Ok(result(
                "criteria_mismatch",
                "Selected API claim uses another reasoning method",
            ));
        }
    }
    let claim = plan["claim"].as_str().ok_or("Plan claim must be text")?;
    let output = requirements["claim_outputs"][claim]
        .as_str()
        .ok_or("Missing claim output binding")?;
    let mut direct = false;
    for name in plan["dependencies"]["arguments"]
        .as_array()
        .ok_or("Plan arguments must be an array")?
    {
        let argument = program
            .arguments
            .get(name.as_str().ok_or("Invalid argument identifier")?)
            .ok_or("Missing argument declaration")?;
        if argument.conclusion != claim {
            continue;
        }
        direct = true;
        let reasoning = program
            .reasoning
            .get(&argument.reasoning)
            .ok_or("Missing argument reasoning")?;
        if !reasoning.predicates.iter().any(|predicate| {
            predicate.path == output && predicate.operator == "==" && predicate.expected == true
        }) {
            return Ok(result(
                "criteria_mismatch",
                "Selected claim does not bind its requested API method output",
            ));
        }
    }
    if !direct {
        return Ok(result(
            "criteria_mismatch",
            "Selected claim does not bind its requested API method output",
        ));
    }
    Ok(passed())
}

fn passed() -> Value {
    json!({"ok":true,"code":Value::Null,"detail":"Pinned registered source and profile correspond to the task"})
}
fn result(code: &str, detail: &str) -> Value {
    json!({"ok":false,"code":code,"detail":detail})
}
