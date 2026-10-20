//! Compare propagation with independently enumerated complete extensions and retained traces.
use eal_core::{
    ExecutionLimits,
    dialectic::{solve_composed, solve_grounded},
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[test]
fn retained_python_graph_outcomes_and_certificates_match() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/evaluation/dialectic.json")).unwrap();
    assert_eq!(fixture["python_unicode_version"], "15.0.0");
    assert_eq!(unicode_general_category::UNICODE_VERSION, (15, 0, 0));
    for case in fixture["cases"].as_array().unwrap() {
        let mut limits = serde_json::to_value(ExecutionLimits::default()).unwrap();
        if let Some(overrides) = case["limits"].as_object() {
            for (key, value) in overrides {
                limits[key] = value.clone();
            }
        }
        let limits: ExecutionLimits = serde_json::from_value(limits).unwrap();
        let request = &case["request"];
        let result = if case["operation"] == "grounded" {
            solve_grounded(&request["arguments"], &request["attacks"], &limits)
        } else {
            solve_composed(
                &request["nodes"],
                &request["claims"],
                &request["attacks"],
                &limits,
            )
        };
        if let Some(error) = case["error"].as_str() {
            assert_eq!(result.unwrap_err(), error, "{}", case["name"]);
        } else {
            assert_eq!(result.unwrap(), case["result"], "{}", case["name"]);
        }
    }
}

fn independently_enumerated(
    nodes: &[String],
    edges: &BTreeSet<(String, String)>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut complete = vec![];
    for mask in 0..1usize << nodes.len() {
        let candidate: BTreeSet<String> = nodes
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, n)| n.clone())
            .collect();
        let conflict_free = edges
            .iter()
            .all(|(source, target)| !candidate.contains(source) || !candidate.contains(target));
        let defended: BTreeSet<_> = nodes
            .iter()
            .filter(|node| {
                edges
                    .iter()
                    .filter(|(_, target)| target == *node)
                    .all(|(attacker, _)| {
                        edges.iter().any(|(defender, target)| {
                            target == attacker && candidate.contains(defender)
                        })
                    })
            })
            .cloned()
            .collect();
        if conflict_free && candidate == defended {
            complete.push(candidate);
        }
    }
    let least: Vec<_> = complete
        .iter()
        .filter(|candidate| complete.iter().all(|other| candidate.is_subset(other)))
        .collect();
    assert_eq!(least.len(), 1);
    let accepted = least[0];
    let rejected: BTreeSet<_> = edges
        .iter()
        .filter(|(source, _)| accepted.contains(source))
        .map(|(_, target)| target.clone())
        .collect();
    let undecided = nodes
        .iter()
        .filter(|n| !accepted.contains(*n) && !rejected.contains(*n))
        .cloned()
        .collect();
    (
        accepted.iter().cloned().collect(),
        rejected.into_iter().collect(),
        undecided,
    )
}

#[test]
fn every_graph_through_three_nodes_matches_complete_extension_enumeration() {
    for size in 0..=3 {
        let nodes: Vec<_> = (0..size).map(|i| format!("a{i}")).collect();
        let possible: Vec<_> = nodes
            .iter()
            .flat_map(|a| nodes.iter().map(move |b| (a.clone(), b.clone())))
            .collect();
        for mask in 0..1usize << possible.len() {
            let edges: BTreeSet<_> = possible
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, e)| e.clone())
                .collect();
            let result =
                solve_grounded(&json!(nodes), &json!(edges), &ExecutionLimits::default()).unwrap();
            let (accepted, rejected, undecided) = independently_enumerated(&nodes, &edges);
            assert_eq!(result["accepted"], json!(accepted));
            assert_eq!(result["rejected"], json!(rejected));
            assert_eq!(result["undecided"], json!(undecided));
            replay_defence_trace(&nodes, &edges, &result);
        }
    }
}
fn replay_defence_trace(nodes: &[String], edges: &BTreeSet<(String, String)>, result: &Value) {
    let mut accepted = BTreeSet::new();
    let mut rejected = BTreeSet::new();
    for (index, step) in result["trace"].as_array().unwrap().iter().enumerate() {
        assert_eq!(step["round"], index + 1);
        let new_accepted: BTreeSet<_> = step["accepted"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["argument"].as_str().unwrap().to_owned())
            .collect();
        let defended: BTreeSet<_> = nodes
            .iter()
            .filter(|node| {
                edges
                    .iter()
                    .filter(|(_, target)| target == *node)
                    .all(|(source, _)| rejected.contains(source))
            })
            .cloned()
            .collect();
        assert_eq!(
            new_accepted,
            defended.difference(&accepted).cloned().collect()
        );
        for certificate in step["accepted"].as_array().unwrap() {
            for defence in certificate["defence"].as_array().unwrap() {
                let defender = defence["defender"].as_str().unwrap().to_owned();
                let attacker = defence["attacker"].as_str().unwrap().to_owned();
                assert!(accepted.contains(&defender));
                assert!(edges.contains(&(defender, attacker)));
            }
        }
        let new_rejected: BTreeSet<_> = step["rejected"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["argument"].as_str().unwrap().to_owned())
            .collect();
        let attacked: BTreeSet<_> = edges
            .iter()
            .filter(|(source, _)| new_accepted.contains(source))
            .map(|(_, target)| target.clone())
            .collect();
        assert_eq!(
            new_rejected,
            attacked.difference(&rejected).cloned().collect()
        );
        accepted.extend(new_accepted);
        rejected.extend(new_rejected);
    }
    assert_eq!(json!(accepted), result["accepted"]);
    assert_eq!(json!(rejected), result["rejected"]);
}

#[test]
fn full_default_budget_chain_is_iterative_and_replayable() {
    let nodes: Vec<_> = (0..4096).map(|i| format!("a{i:04}")).collect();
    let edges: Vec<_> = nodes
        .windows(2)
        .map(|w| vec![w[0].clone(), w[1].clone()])
        .collect();
    let result = solve_grounded(&json!(nodes), &json!(edges), &ExecutionLimits::default()).unwrap();
    assert_eq!(
        result["accepted"],
        json!(nodes.iter().step_by(2).collect::<Vec<_>>())
    );
    assert_eq!(
        result["rejected"],
        json!(nodes.iter().skip(1).step_by(2).collect::<Vec<_>>())
    );
    assert_eq!(result["trace"].as_array().unwrap().len(), 2048);
}

#[test]
fn attack_only_support_extension_agrees_with_dung_on_all_three_node_graphs() {
    let nodes = ["a", "b", "c"];
    for mask in 0..512 {
        let mut attacks = vec![];
        for i in 0..3 {
            for j in 0..3 {
                if mask & (1 << (i * 3 + j)) != 0 {
                    attacks.push(vec![nodes[i], nodes[j]]);
                }
            }
        }
        let plain =
            solve_grounded(&json!(nodes), &json!(attacks), &ExecutionLimits::default()).unwrap();
        let composed=solve_composed(&json!({"a":{"usable":true,"premises":[]},"b":{"usable":true,"premises":[]},"c":{"usable":true,"premises":[]}}),&json!({}),&json!(attacks),&ExecutionLimits::default()).unwrap();
        for node in nodes {
            let label = if plain["accepted"].as_array().unwrap().contains(&json!(node)) {
                "accepted"
            } else if plain["rejected"].as_array().unwrap().contains(&json!(node)) {
                "rejected"
            } else {
                "undecided"
            };
            assert_eq!(composed["nodes"][node], label);
        }
    }
}
