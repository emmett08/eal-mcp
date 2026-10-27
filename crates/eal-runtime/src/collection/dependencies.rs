//! Dependency-ready scheduling and retention of the original input measurement age.
use super::EvidenceRuntime;
use crate::{lease::AcquisitionLease, store::utc_now};
use eal_core::{
    acquisition::ResolvedInput,
    evaluation,
    semantic::{Evidence, Program},
    validation,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, sync::Arc};

impl EvidenceRuntime {
    pub(super) fn collect_dependencies(
        &self,
        program: &Program,
        context: &Value,
        names: &[String],
        lease: Option<Arc<AcquisitionLease>>,
    ) -> Result<Value, String> {
        let mut records = json!({});
        let mut pending = names
            .iter()
            .filter(|name| records.get(*name).is_none())
            .cloned()
            .collect::<Vec<_>>();
        while !pending.is_empty() {
            let ready = pending
                .iter()
                .filter(|name| {
                    program.evidence[*name]
                        .bindings
                        .iter()
                        .all(|binding| records.get(&binding.evidence).is_some())
                })
                .cloned()
                .collect::<Vec<_>>();
            if ready.is_empty() {
                return Err("Acquisition dependencies cannot make progress".into());
            }
            let batch = self.scheduler.run(
                &ready,
                |name| {
                    let evidence = &program.evidence[name];
                    if evidence.derivation.is_some() {
                        return true;
                    }
                    evidence
                        .tool
                        .as_ref()
                        .and_then(|name| program.tools.get(name))
                        .is_some_and(|tool| {
                            self.registry
                                .binding_for(&tool.name, &tool.version)
                                .is_ok_and(|binding| binding.parallel_safe)
                        })
                },
                |name| self.collect_one(program, name, context, &records, lease.as_deref()),
            )?;
            let completed = ready.iter().collect::<BTreeSet<_>>();
            pending.retain(|name| !completed.contains(name));
            for (name, record) in batch {
                records[&name] = record;
            }
        }
        Ok(records)
    }

    pub(super) fn resolve_dependencies(
        &self,
        program: &Program,
        name: &str,
        records: &Value,
        context: &Value,
    ) -> Result<ResolvedInput, String> {
        let evidence = &program.evidence[name];
        let parents = evidence
            .bindings
            .iter()
            .map(|binding| binding.evidence.clone())
            .collect::<Vec<_>>();
        if !parents.is_empty() {
            let bindings = self.current_binding_digests(program);
            let checked = evaluation::assess_acquisition_inputs(
                program,
                &parents,
                records,
                validation::parse_time(&utc_now())?,
                context,
                Some(&bindings),
            )?;
            for parent in parents {
                if !checked
                    .get(&parent)
                    .is_some_and(|verdict| verdict.data_available)
                {
                    return Err(format!(
                        "Dependency {parent} is unavailable, stale or invalid"
                    ));
                }
            }
        }
        eal_core::acquisition::resolve_input(program, name, records)
    }
}

/// A computation creates a new result, not a new measurement of its inputs.
/// Its evidence age cannot become younger than the oldest measurement used.
pub(super) fn retain_measurement_time(
    record: &mut Value,
    parents: &Value,
    evidence: &Evidence,
) -> Result<(), String> {
    let computed = record["collected_at"]
        .as_str()
        .ok_or("Missing observation time")?
        .to_owned();
    let mut earliest = validation::parse_time(&computed)?;
    let mut original = computed.clone();
    for binding in &evidence.bindings {
        let value = parents[&binding.evidence]["collected_at"]
            .as_str()
            .ok_or("Missing dependency observation time")?;
        let instant = validation::parse_time(value)?;
        if instant < earliest {
            earliest = instant;
            original = value.to_owned();
        }
    }
    // A review is measured when the reviewer issued it, not when its retained
    // envelope was imported. Invalid review shapes remain raw observations for
    // the core validator to reject; they must not abort unrelated acquisitions.
    if evidence.kind == "scoped_review"
        && let Some(reviewed) = record["value"]["reviewed_at"].as_str()
        && let Ok(instant) = validation::parse_time(reviewed)
        && instant < earliest
    {
        original = reviewed.to_owned();
    }
    if !evidence.bindings.is_empty() || evidence.kind == "scoped_review" {
        record["computed_at"] = json!(computed);
        record["collected_at"] = json!(original);
    }
    Ok(())
}
