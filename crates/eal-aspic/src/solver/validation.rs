//! Structural and semantic validation before any formal construction.
use std::collections::{BTreeMap, BTreeSet};

use eal_core::ExecutionLimits;
use serde_json::Value;

use super::{SolverResult, model::*, preferences::priority_order};

pub(super) fn checked_theory(
    value: &Value,
    limits: &ExecutionLimits,
) -> SolverResult<CheckedTheory> {
    limits.validate()?;
    check_json_budget(value, limits)?;
    // Retain the source validator's independent 100k-node/depth-64 profile,
    // after enforcing captured host JSON budgets at this public boundary.
    let errors = eal_core::methods::schema_errors(value, crate::registry::theory_schema(), "$");
    if !errors.is_empty() {
        return Err(errors.join("; ").into());
    }
    let theory: Theory = serde_json::from_value(value.clone())
        .map_err(|error| format!("Invalid ASPIC theory: {error}"))?;
    for (name, count, limit) in [
        ("premises", theory.premises.len(), limits.formal_premises),
        ("rules", theory.rules.len(), limits.formal_rules),
        (
            "contraries",
            theory.contraries.len(),
            limits.formal_contraries,
        ),
    ] {
        if count > limit {
            return Err(crate::AspicError::Incomplete(format!(
                "Theory exceeds the {name} budget of {limit}"
            )));
        }
    }
    let mut atoms = BTreeSet::new();
    let mut premises = BTreeSet::new();
    let mut rules = BTreeSet::new();
    let mut names = BTreeSet::new();
    for item in &theory.premises {
        if !bounded_atom(&item.atom) || !premises.insert(item.atom.clone()) {
            return Err("Premise atoms must be distinct bounded literals".into());
        }
        atoms.insert(item.atom.clone());
    }
    for item in &theory.rules {
        if !bounded_name(&item.id) || !rules.insert(item.id.clone()) {
            return Err("Rule identifiers must be distinct bounded identifiers".into());
        }
        if item.antecedents.len() > limits.formal_antecedents {
            return Err(crate::AspicError::Incomplete(
                "Rule exceeds the antecedent budget".into(),
            ));
        }
        if item.antecedents.iter().collect::<BTreeSet<_>>().len() != item.antecedents.len() {
            return Err("A rule cannot repeat an antecedent".into());
        }
        for atom in item.antecedents.iter().chain([&item.consequent]) {
            if !bounded_atom(atom) {
                return Err("Rules require bounded literal atoms".into());
            }
            atoms.insert(atom.clone());
        }
        if item.kind == "defeasible" {
            let name = item.name.as_deref().unwrap_or("");
            if !bounded_name(name) || !names.insert(name.to_owned()) {
                return Err("Defeasible rule names must be unique positive atoms".into());
            }
            atoms.insert(name.to_owned());
        }
    }
    if !bounded_atom(&theory.goal) {
        return Err("Goal must be a bounded literal atom".into());
    }
    atoms.insert(theory.goal.clone());
    let mut contraries = Contraries::new();
    for pair in &theory.contraries {
        if !bounded_atom(&pair.attacker)
            || !bounded_atom(&pair.target)
            || pair.attacker == pair.target
            || !contraries.insert((pair.attacker.clone(), pair.target.clone()))
        {
            return Err("Contrariness requires unique, distinct bounded literal pairs".into());
        }
        atoms.extend([pair.attacker.clone(), pair.target.clone()]);
    }
    if atoms.len() > limits.formal_atoms {
        return Err(crate::AspicError::Incomplete(
            "Theory exceeds the atom budget".into(),
        ));
    }
    let priorities = priority_order(&theory, limits)?;
    let mut graph: BTreeMap<_, BTreeSet<String>> = atoms
        .iter()
        .map(|atom| (atom.clone(), BTreeSet::new()))
        .collect();
    for rule in &theory.rules {
        for antecedent in &rule.antecedents {
            graph
                .get_mut(antecedent)
                .expect("checked atom")
                .insert(rule.consequent.clone());
        }
    }
    let mut indegree: BTreeMap<_, usize> = atoms.iter().map(|atom| (atom.clone(), 0)).collect();
    for children in graph.values() {
        for child in children {
            *indegree.get_mut(child).expect("checked atom") += 1;
        }
    }
    let mut ready: BTreeSet<_> = indegree
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(atom, _)| atom.clone())
        .collect();
    let mut ordered = Vec::new();
    while let Some(atom) = ready.pop_first() {
        ordered.push(atom.clone());
        for child in &graph[&atom] {
            let count = indegree.get_mut(child).expect("checked atom");
            *count -= 1;
            if *count == 0 {
                ready.insert(child.clone());
            }
        }
    }
    let atom_order = (ordered.len() == atoms.len()).then_some(ordered);
    Ok(CheckedTheory {
        theory,
        contraries,
        atom_order,
        priorities,
    })
}

fn check_json_budget(value: &Value, limits: &ExecutionLimits) -> SolverResult<()> {
    let mut pending = vec![(value, 0usize)];
    let mut remaining = limits.json_nodes;
    while let Some((value, depth)) = pending.pop() {
        if remaining == 0 || depth > limits.json_depth.min(128) {
            return Err(crate::AspicError::Incomplete(
                "ASPIC JSON resource limit exceeded".into(),
            ));
        }
        remaining -= 1;
        match value {
            Value::Array(values) => pending.extend(values.iter().map(|value| (value, depth + 1))),
            Value::Object(values) => {
                pending.extend(values.values().map(|value| (value, depth + 1)))
            }
            _ => {}
        }
    }
    Ok(())
}

fn bounded_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    (1..=64).contains(&value.len())
        && bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn bounded_atom(value: &str) -> bool {
    bounded_name(value.strip_prefix('~').unwrap_or(value))
}
