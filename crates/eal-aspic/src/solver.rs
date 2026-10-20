//! A finite, explicitly bounded ASPIC instantiation over supplied formal data.
mod construction;
mod defeats;
mod extensions;
mod model;
mod preferences;
mod validation;

use std::collections::BTreeSet;

use eal_core::ExecutionLimits;
use serde_json::{Value, json};

use crate::AspicError;
use model::{Argument, Arguments, CheckedTheory, Edges, argument_id};

type SolverResult<T> = Result<T, AspicError>;

/// Check the closed theory schema, names, priorities and formal input budgets.
/// Cyclic dependencies are admitted under finite premise-founded construction.
pub fn validate_theory(theory: &Value, limits: &ExecutionLimits) -> SolverResult<()> {
    validation::checked_theory(theory, limits).map(|_| ())
}

/// Construct arguments and defeats, then compute the complete selected extension
/// family and conclusion query. Exhausted searches return no partial result.
pub fn solve_aspic(payload: &Value, limits: &ExecutionLimits) -> SolverResult<Value> {
    let Some(object) = payload
        .as_object()
        .filter(|object| object.len() == 1 && object.contains_key("theory"))
    else {
        return Err("ASPIC input requires exactly one theory".into());
    };
    let theory_value = &object["theory"];
    let checked = validation::checked_theory(theory_value, limits)?;
    let arguments = construction::construct(&checked, limits)?;
    let (edges, witnesses) = defeats::defeats(&checked, &arguments, limits)?;
    let (accepted, rejected) = defeats::grounded(arguments.values.len(), &edges);
    let goals = arguments
        .by_atom
        .get(&checked.theory.goal)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let grounded_status = if goals.is_empty() {
        "unconstructed"
    } else if goals.iter().any(|goal| accepted.contains(goal)) {
        "accepted"
    } else if goals.iter().all(|goal| rejected.contains(goal)) {
        "rejected"
    } else {
        "undecided"
    };
    check_grounded_consistency(&checked, &arguments, &accepted)?;
    let semantics = checked.theory.options.semantics();
    let query_mode = checked.theory.options.query_mode();
    let extensions = if semantics == "grounded" {
        vec![accepted.clone()]
    } else {
        extensions::extensions(
            arguments.values.len(),
            &edges,
            &accepted,
            &rejected,
            semantics,
            limits,
        )?
    };
    let goal_ids: BTreeSet<_> = goals.iter().copied().collect();
    let query_status = query_status(&extensions, &goal_ids, &edges, query_mode);
    for extension in &extensions {
        let conclusions: BTreeSet<_> = extension
            .iter()
            .map(|index| arguments.values[*index].conclusion.as_str())
            .collect();
        if checked
            .contraries
            .iter()
            .any(|(a, b)| conclusions.contains(a.as_str()) && conclusions.contains(b.as_str()))
        {
            return Err("An extension contains contrary conclusions; outside this profile".into());
        }
    }
    let theory_digest = eal_core::canonical::canonical_digest(theory_value, limits)?;
    Ok(json!({
        "semantics":semantics,
        "query_mode":query_mode,
        "preference":checked.theory.options.preference(),
        "query_status":query_status,
        "query_accepted":query_status=="accepted",
        "query_rejected":query_status=="rejected",
        "extensions":extensions.iter().map(|extension|extension.iter().map(|index|argument_id(*index)).collect::<Vec<_>>()).collect::<Vec<_>>(),
        "grounded_accepted":grounded_status=="accepted",
        "grounded_rejected":grounded_status=="rejected",
        "grounded_status":grounded_status,
        "argument_count":arguments.values.len(),
        "defeat_count":edges.len(),
        "theory_sha256":theory_digest,
        "arguments":arguments.values.iter().enumerate().map(|(index,argument)|argument_json(index,argument,&accepted,&rejected)).collect::<Vec<_>>(),
        "defeats":witnesses,
    }))
}

fn check_grounded_consistency(
    checked: &CheckedTheory,
    arguments: &Arguments,
    accepted: &BTreeSet<usize>,
) -> SolverResult<()> {
    let indefeasible: Vec<_> = arguments
        .values
        .iter()
        .filter(|argument| argument.strength == 1001)
        .collect();
    if indefeasible.iter().any(|a| {
        indefeasible.iter().any(|b| {
            checked
                .contraries
                .contains(&(a.conclusion.clone(), b.conclusion.clone()))
        })
    }) {
        return Err("Axiom and strict-rule closure contains contrary conclusions".into());
    }
    let conclusions: BTreeSet<_> = accepted
        .iter()
        .map(|index| arguments.values[*index].conclusion.as_str())
        .collect();
    if let Some((attacker, target)) = checked
        .contraries
        .iter()
        .find(|(a, b)| conclusions.contains(a.as_str()) && conclusions.contains(b.as_str()))
    {
        // Checked atom names contain ASCII letters/digits/underscore and '~'.
        return Err(format!("Accepted conclusions contain a declared contrary pair '{attacker}' -> '{target}'; outside the bounded ASPIC profile").into());
    }
    Ok(())
}

fn query_status(
    extensions: &[BTreeSet<usize>],
    goals: &BTreeSet<usize>,
    edges: &Edges,
    mode: &str,
) -> &'static str {
    if extensions.is_empty() {
        return "no_extension";
    }
    if goals.is_empty() {
        return "unconstructed";
    }
    let accepts = |extension: &BTreeSet<usize>| !extension.is_disjoint(goals);
    let accepted = if mode == "credulous" {
        extensions.iter().any(accepts)
    } else {
        extensions.iter().all(accepts)
    };
    if accepted {
        return "accepted";
    }
    let all_rejected = extensions.iter().all(|extension| {
        let attacked: BTreeSet<_> = edges
            .iter()
            .filter(|(attacker, _)| extension.contains(attacker))
            .map(|(_, target)| *target)
            .collect();
        goals.is_subset(&attacked)
    });
    if all_rejected {
        "rejected"
    } else {
        "undecided"
    }
}

fn argument_json(
    index: usize,
    argument: &Argument,
    accepted: &BTreeSet<usize>,
    rejected: &BTreeSet<usize>,
) -> Value {
    let mut value = json!({
        "id":argument_id(index),
        "conclusion":argument.conclusion,
        "top":argument.top,
        "subarguments":argument.subarguments.iter().map(|index|argument_id(*index)).collect::<Vec<_>>(),
        "strength":argument.strength,
        "direct_subarguments":argument.direct_subarguments.iter().map(|index|argument_id(*index)).collect::<Vec<_>>(),
        "label":if accepted.contains(&index) { "in" } else if rejected.contains(&index) { "out" } else { "undecided" },
    });
    if let Some(id) = &argument.rule_id {
        value["rule_id"] = json!(id);
    }
    if let Some(name) = &argument.rule_name {
        value["rule_name"] = json!(name);
    }
    if let Some(rank) = argument.rank {
        value["rank"] = json!(rank);
    }
    value
}
