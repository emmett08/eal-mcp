//! Pure acquisition dependencies and request construction from retained observations.
use crate::{canonical, semantic::Program};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Expand selected acquisitions once, with every parent preceding its consumers.
/// Argument/objection cycles are unrelated and are not traversed here.
pub fn dependency_order(program: &Program, selected: &[String]) -> Result<Vec<String>, String> {
    let mut states = BTreeMap::<String, u8>::new();
    let mut ordered = Vec::new();
    for root in selected {
        let mut stack = vec![(root.clone(), false)];
        while let Some((name, exiting)) = stack.pop() {
            if exiting {
                states.insert(name.clone(), 2);
                ordered.push(name);
                continue;
            }
            match states.get(&name) {
                Some(2) => continue,
                Some(1) => return Err(format!("Acquisition dependency cycle at {name}")),
                _ => {}
            }
            let evidence = program
                .evidence
                .get(&name)
                .ok_or_else(|| format!("Unknown acquisition evidence {name}"))?;
            states.insert(name.clone(), 1);
            if states.len() > program.limits.declarations {
                return Err("Acquisition dependencies exceed declaration budget".into());
            }
            stack.push((name, true));
            for binding in evidence.bindings.iter().rev() {
                stack.push((binding.evidence.clone(), false));
            }
        }
    }
    Ok(ordered)
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedInput {
    pub input: Value,
    /// Exact original acquisition identities; repeated uses are one observation.
    pub dependencies: Value,
}

/// Substitute explicit pointers into the authored input. Only the normalised
/// observation value is readable; executor metadata cannot become evidence data.
/// Correspondence, scope and freshness are checked separately by assessment.
pub fn resolve_input(
    program: &Program,
    name: &str,
    records: &Value,
) -> Result<ResolvedInput, String> {
    let evidence = program
        .evidence
        .get(name)
        .ok_or_else(|| format!("Unknown acquisition evidence {name}"))?;
    let mut input = evidence.input.clone();
    let mut dependencies = Vec::new();
    let mut seen = BTreeSet::new();
    for binding in &evidence.bindings {
        let parent = records
            .get(&binding.evidence)
            .filter(|record| record.is_object())
            .ok_or_else(|| format!("Dependency {} has no observation", binding.evidence))?;
        if parent["status"] != "ok" {
            return Err(format!(
                "Dependency {} did not produce an ok observation",
                binding.evidence
            ));
        }
        let value = parent
            .get("value")
            .and_then(|value| value.pointer(&binding.source))
            .ok_or_else(|| {
                format!(
                    "Dependency {} has no value at {}",
                    binding.evidence, binding.source
                )
            })?
            .clone();
        let target = input.pointer_mut(&binding.target).ok_or_else(|| {
            format!(
                "Acquisition {name} has no input destination {}",
                binding.target
            )
        })?;
        *target = value;
        if canonical::canonical_json_bytes(&input, &program.limits)?.len()
            > program.limits.record_bytes
        {
            return Err("Resolved acquisition input exceeds the byte budget".into());
        }
        if seen.insert(binding.evidence.clone()) {
            dependencies.push(dependency_identity(&binding.evidence, parent));
        }
    }
    canonical::canonical_json_bytes(&input, &program.limits)?;
    Ok(ResolvedInput {
        input,
        dependencies: json!(dependencies),
    })
}

/// Fields binding the selected value to its measured origin. Presentation and
/// ingestion metadata are deliberately excluded so reuse never resets the clock.
fn dependency_identity(name: &str, record: &Value) -> Value {
    json!({
        "evidence_id": name,
        "run_id": record["run_id"],
        "origin_run_id": record.get("origin_run_id").unwrap_or(&record["run_id"]),
        "source_digest": record["source_digest"],
        "data_digest": record["data_digest"],
        "input_digest": record["input_digest"],
        "acquisition_request_digest": record["acquisition_request_digest"],
        "tool_binding_digest": record["tool_binding_digest"],
        "collected_at": record["collected_at"]
    })
}

/// The pure expression implementation is language-owned, never an operator tool.
pub fn derivation_digest(program: &Program, name: &str) -> Result<String, String> {
    let evidence = program.evidence.get(name).ok_or("Unknown derivation")?;
    let expression = evidence
        .derivation
        .as_ref()
        .ok_or("Evidence is not a derivation")?;
    canonical::canonical_digest(
        &json!({"contract":"EAL/expression/1", "expression":expression}),
        &program.limits,
    )
}
