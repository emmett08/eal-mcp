use super::*;
use crate::{ExecutionLimits, canonical, expressions, methods::logical};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub fn evaluate(
    coverage: &Coverage,
    contract: &Contract,
    context: &Context,
    limits: &ExecutionLimits,
) -> Result<Report, String> {
    validate(coverage, contract, context, limits)?;
    let mut report = Report {
        schema: REPORT_SCHEMA.into(),
        target_id: coverage.target_id.clone(),
        mode: coverage.mode,
        status: Status::Pass,
        obligations: vec![],
        atoms: vec![],
        formulas: vec![],
    };
    if contract.target_id != coverage.target_id || contract.mode != coverage.mode {
        report.obligations.push(obligation("coverage",Role::Grounding,Status::Fail,
            "Submitted contract changes the independently required target or mode",json!({"expected_target":coverage.target_id,"actual_target":contract.target_id,"expected_mode":coverage.mode,"actual_mode":contract.mode})));
    }
    let mut used = BTreeSet::new();
    if coverage.mode == Mode::Mathematical {
        report.obligations.push(obligation("grounding",Role::Grounding,Status::NotRequired,
            "A mathematical consequence needs no empirical atom valuation; its formal inference is checked separately",json!({"inference_checked":false})));
        for formula in &coverage.formulas {
            report.formulas.push(FormulaResult{id:formula.id.clone(),status:Status::NotRequired,truth:None,
                reasons:vec!["This formula is a mathematical definition or hypothesis, not an asserted observation".into()],details:json!({"formula":formula.formula})});
            report_budget(&report, limits)?;
        }
    } else {
        let required_atoms =
            coverage
                .formulas
                .iter()
                .try_fold(BTreeSet::new(), |mut all, formula| {
                    all.extend(logical::formula_atoms(&formula.formula)?);
                    Ok::<_, String>(all)
                })?;
        let mut assignment = BTreeMap::new();
        for symbol in &required_atoms {
            let Some(binding) = contract
                .atoms
                .iter()
                .find(|binding| &binding.symbol == symbol)
            else {
                report.atoms.push(AtomResult {
                    symbol: symbol.clone(),
                    observation_id: None,
                    status: Status::Unknown,
                    truth: None,
                    reasons: vec![
                        "No executable source predicate defines this required formula atom".into(),
                    ],
                    details: json!({}),
                });
                continue;
            };
            used.insert(binding.observation_id.clone());
            let (status, truth, reason, details) =
                source_predicate(&binding.observation_id, &binding.predicate, context, limits);
            if let Some(truth) = truth {
                assignment.insert(symbol.clone(), truth);
            }
            report.atoms.push(AtomResult {
                symbol: symbol.clone(),
                observation_id: Some(binding.observation_id.clone()),
                status,
                truth,
                reasons: vec![reason],
                details,
            });
            report_budget(&report, limits)?;
        }
        for formula in &coverage.formulas {
            let missing: Vec<_> = logical::formula_atoms(&formula.formula)?
                .into_iter()
                .filter(|atom| !assignment.contains_key(atom))
                .collect();
            let truth = if missing.is_empty() {
                Some(logical::grounded_truth(&formula.formula, &assignment)?)
            } else {
                None
            };
            let status = match truth {
                Some(true) => Status::Pass,
                Some(false) => Status::Fail,
                None => Status::Unknown,
            };
            report.formulas.push(FormulaResult{id:formula.id.clone(),status,truth,
                reasons:vec![match truth{Some(true)=>"The formal premise holds under the acquired atom valuation".into(),Some(false)=>"The acquired atom valuation refutes the formal premise".into(),None=>"Required atom denotations or observations remain unavailable".into()}],
                details:json!({"formula":formula.formula,"valuation":assignment,"missing_atoms":missing})});
            report_budget(&report, limits)?;
        }
        if coverage.formulas.is_empty() && coverage.required_obligations.is_empty() {
            report.obligations.push(obligation("coverage",Role::Grounding,Status::Unknown,
                "An empirical or defeasible target has no independently required formula or source obligation",json!({})));
        }
    }
    let mut checked = BTreeSet::new();
    for required in &coverage.required_obligations {
        let Some(binding) = contract
            .obligations
            .iter()
            .find(|binding| binding.id == required.id && binding.role == required.role)
        else {
            report.obligations.push(obligation(
                &required.id,
                required.role,
                Status::Unknown,
                "The contract omits an independently required source obligation",
                json!({}),
            ));
            continue;
        };
        checked.insert(binding.id.clone());
        used.insert(binding.observation_id.clone());
        let (available, truth, reason, details) =
            source_predicate(&binding.observation_id, &binding.predicate, context, limits);
        let status = if available != Status::Pass {
            available
        } else if truth == Some(true) {
            Status::Pass
        } else {
            Status::Fail
        };
        report.obligations.push(obligation(
            &binding.id,
            binding.role,
            status,
            &reason,
            details,
        ));
        report_budget(&report, limits)?;
    }
    // Additional authored conditions strengthen the contract; never silently
    // ignore one merely because it was not necessary for minimum coverage.
    for binding in &contract.obligations {
        if checked.contains(&binding.id) {
            continue;
        }
        used.insert(binding.observation_id.clone());
        let (available, truth, reason, details) =
            source_predicate(&binding.observation_id, &binding.predicate, context, limits);
        let status = if available != Status::Pass {
            available
        } else if truth == Some(true) {
            Status::Pass
        } else {
            Status::Fail
        };
        report.obligations.push(obligation(
            &binding.id,
            binding.role,
            status,
            &reason,
            details,
        ));
        report_budget(&report, limits)?;
    }
    if coverage.mode == Mode::Defeasible
        && !coverage
            .required_obligations
            .iter()
            .any(|obligation| obligation.role == Role::Warrant)
    {
        report.obligations.push(obligation(
            "coverage:warrant",
            Role::Warrant,
            Status::Unknown,
            "A defeasible target requires an independently identified warrant or review obligation",
            json!({}),
        ));
    }
    if used.is_empty() {
        report.obligations.push(obligation(
            "applicability",
            Role::Applicability,
            if coverage.mode == Mode::Mathematical {
                Status::NotRequired
            } else {
                Status::Unknown
            },
            if coverage.mode == Mode::Mathematical {
                "There is no relied-upon empirical observation"
            } else {
                "No acquired observations establish this target's applicability"
            },
            json!({}),
        ));
    }
    for source in used {
        report.obligations.extend(super::applicability::evaluate(
            &source, contract, context, limits,
        )?);
        report_budget(&report, limits)?;
    }
    report.status = aggregate(
        report
            .obligations
            .iter()
            .map(|result| result.status)
            .chain(report.atoms.iter().map(|result| result.status))
            .chain(report.formulas.iter().map(|result| result.status)),
    );
    for result in &mut report.obligations {
        super::details::project(&mut result.details);
    }
    for result in &mut report.atoms {
        super::details::project(&mut result.details);
    }
    for result in &mut report.formulas {
        super::details::project(&mut result.details);
    }
    report_budget(&report, limits)?;
    Ok(report)
}

fn report_budget(report: &Report, limits: &ExecutionLimits) -> Result<(), String> {
    let footprint = canonical::canonical_json_footprint(
        &serde_json::to_value(report).map_err(|error| error.to_string())?,
        limits,
    )?;
    if footprint.bytes > 1_048_576 {
        return Err("Verification report byte budget exhausted".into());
    }
    Ok(())
}

pub fn evaluate_query(query: &Value, limits: &ExecutionLimits) -> Result<Report, String> {
    canonical::canonical_json_footprint(query, limits)?;
    let query: Query = serde_json::from_value(query.clone())
        .map_err(|error| format!("Invalid verification query: {error}"))?;
    evaluate(
        &query.coverage,
        &query.contract,
        &Context {
            observations: query.observations,
            assessed_at: query.assessed_at,
        },
        limits,
    )
}

/// Admit authored coverage and executable predicates before any acquisition.
/// Undefined source values still belong to later unknown results, not invented data.
pub fn validate_reference(
    coverage: &Value,
    contract: &Value,
    limits: &ExecutionLimits,
) -> Result<(), String> {
    let coverage: Coverage =
        serde_json::from_value(coverage.clone()).map_err(|error| error.to_string())?;
    let contract: Contract =
        serde_json::from_value(contract.clone()).map_err(|error| error.to_string())?;
    validate(
        &coverage,
        &contract,
        &Context {
            observations: BTreeMap::new(),
            assessed_at: String::new(),
        },
        limits,
    )
}

pub(super) fn obligation(
    id: &str,
    role: Role,
    status: Status,
    reason: &str,
    details: Value,
) -> ObligationResult {
    ObligationResult {
        id: id.into(),
        role,
        status,
        reasons: vec![reason.into()],
        details,
    }
}
fn aggregate(statuses: impl Iterator<Item = Status>) -> Status {
    let mut overall = Status::Pass;
    for status in statuses {
        match status {
            Status::Fail => return Status::Fail,
            Status::Unknown => overall = Status::Unknown,
            _ => {}
        }
    }
    overall
}
pub(super) fn source_predicate(
    source: &str,
    predicate: &Predicate,
    context: &Context,
    limits: &ExecutionLimits,
) -> (Status, Option<bool>, String, Value) {
    let Some(observation) = context.observations.get(source) else {
        return (
            Status::Unknown,
            None,
            "The required acquired observation is missing".into(),
            json!({"observation_id":source}),
        );
    };
    let native_predicate = crate::semantic::Predicate::from(predicate.clone());
    let read_fields: BTreeMap<_, _> = expressions::predicate_paths(&native_predicate)
        .into_iter()
        .map(|path| {
            let actual = path.split('.').try_fold(&observation.value, |value, part| {
                value.as_object()?.get(part)
            });
            (path, actual.cloned())
        })
        .collect();
    let details = json!({"observation_id":source,"predicate":predicate,"read_fields":read_fields,"observed_at":observation.observed_at,"source_digest":observation.source_digest});
    if observation.status != ObservationStatus::Available {
        return (
            Status::Unknown,
            None,
            "The acquisition is unavailable or failed; no truth value is invented".into(),
            details,
        );
    }
    if read_fields.values().any(Option::is_none) {
        return (
            Status::Unknown,
            None,
            "A source field required by the grounding predicate is missing".into(),
            details,
        );
    }
    let checked = crate::evaluation::predicate_check(&native_predicate, &observation.value, limits);
    if checked.comparable {
        (Status::Pass, Some(checked.holds), checked.reason, details)
    } else {
        (Status::Unknown, None, checked.reason, details)
    }
}

fn validate(
    coverage: &Coverage,
    contract: &Contract,
    context: &Context,
    limits: &ExecutionLimits,
) -> Result<(), String> {
    limits.validate()?;
    let footprint = canonical::canonical_json_footprint(
        &json!({"coverage":coverage,"contract":contract,"context":context}),
        limits,
    )?;
    if footprint.nodes.saturating_mul(64) > limits.method_work {
        return Err("Verification method_work budget exhausted".into());
    }
    let text = |value: &str| {
        !value.trim().is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
    };
    if contract.schema != CONTRACT_SCHEMA
        || !text(&coverage.target_id)
        || !text(&contract.target_id)
    {
        return Err("Verification requires its supported schema and bounded target IDs".into());
    }
    if coverage.formulas.len() > 128
        || coverage.required_obligations.len() > 128
        || contract.atoms.len() > 128
        || contract.obligations.len() > 128
        || contract.applicability.len() > 128
        || context.observations.len() > 128
    {
        return Err("Verification supports at most 128 entries per contract collection".into());
    }
    let mut seen = BTreeSet::new();
    for formula in &coverage.formulas {
        if !text(&formula.id) || !seen.insert(formula.id.clone()) {
            return Err("Formula IDs must be unique bounded names".into());
        }
        logical::formula_atoms(&formula.formula)?;
    }
    seen.clear();
    for required in &coverage.required_obligations {
        if !text(&required.id) || !seen.insert(required.id.clone()) {
            return Err("Required obligation IDs must be unique bounded names".into());
        }
    }
    seen.clear();
    for atom in &contract.atoms {
        if !text(&atom.symbol) || !text(&atom.observation_id) || !seen.insert(atom.symbol.clone()) {
            return Err("Atom symbols require unique bounded names and source IDs".into());
        }
        validate_predicate(&atom.predicate, limits)?;
    }
    seen.clear();
    for obligation in &contract.obligations {
        if !text(&obligation.id)
            || !text(&obligation.observation_id)
            || !seen.insert(obligation.id.clone())
        {
            return Err("Obligation bindings require unique bounded IDs and sources".into());
        }
        validate_predicate(&obligation.predicate, limits)?;
    }
    seen.clear();
    for applicability in &contract.applicability {
        if !text(&applicability.observation_id)
            || !seen.insert(applicability.observation_id.clone())
        {
            return Err("Each source has at most one applicability contract".into());
        }
        for pointer in [
            &applicability.identity_pointer,
            &applicability.unit_pointer,
            &applicability.scope_pointer,
            &applicability.valid_from_pointer,
            &applicability.valid_until_pointer,
        ] {
            if !pointer.starts_with('/') || pointer.len() > 2048 {
                return Err("Applicability facets require bounded JSON pointers".into());
            }
        }
        if applicability.unit.is_empty() || applicability.max_age_seconds == 0 {
            return Err(
                "Applicability needs an explicit unit and positive maximum observation age".into(),
            );
        }
    }
    Ok(())
}
fn validate_predicate(predicate: &Predicate, limits: &ExecutionLimits) -> Result<(), String> {
    if predicate.expression.is_none()
        && (!expressions::property_path(&predicate.path)
            || expressions::comparison_operator(&predicate.operator).is_none())
    {
        return Err(
            "A source predicate requires a valid EAL field comparison or closed expression".into(),
        );
    }
    let predicate = crate::semantic::Predicate::from(predicate.clone());
    if expressions::predicate_paths(&predicate).is_empty() {
        return Err("A source predicate must read an actual acquired field; constant expressions do not ground a proposition".into());
    }
    let errors = expressions::expression_errors_with_limits(
        &expressions::predicate_expression(&predicate),
        None,
        limits,
    );
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
