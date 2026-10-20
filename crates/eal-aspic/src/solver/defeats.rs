//! Directed attacks lifted through subarguments and the selected preferences.
use std::collections::BTreeSet;

use eal_core::ExecutionLimits;
use serde_json::{Value, json};

use super::{SolverResult, model::*, preferences::Preference};

pub(super) fn defeats(
    checked: &CheckedTheory,
    arguments: &Arguments,
    limits: &ExecutionLimits,
) -> SolverResult<(Edges, Vec<Value>)> {
    if checked.contraries.is_empty() {
        return Ok((Edges::new(), Vec::new()));
    }
    let preference = Preference::new(&checked.theory, &checked.priorities);
    let contrary = |attacker: &str, target: &str| {
        checked
            .contraries
            .contains(&(attacker.to_owned(), target.to_owned()))
    };
    let mut edges = Edges::new();
    let mut witnesses = Vec::new();
    for (a, attacker) in arguments.values.iter().enumerate() {
        for (t, target) in arguments.values.iter().enumerate() {
            for sub_id in target.subarguments.iter().copied().chain([t]) {
                let sub = &arguments.values[sub_id];
                let kind = if sub.top == "ordinary"
                    && contrary(&attacker.conclusion, &sub.conclusion)
                {
                    "undermine"
                } else if sub.top == "defeasible"
                    && sub
                        .rule_name
                        .as_deref()
                        .is_some_and(|name| contrary(&attacker.conclusion, name))
                {
                    "undercut"
                } else if sub.top == "defeasible" && contrary(&attacker.conclusion, &sub.conclusion)
                {
                    "rebut"
                } else {
                    continue;
                };
                if kind != "undercut"
                    && contrary(&sub.conclusion, &attacker.conclusion)
                    && preference.weaker(attacker, sub)
                {
                    continue;
                }
                edges.insert((a, t));
                if witnesses.len() >= limits.formal_defeats {
                    return Err(crate::AspicError::Incomplete(
                        "Theory exceeds the defeat-witness budget".into(),
                    ));
                }
                witnesses.push(json!({"attacker":argument_id(a),"target":argument_id(t),"subargument":argument_id(sub_id),"kind":kind}));
            }
        }
    }
    Ok((edges, witnesses))
}

pub(super) fn grounded(count: usize, edges: &Edges) -> (BTreeSet<usize>, BTreeSet<usize>) {
    let mut incoming = vec![BTreeSet::new(); count];
    for &(attacker, target) in edges {
        incoming[target].insert(attacker);
    }
    let mut accepted = BTreeSet::new();
    let mut rejected = BTreeSet::new();
    loop {
        let new_in: BTreeSet<_> = (0..count)
            .filter(|index| {
                !accepted.contains(index)
                    && !rejected.contains(index)
                    && incoming[*index].is_subset(&rejected)
            })
            .collect();
        let new_out: BTreeSet<_> = edges
            .iter()
            .filter(|(attacker, _)| accepted.contains(attacker) || new_in.contains(attacker))
            .map(|(_, target)| *target)
            .filter(|target| !rejected.contains(target))
            .collect();
        if new_in.is_empty() && new_out.is_empty() {
            return (accepted, rejected);
        }
        accepted.extend(new_in);
        rejected.extend(new_out);
    }
}
