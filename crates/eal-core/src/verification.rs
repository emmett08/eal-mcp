//! Bounded source-to-formula grounding and applicability, separate from entailment.
//!
//! Coverage comes from the consumer's selected argument, not the proposer. This
//! module has no clock or IO: callers supply actual acquired values and their
//! original observation times. Passing these checks does not authenticate a
//! source or prove that the declared denotations exhaust an English claim.
mod applicability;
mod details;
mod evaluate;
pub(crate) mod native;
mod types;

pub use evaluate::{evaluate, evaluate_query, validate_reference};
pub use types::*;

/// Aggregate consumer-selected dependency reports without adding a measurement
/// or performing an inference. Missing or ambiguous dependency reports remain
/// unresolved. Child reports are bound by canonical digest, not copied.
pub fn compose(
    target_id: &str,
    mode: Mode,
    required_targets: &[String],
    reports: &[Report],
    limits: &crate::ExecutionLimits,
) -> Result<Report, String> {
    use serde_json::json;
    use std::collections::BTreeSet;
    limits.validate()?;
    if target_id.trim().is_empty()
        || target_id.len() > 128
        || required_targets.len() > 128
        || reports.len() > 128
    {
        return Err(
            "Report composition requires a bounded target and at most 128 dependencies".into(),
        );
    }
    if required_targets
        .iter()
        .any(|target| target.trim().is_empty() || target.len() > 128 || target == target_id)
        || required_targets.iter().collect::<BTreeSet<_>>().len() != required_targets.len()
    {
        return Err(
            "Composition dependencies require unique nonempty IDs different from the target".into(),
        );
    }
    crate::canonical::canonical_json_footprint(
        &serde_json::to_value(reports).map_err(|error| error.to_string())?,
        limits,
    )?;
    let mut obligations = Vec::new();
    let mut status = Status::Pass;
    if required_targets.is_empty() {
        status = Status::Unknown;
        obligations.push(evaluate::obligation(
            "coverage",
            Role::Grounding,
            Status::Unknown,
            "A composed target has no independently selected dependencies",
            json!({}),
        ));
    }
    for target in required_targets {
        let matching: Vec<_> = reports
            .iter()
            .filter(|report| &report.target_id == target)
            .collect();
        let (child_status, details, reason) = if let [report] = matching.as_slice() {
            if report.schema != REPORT_SCHEMA || report.status == Status::NotRequired {
                (
                    Status::Unknown,
                    json!({"dependency_target":target}),
                    "Dependency report has an unsupported schema or overall state",
                )
            } else {
                let digest = crate::canonical::canonical_digest(
                    &serde_json::to_value(report).map_err(|error| error.to_string())?,
                    limits,
                )?;
                (
                    report.status,
                    json!({"dependency_target":target,"report_sha256":digest,"dependency_mode":report.mode}),
                    "Retained result of the independently selected dependency",
                )
            }
        } else {
            (
                Status::Unknown,
                json!({"dependency_target":target,"matching_reports":matching.len()}),
                "A selected dependency report is missing or ambiguous",
            )
        };
        if child_status == Status::Fail {
            status = Status::Fail;
        } else if child_status == Status::Unknown && status != Status::Fail {
            status = Status::Unknown;
        }
        obligations.push(evaluate::obligation(
            target,
            Role::Grounding,
            child_status,
            reason,
            details,
        ));
    }
    Ok(Report {
        schema: REPORT_SCHEMA.into(),
        target_id: target_id.into(),
        mode,
        status,
        obligations,
        atoms: vec![],
        formulas: vec![],
    })
}
