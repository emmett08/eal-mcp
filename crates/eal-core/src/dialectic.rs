//! Iterative grounded labelling of finite attack graphs and explicit AND/OR support.
use crate::ExecutionLimits;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const MAX_IDENTIFIER_LENGTH: usize = 256;

fn identifier<'a>(value: &'a Value, location: &str) -> Result<&'a str, String> {
    let value = value
        .as_str()
        .ok_or_else(|| format!("{location} must be a string"))?;
    if value.is_empty() || value.trim() != value || !value.chars().all(printable) {
        return Err(format!(
            "{location} must be nonempty and printable without surrounding whitespace"
        ));
    }
    if value.chars().count() > MAX_IDENTIFIER_LENGTH {
        return Err(format!(
            "{location} exceeds {MAX_IDENTIFIER_LENGTH} characters"
        ));
    }
    Ok(value)
}
fn printable(character: char) -> bool {
    use unicode_general_category::{GeneralCategory as G, get_general_category};
    character == ' '
        || !matches!(
            get_general_category(character),
            G::Control
                | G::Format
                | G::Unassigned
                | G::PrivateUse
                | G::Surrogate
                | G::LineSeparator
                | G::ParagraphSeparator
                | G::SpaceSeparator
        )
}
fn array<'a>(value: &'a Value, name: &str) -> Result<&'a Vec<Value>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{name} must be a list"))
}
fn limit(length: usize, ceiling: usize, name: &str) -> Result<(), String> {
    if length > ceiling {
        Err(format!("{name} exceeds the limit of {ceiling}"))
    } else {
        Ok(())
    }
}
fn edges(
    attacks: &Value,
    declared: &BTreeSet<String>,
    label: &str,
) -> Result<BTreeSet<(String, String)>, String> {
    let mut edges = BTreeSet::new();
    for (index, edge) in array(attacks, "attacks")?.iter().enumerate() {
        let items = edge
            .as_array()
            .filter(|a| a.len() == 2)
            .ok_or_else(|| format!("attacks[{index}] must be a two-element list"))?;
        let source = identifier(&items[0], &format!("attacks[{index}][0]"))?;
        let target = identifier(&items[1], &format!("attacks[{index}][1]"))?;
        if !declared.contains(source) || !declared.contains(target) {
            return Err(format!(
                "attack contains an undeclared {label}: {}",
                py_list(source, target)
            ));
        }
        if !edges.insert((source.to_owned(), target.to_owned())) {
            return Err(format!("duplicate attack: {}", py_list(source, target)));
        }
    }
    Ok(edges)
}
fn py_list(a: &str, b: &str) -> String {
    format!(
        "[{}, {}]",
        super::evaluation::py_repr(&Value::String(a.into())),
        super::evaluation::py_repr(&Value::String(b.into()))
    )
}

/// Dung's least fixed point, with defence certificates referencing earlier waves.
pub fn solve_grounded(
    arguments: &Value,
    attacks: &Value,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    limits.validate()?;
    let arguments = array(arguments, "arguments")?;
    let attack_list = array(attacks, "attacks")?;
    limit(arguments.len(), limits.declarations, "arguments")?;
    limit(attack_list.len(), limits.abstract_attacks, "attacks")?;
    let mut nodes = BTreeSet::new();
    for (index, value) in arguments.iter().enumerate() {
        let node = identifier(value, &format!("arguments[{index}]"))?;
        if !nodes.insert(node.to_owned()) {
            return Err(format!(
                "duplicate argument: {}",
                super::evaluation::py_repr(value)
            ));
        }
    }
    let edges = edges(attacks, &nodes, "argument")?;
    let mut incoming: BTreeMap<String, Vec<String>> =
        nodes.iter().map(|x| (x.clone(), vec![])).collect();
    let mut outgoing = incoming.clone();
    for (source, target) in &edges {
        incoming.get_mut(target).unwrap().push(source.clone());
        outgoing.get_mut(source).unwrap().push(target.clone());
    }
    let mut remaining: BTreeMap<_, _> =
        incoming.iter().map(|(n, a)| (n.clone(), a.len())).collect();
    let mut accepted = BTreeSet::new();
    let mut rejected = BTreeSet::new();
    let mut rejected_by = BTreeMap::new();
    let mut wave: Vec<String> = nodes
        .iter()
        .filter(|n| remaining[*n] == 0)
        .cloned()
        .collect();
    let mut trace = vec![];
    while !wave.is_empty() {
        let accepted_trace:Vec<Value>=wave.iter().map(|n|json!({"argument":n,"defence":incoming[n].iter().map(|a|json!({"attacker":a,"defender":rejected_by[a]})).collect::<Vec<_>>()})).collect();
        accepted.extend(wave.iter().cloned());
        let mut defeated: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for defender in &wave {
            for target in &outgoing[defender] {
                if !rejected.contains(target) {
                    defeated
                        .entry(target.clone())
                        .or_default()
                        .push(defender.clone());
                }
            }
        }
        rejected.extend(defeated.keys().cloned());
        for (target, defenders) in &defeated {
            rejected_by.insert(target.clone(), defenders[0].clone());
        }
        trace.push(json!({"round":trace.len()+1,"accepted":accepted_trace,"rejected":defeated.iter().map(|(t,d)|json!({"argument":t,"attacked_by":d})).collect::<Vec<_>>()}));
        let mut next = BTreeSet::new();
        for target in defeated.keys() {
            for dependent in &outgoing[target] {
                let count = remaining.get_mut(dependent).unwrap();
                *count -= 1;
                if *count == 0 && !accepted.contains(dependent) && !rejected.contains(dependent) {
                    next.insert(dependent.clone());
                }
            }
        }
        wave = next.into_iter().collect();
    }
    let undecided: Vec<_> = nodes
        .iter()
        .filter(|n| !accepted.contains(*n) && !rejected.contains(*n))
        .cloned()
        .collect();
    let unresolved: BTreeMap<_, Vec<_>> = undecided
        .iter()
        .map(|n| {
            (
                n.clone(),
                incoming[n]
                    .iter()
                    .filter(|a| !rejected.contains(*a))
                    .cloned()
                    .collect(),
            )
        })
        .collect();
    Ok(
        json!({"semantics":"grounded","accepted":accepted,"rejected":rejected,"undecided":undecided,"trace":trace,"unresolved_attackers":unresolved}),
    )
}

fn references(
    value: &Value,
    declared: &BTreeSet<String>,
    location: &str,
    edge_count: &mut usize,
    limits: &ExecutionLimits,
) -> Result<Vec<String>, String> {
    let values = array(value, location)?;
    *edge_count += values.len();
    limit(*edge_count, limits.composed_edges, "relationships")?;
    let mut found = BTreeSet::new();
    for item in values {
        let item = identifier(item, location)?;
        if !declared.contains(item) {
            return Err(format!(
                "{location} contains an undeclared identifier: {}",
                super::evaluation::py_repr(&Value::String(item.into()))
            ));
        }
        if !found.insert(item.to_owned()) {
            return Err(format!(
                "{location} contains a duplicate identifier: {}",
                super::evaluation::py_repr(&Value::String(item.into()))
            ));
        }
    }
    Ok(found.into_iter().collect())
}

/// Least-information fixed point over locally usable applications and conjunctive premises.
pub fn solve_composed(
    nodes: &Value,
    claims: &Value,
    attacks: &Value,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    limits.validate()?;
    let nodes = nodes
        .as_object()
        .ok_or("nodes and claims must be dictionaries")?;
    let claims = claims
        .as_object()
        .ok_or("nodes and claims must be dictionaries")?;
    let attack_list = array(attacks, "attacks")?;
    limit(nodes.len(), limits.declarations, "nodes")?;
    limit(claims.len(), limits.declarations, "claims")?;
    limit(attack_list.len(), limits.composed_edges, "relationships")?;
    let node_ids: BTreeSet<_> = nodes.keys().cloned().collect();
    let claim_ids: BTreeSet<_> = claims.keys().cloned().collect();
    for node in &node_ids {
        identifier(&Value::String(node.clone()), "node identifier")?;
    }
    for claim in &claim_ids {
        identifier(&Value::String(claim.clone()), "claim identifier")?;
    }
    let mut edge_count = attack_list.len();
    let mut premises = BTreeMap::new();
    let mut usable = BTreeMap::new();
    for node in &node_ids {
        let spec = nodes[node]
            .as_object()
            .filter(|o| o.len() == 2 && o.contains_key("usable") && o.contains_key("premises"))
            .ok_or_else(|| {
                format!(
                    "Node {} requires exactly usable and premises",
                    super::evaluation::py_repr(&Value::String(node.clone()))
                )
            })?;
        let enabled = spec["usable"].as_bool().ok_or_else(|| {
            format!(
                "Node {} usable must be a boolean",
                super::evaluation::py_repr(&Value::String(node.clone()))
            )
        })?;
        usable.insert(node.clone(), enabled);
        premises.insert(
            node.clone(),
            references(
                &spec["premises"],
                &claim_ids,
                &format!(
                    "Node {} premises",
                    super::evaluation::py_repr(&Value::String(node.clone()))
                ),
                &mut edge_count,
                limits,
            )?,
        );
    }
    let mut derivations = BTreeMap::new();
    for claim in &claim_ids {
        derivations.insert(
            claim.clone(),
            references(
                &claims[claim],
                &node_ids,
                &format!(
                    "Claim {} derivations",
                    super::evaluation::py_repr(&Value::String(claim.clone()))
                ),
                &mut edge_count,
                limits,
            )?,
        );
    }
    let edge_set = edges(attacks, &node_ids, "node")?;
    let mut attackers: BTreeMap<String, Vec<String>> =
        node_ids.iter().map(|n| (n.clone(), vec![])).collect();
    let mut attacked_nodes = attackers.clone();
    let mut derived_claims = attackers.clone();
    let mut premise_dependants: BTreeMap<String, Vec<String>> =
        claim_ids.iter().map(|n| (n.clone(), vec![])).collect();
    for (source, target) in &edge_set {
        attackers.get_mut(target).unwrap().push(source.clone());
        attacked_nodes.get_mut(source).unwrap().push(target.clone());
    }
    for (node, ps) in &premises {
        for p in ps {
            premise_dependants.get_mut(p).unwrap().push(node.clone());
        }
    }
    for (claim, ds) in &derivations {
        for d in ds {
            derived_claims.get_mut(d).unwrap().push(claim.clone());
        }
    }
    let mut node_labels: BTreeMap<String, &str> =
        node_ids.iter().map(|n| (n.clone(), "undecided")).collect();
    let mut claim_labels: BTreeMap<String, &str> =
        claim_ids.iter().map(|n| (n.clone(), "undecided")).collect();
    let mut premises_in: BTreeMap<String, usize> =
        node_ids.iter().map(|n| (n.clone(), 0)).collect();
    let mut premises_out = premises_in.clone();
    let mut attackers_in = premises_in.clone();
    let mut attackers_out = premises_in.clone();
    let mut derivations_in: BTreeMap<String, usize> =
        claim_ids.iter().map(|n| (n.clone(), 0)).collect();
    let mut derivations_out = derivations_in.clone();
    let mut pending_nodes = node_ids;
    let mut pending_claims = claim_ids;
    let mut trace = vec![];
    while !pending_nodes.is_empty() || !pending_claims.is_empty() {
        let mut new_nodes = BTreeMap::new();
        let mut new_claims = BTreeMap::new();
        for node in &pending_nodes {
            if node_labels[node] != "undecided" {
                continue;
            }
            if !usable[node] || premises_out[node] > 0 || attackers_in[node] > 0 {
                new_nodes.insert(node.clone(), "rejected");
            } else if premises_in[node] == premises[node].len()
                && attackers_out[node] == attackers[node].len()
            {
                new_nodes.insert(node.clone(), "accepted");
            }
        }
        for claim in &pending_claims {
            if claim_labels[claim] != "undecided" {
                continue;
            }
            if derivations_in[claim] > 0 {
                new_claims.insert(claim.clone(), "accepted");
            } else if derivations_out[claim] == derivations[claim].len() {
                new_claims.insert(claim.clone(), "rejected");
            }
        }
        if new_nodes.is_empty() && new_claims.is_empty() {
            break;
        }
        let mut node_trace = vec![];
        let mut claim_trace = vec![];
        for (node, label) in &new_nodes {
            let mut reasons = vec![];
            if *label == "accepted" {
                reasons.push(json!({"kind":"local","usable":true}));
                reasons.extend(
                    premises[node]
                        .iter()
                        .map(|p| json!({"kind":"premise","id":p,"status":"accepted"})),
                );
                reasons.extend(
                    attackers[node]
                        .iter()
                        .map(|a| json!({"kind":"attacker","id":a,"status":"rejected"})),
                );
            } else {
                if !usable[node] {
                    reasons.push(json!({"kind":"local","usable":false}));
                }
                reasons.extend(
                    premises[node]
                        .iter()
                        .filter(|p| claim_labels[*p] == "rejected")
                        .map(|p| json!({"kind":"premise","id":p,"status":"rejected"})),
                );
                reasons.extend(
                    attackers[node]
                        .iter()
                        .filter(|a| node_labels[*a] == "accepted")
                        .map(|a| json!({"kind":"attacker","id":a,"status":"accepted"})),
                );
            }
            node_trace.push(json!({"id":node,"status":label,"reasons":reasons}));
        }
        for (claim, label) in &new_claims {
            let mut reasons: Vec<Value> = derivations[claim]
                .iter()
                .filter(|n| node_labels[*n] == *label)
                .map(|n| json!({"kind":"derivation","id":n,"status":label}))
                .collect();
            if reasons.is_empty() {
                reasons.push(json!({"kind":"no_derivation"}));
            }
            claim_trace.push(json!({"id":claim,"status":label,"reasons":reasons}));
        }
        trace.push(json!({"round":trace.len()+1,"nodes":node_trace,"claims":claim_trace}));
        node_labels.extend(new_nodes.clone());
        claim_labels.extend(new_claims.clone());
        pending_nodes = BTreeSet::new();
        pending_claims = BTreeSet::new();
        for (node, label) in new_nodes {
            for target in &attacked_nodes[&node] {
                let counts = if label == "accepted" {
                    &mut attackers_in
                } else {
                    &mut attackers_out
                };
                *counts.get_mut(target).unwrap() += 1;
                pending_nodes.insert(target.clone());
            }
            for claim in &derived_claims[&node] {
                let counts = if label == "accepted" {
                    &mut derivations_in
                } else {
                    &mut derivations_out
                };
                *counts.get_mut(claim).unwrap() += 1;
                pending_claims.insert(claim.clone());
            }
        }
        for (claim, label) in new_claims {
            for node in &premise_dependants[&claim] {
                let counts = if label == "accepted" {
                    &mut premises_in
                } else {
                    &mut premises_out
                };
                *counts.get_mut(node).unwrap() += 1;
                pending_nodes.insert(node.clone());
            }
        }
    }
    Ok(
        json!({"semantics":"grounded_with_support","nodes":node_labels,"claims":claim_labels,"trace":trace}),
    )
}
