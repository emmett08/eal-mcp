//! Construct scoped support/attack edges and project grounded labels into assessments.
use super::{arguments::SourceAssessment, name_repr, push_reason, status};
use crate::{semantic::Program, validation};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct ComposedGraph {
    pub nodes: Value,
    pub claims: Value,
    pub attacks: BTreeSet<(String, String)>,
    pub objection_sources: BTreeMap<String, bool>,
}
fn distinct(values: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .iter()
        .filter(|v| seen.insert((*v).clone()))
        .cloned()
        .collect()
}

pub(crate) fn construct(
    program: &Program,
    result: &mut Value,
    sources: &SourceAssessment,
) -> Result<ComposedGraph, String> {
    let scopes = validation::objection_scopes(program);
    let mut nodes = json!({});
    let mut claims = json!({});
    for (name, argument) in &program.arguments {
        nodes[format!("argument:{name}")] =
            json!({"usable":sources.local_arguments[name],"premises":distinct(&argument.premises)});
    }
    for (name, arguments) in &sources.by_conclusion {
        claims[name] = json!(
            arguments
                .iter()
                .map(|a| format!("argument:{a}"))
                .collect::<Vec<_>>()
        );
    }
    let mut objection_sources = BTreeMap::new();
    for (name, objection) in &program.objections {
        let environment = scopes
            .get(name)
            .and_then(|s| s.iter().next())
            .ok_or("Objection has no unique checked environment")?;
        let local = status(result, "environments", environment) == "matched"
            && objection
                .evidence
                .iter()
                .all(|e| status(result, "evidence", e) == "available");
        let possible = local
            && objection
                .premises
                .iter()
                .all(|p| sources.source_claims[p]["source_usable"] == true);
        objection_sources.insert(name.clone(), possible);
        nodes[format!("objection:{name}")] =
            json!({"usable":local,"premises":distinct(&objection.premises)});
        result["objections"][name] = json!({"status":"inactive","reasons":objection.evidence.iter().map(|e|format!("Objection evidence {} is {}",name_repr(e),status(result,"evidence",e))).collect::<Vec<_>>(),"target_kind":objection.target_kind,"target":objection.target,"environment":environment,"evidence":objection.evidence,"premises":objection.premises,"source_usable":possible});
    }
    let mut attacks = BTreeSet::new();
    let dependency_edges: usize = nodes
        .as_object()
        .unwrap()
        .values()
        .map(|n| n["premises"].as_array().unwrap().len())
        .sum::<usize>()
        + claims
            .as_object()
            .unwrap()
            .values()
            .map(|ds| ds.as_array().unwrap().len())
            .sum::<usize>();
    for (name, objection) in &program.objections {
        let source = format!("objection:{name}");
        let scope = scopes[name].iter().next().unwrap();
        let targets: Vec<String> = if objection.target_kind == "objection" {
            vec![format!("objection:{}", objection.target)]
        } else {
            program
                .arguments
                .iter()
                .filter(|(_, a)| program.claims[&a.conclusion].environment == *scope)
                .filter(|(name, a)| match objection.target_kind.as_str() {
                    "claim" => a.conclusion == objection.target,
                    "reasoning" => a.reasoning == objection.target,
                    "assumption" => a.assumptions.contains(&objection.target),
                    "argument" => **name == objection.target,
                    _ => false,
                })
                .map(|(name, _)| format!("argument:{name}"))
                .collect()
        };
        for target in targets {
            attacks.insert((source.clone(), target));
            if attacks.len() + dependency_edges > program.limits.composed_edges {
                return Err("Composed attack and support graph exceeds the edge limit".into());
            }
        }
    }
    Ok(ComposedGraph {
        nodes,
        claims,
        attacks,
        objection_sources,
    })
}

pub(crate) fn project(
    program: &Program,
    result: &mut Value,
    sources: &SourceAssessment,
    graph: &ComposedGraph,
    mut grounded: Value,
) {
    let mut attackers: BTreeMap<String, Vec<String>> = graph
        .nodes
        .as_object()
        .unwrap()
        .keys()
        .map(|n| (n.clone(), vec![]))
        .collect();
    for (source, target) in &graph.attacks {
        attackers.get_mut(target).unwrap().push(source.clone());
    }
    grounded["attacks"] = json!(graph.attacks);
    grounded["construction"] = json!("EAL/3 scoped applications with conjunctive claim support");
    result["dialectic"] = grounded.clone();
    for (name, argument) in &program.arguments {
        let label = grounded["nodes"][format!("argument:{name}")]
            .as_str()
            .unwrap();
        let entry = &mut result["arguments"][name];
        entry["grounded_label"] = json!(label);
        entry["attackers"] = json!(attackers[&format!("argument:{name}")]);
        entry["premise_labels"] = argument
            .premises
            .iter()
            .map(|p| (p.clone(), grounded["claims"][p].clone()))
            .collect::<serde_json::Map<_, _>>()
            .into();
        push_reason(
            entry,
            format!("Least-information acceptability label: {label}"),
        );
        if sources.source_arguments[name] {
            entry["status"] = json!(if label == "accepted" {
                "supported"
            } else {
                "contested"
            });
        }
    }
    for (name, objection) in &program.objections {
        let label = grounded["nodes"][format!("objection:{name}")]
            .as_str()
            .unwrap();
        let entry = &mut result["objections"][name];
        entry["grounded_label"] = json!(label);
        entry["attackers"] = json!(attackers[&format!("objection:{name}")]);
        entry["premise_labels"] = objection
            .premises
            .iter()
            .map(|p| (p.clone(), grounded["claims"][p].clone()))
            .collect::<serde_json::Map<_, _>>()
            .into();
        entry["status"] = json!(if graph.objection_sources[name] {
            match label {
                "accepted" => "active",
                "rejected" => "defeated",
                _ => "undecided",
            }
        } else {
            "inactive"
        });
        push_reason(
            entry,
            format!("Least-information acceptability label: {label}"),
        );
        for premise in &objection.premises {
            push_reason(
                entry,
                format!(
                    "Supporting claim {} is {}",
                    name_repr(premise),
                    grounded["claims"][premise].as_str().unwrap()
                ),
            );
        }
    }
    for (name, claim) in &program.claims {
        let label = grounded["claims"][name].as_str().unwrap();
        let supporting: Vec<_> = sources.by_conclusion[name]
            .iter()
            .filter(|a| status(result, "arguments", a) == "supported")
            .cloned()
            .collect();
        let contested: Vec<_> = sources.by_conclusion[name]
            .iter()
            .filter(|a| status(result, "arguments", a) == "contested")
            .cloned()
            .collect();
        let in_scope = status(result, "environments", &claim.environment) == "matched";
        let (state, mut reasons) = if !in_scope {
            (
                "out_of_scope",
                vec![format!(
                    "Environment {} does not match supplied context",
                    name_repr(&claim.environment)
                )],
            )
        } else if !supporting.is_empty() {
            (
                "supported",
                vec![format!("Accepted derivations: {}", supporting.join(", "))],
            )
        } else if !contested.is_empty() {
            (
                "contested",
                vec![format!(
                    "Source-usable derivations are rejected or undecided: {}",
                    contested.join(", ")
                )],
            )
        } else {
            (
                "unsupported",
                vec![
                    "No source-usable derivation supplies support; this does not establish falsity"
                        .into(),
                ],
            )
        };
        reasons.push(format!("Least-information acceptability label: {label}; rejection does not establish claim falsity"));
        let direct: serde_json::Map<String, Value> = program
            .objections
            .iter()
            .filter(|(_, o)| o.target_kind == "claim" && o.target == *name)
            .map(|(o, _)| (o.clone(), result["objections"][o]["status"].clone()))
            .collect();
        let active: Vec<_> = direct
            .iter()
            .filter(|(_, s)| *s == "active")
            .map(|(o, _)| o.clone())
            .collect();
        let undecided: Vec<_> = direct
            .iter()
            .filter(|(_, s)| *s == "undecided")
            .map(|(o, _)| o.clone())
            .collect();
        result["claims"][name] = json!({"status":state,"reasons":reasons,"grounded_label":label,"statement":claim.statement,"environment":claim.environment,"supporting_arguments":supporting,"contested_arguments":contested,"objections":active,"undecided_objections":undecided,"objection_statuses":direct,"source_usable":sources.source_claims[name]["source_usable"],"proposition":super::proposition_value(claim.proposition.as_ref()),"prose_verified":false});
    }
    for (kind, names) in [
        ("reasoning", program.reasoning.keys().collect::<Vec<_>>()),
        ("assumption", program.assumptions.keys().collect::<Vec<_>>()),
    ] {
        let group = if kind == "assumption" {
            "assumptions"
        } else {
            "reasoning"
        };
        for name in names {
            let applicable: serde_json::Map<String, Value> = program
                .objections
                .iter()
                .filter(|(_, o)| o.target_kind == kind && o.target == *name)
                .map(|(o, _)| (o.clone(), result["objections"][o]["status"].clone()))
                .collect();
            let contested = applicable
                .values()
                .any(|s| s == "active" || s == "undecided");
            result[group][name]["objection_statuses"] = json!(applicable);
            if kind == "assumption" && status(result, group, name) == "supported" && contested {
                result[group][name]["status"] = json!("contested");
            }
        }
    }
}
