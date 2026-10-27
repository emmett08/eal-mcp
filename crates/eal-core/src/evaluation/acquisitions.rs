//! Assess an acquisition closure without equating negative data with failed acquisition.
use super::evidence::assess_resolved_evidence_record;
use super::{EvidenceVerdict, assess_environment};
use crate::semantic::Program;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

/// Assess a complete acquisition closure in dependency order. A valid measured
/// negative remains usable input; missing, stale or mismatched observations do not.
pub fn assess_evidence_records(
    program: &Program,
    selected: &[String],
    records: &Value,
    instant: DateTime<Utc>,
    context: &Value,
    binding_digests: Option<&Value>,
) -> Result<std::collections::BTreeMap<String, EvidenceVerdict>, String> {
    assess_record_set(
        program,
        selected,
        records,
        instant,
        context,
        binding_digests,
        true,
    )
}

/// Acquisition consumes well-formed observations without deciding their current
/// freshness. Historical calculations preserve age; explicit-time assessment
/// determines whether those observations support a current or historical claim.
pub fn assess_acquisition_inputs(
    program: &Program,
    selected: &[String],
    records: &Value,
    instant: DateTime<Utc>,
    context: &Value,
    binding_digests: Option<&Value>,
) -> Result<std::collections::BTreeMap<String, EvidenceVerdict>, String> {
    assess_record_set(
        program,
        selected,
        records,
        instant,
        context,
        binding_digests,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn assess_record_set(
    program: &Program,
    selected: &[String],
    records: &Value,
    instant: DateTime<Utc>,
    context: &Value,
    binding_digests: Option<&Value>,
    check_time: bool,
) -> Result<std::collections::BTreeMap<String, EvidenceVerdict>, String> {
    let mut results = std::collections::BTreeMap::<String, EvidenceVerdict>::new();
    for name in crate::acquisition::dependency_order(program, selected)? {
        let evidence = &program.evidence[&name];
        let unavailable = evidence.bindings.iter().find(|binding| {
            results
                .get(&binding.evidence)
                .is_none_or(|verdict| !verdict.data_available)
        });
        let verdict = if let Some(binding) = unavailable {
            blocked(format!(
                "Dependency {} is unavailable, stale or invalid",
                binding.evidence
            ))
        } else {
            match crate::acquisition::resolve_input(program, &name, records) {
                Ok(resolved) => {
                    let environment = program
                        .environments
                        .get(&evidence.environment)
                        .ok_or("Unknown evidence environment")?;
                    let matched = assess_environment(environment, context)?["status"] == "matched";
                    assess_resolved_evidence_record(
                        program,
                        &name,
                        records.get(&name),
                        instant,
                        context,
                        matched,
                        binding_digests.and_then(|v| v.get(&name)),
                        binding_digests.is_some(),
                        check_time,
                        &resolved,
                    )?
                }
                Err(reason) => blocked(reason),
            }
        };
        results.insert(name, verdict);
    }
    Ok(results)
}

pub(super) fn blocked(reason: String) -> EvidenceVerdict {
    EvidenceVerdict {
        entry: json!({"status":"unavailable", "reasons":[reason],
            "availability_issues":["dependency_unavailable"]}),
        complete: false,
        data_available: false,
    }
}
