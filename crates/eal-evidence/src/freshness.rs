//! Original source-contract freshness stays separate from each consumer's age limit.
use crate::{
    execution::{SourceRecord, StepResult, StepStatus},
    plan::EvidenceExecutionPlan,
};
use std::collections::{BTreeMap, BTreeSet};
pub(crate) fn diagnostic<'a>(
    maximum: Option<u64>,
    times: impl Iterator<Item = &'a str>,
    fallback: Option<&str>,
    now: &str,
) -> Option<String> {
    let maximum = maximum?;
    let times = times.collect::<Vec<_>>();
    let times = if times.is_empty() {
        fallback.into_iter().collect()
    } else {
        times
    };
    let current = chrono::DateTime::parse_from_rfc3339(now).ok()?;
    if times.iter().any(|at| {
        chrono::DateTime::parse_from_rfc3339(at)
            .ok()
            .is_none_or(|at| {
                at > current
                    || current
                        .signed_duration_since(at)
                        .to_std()
                        .is_ok_and(|age| age > std::time::Duration::from_secs(maximum))
            })
    }) {
        Some("source observation does not meet its original acquisition freshness contract".into())
    } else {
        None
    }
}
pub(crate) fn enforce(
    plan: &EvidenceExecutionPlan,
    steps: &mut BTreeMap<String, StepResult>,
    sources: &BTreeMap<String, SourceRecord>,
    owned: &BTreeMap<String, Vec<String>>,
    now: &str,
) {
    let mut stale = BTreeSet::new();
    for contract in &plan.acquisitions {
        let Some(step) = steps.get_mut(&contract.id) else {
            continue;
        };
        if let Some(message) = diagnostic(
            contract.max_age_seconds,
            owned
                .get(&contract.id)
                .into_iter()
                .flatten()
                .filter_map(|id| sources.get(id))
                .map(|record| record.observed_at.as_str()),
            step.observed_at.as_deref(),
            now,
        ) {
            stale.insert(contract.id.clone());
            if step.value.is_some() {
                step.value = None;
                step.status = StepStatus::Unavailable;
                step.diagnostic = Some(message);
            }
        }
    }
    loop {
        let before = stale.len();
        for contract in &plan.acquisitions {
            if contract
                .bindings
                .iter()
                .map(|binding| &binding.output)
                .chain(contract.for_each.iter().map(|binding| &binding.output))
                .any(|reference| stale.contains(&reference.step_id))
            {
                stale.insert(contract.id.clone());
            }
        }
        for contract in &plan.derivations {
            if contract
                .inputs
                .iter()
                .any(|reference| stale.contains(&reference.step_id))
            {
                stale.insert(contract.id.clone());
            }
        }
        if stale.len() == before {
            break;
        }
    }
    for id in stale {
        if let Some(step) = steps.get_mut(&id)
            && step.value.is_some()
        {
            step.value = None;
            step.status = StepStatus::Unavailable;
            step.diagnostic = Some(
                "source dependency does not meet its original acquisition freshness contract"
                    .into(),
            );
        }
    }
}
