//! Exhaustive finite Dung definitions independent of the production ASPIC solver.
use eal_aspic::solve_aspic;
use eal_core::ExecutionLimits;
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn attacks(selected: u8, nodes: usize, graph: u16) -> u8 {
    let mut attacked = 0;
    for attacker in 0..nodes {
        if selected & (1 << attacker) == 0 {
            continue;
        }
        for target in 0..nodes {
            if graph & (1 << (attacker * nodes + target)) != 0 {
                attacked |= 1 << target;
            }
        }
    }
    attacked
}

fn defended(selected: u8, argument: usize, nodes: usize, graph: u16) -> bool {
    let counterattacked = attacks(selected, nodes, graph);
    (0..nodes).all(|attacker| {
        graph & (1 << (attacker * nodes + argument)) == 0 || counterattacked & (1 << attacker) != 0
    })
}

fn grounded(nodes: usize, graph: u16) -> u8 {
    let mut current = 0;
    loop {
        let next = (0..nodes)
            .filter(|&argument| defended(current, argument, nodes, graph))
            .fold(0, |set, argument| set | (1 << argument));
        if next == current {
            return current;
        }
        current = next;
    }
}

fn extensions(nodes: usize, graph: u16, stable: bool) -> Vec<u8> {
    let universe = (1 << nodes) - 1;
    let candidates: Vec<_> = (0..=universe)
        .filter(|&selected| {
            let attacked = attacks(selected, nodes, graph);
            if selected & attacked != 0 {
                return false;
            }
            if stable {
                selected | attacked == universe
            } else {
                (0..nodes).all(|argument| {
                    selected & (1 << argument) == 0 || defended(selected, argument, nodes, graph)
                })
            }
        })
        .collect();
    if stable {
        return candidates;
    }
    candidates
        .iter()
        .copied()
        .filter(|&selected| {
            !candidates
                .iter()
                .any(|&larger| selected != larger && selected & larger == selected)
        })
        .collect()
}

fn graph_edges(nodes: usize, graph: u16) -> impl Iterator<Item = (usize, usize)> {
    (0..nodes).flat_map(move |attacker| {
        (0..nodes)
            .filter(move |&target| graph & (1 << (attacker * nodes + target)) != 0)
            .map(move |target| (attacker, target))
    })
}

fn encoded(nodes: usize, graph: u16, premise_encoding: bool) -> Value {
    if premise_encoding {
        return json!({"premises":(0..nodes).map(|n|json!({"atom":format!("node_{n}"),"kind":"ordinary","rank":500})).collect::<Vec<_>>(),"rules":[],
            "contraries":graph_edges(nodes, graph).map(|(a,b)|json!({"attacker":format!("node_{a}"),"target":format!("node_{b}")})).collect::<Vec<_>>(),"goal":"absent"});
    }
    // One indisputable basis is isolated. Each node is a fallible rule; an edge
    // attacks that rule's distinct applicability atom, allowing self-attacks.
    json!({"premises":[{"atom":"basis","kind":"axiom"}],
        "rules":(0..nodes).map(|n|json!({"id":format!("r_{n}"),"kind":"defeasible","antecedents":["basis"],"consequent":format!("node_{n}"),"name":format!("use_{n}"),"rank":500})).collect::<Vec<_>>(),
        "contraries":graph_edges(nodes, graph).map(|(a,b)|json!({"attacker":format!("node_{a}"),"target":format!("use_{b}")})).collect::<Vec<_>>(),"goal":"absent"})
}

fn family(value: &Value, nodes: usize, premise_encoding: bool) -> Vec<u8> {
    let mut extensions: Vec<_> = value
        .as_array()
        .unwrap()
        .iter()
        .map(|extension| {
            let members = extension.as_array().unwrap();
            let mut mask: u8 = 0;
            if !premise_encoding {
                assert!(
                    members.iter().any(|id| id == "A0"),
                    "The isolated axiom must belong to every extension"
                );
            }
            for identifier in members {
                let index: usize = identifier
                    .as_str()
                    .unwrap()
                    .strip_prefix('A')
                    .unwrap()
                    .parse()
                    .unwrap();
                if !premise_encoding && index == 0 {
                    continue;
                }
                let node = index - usize::from(!premise_encoding);
                assert!(node < nodes);
                mask |= 1 << node;
            }
            assert_eq!(
                members.len(),
                mask.count_ones() as usize + usize::from(!premise_encoding)
            );
            mask
        })
        .collect();
    extensions.sort_unstable();
    extensions
}

fn check_encoded_graph(result: &Value, nodes: usize, graph: u16, premise_encoding: bool) {
    let offset = usize::from(!premise_encoding);
    let actual: BTreeSet<_> = result["defeats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|witness| {
            assert_eq!(witness["subargument"], witness["target"]);
            assert_eq!(
                witness["kind"],
                if premise_encoding {
                    "undermine"
                } else {
                    "undercut"
                }
            );
            (
                witness["attacker"].as_str().unwrap().to_owned(),
                witness["target"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    let expected: BTreeSet<_> = graph_edges(nodes, graph)
        .map(|(attacker, target)| {
            (
                format!("A{}", attacker + offset),
                format!("A{}", target + offset),
            )
        })
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(result["defeats"].as_array().unwrap().len(), expected.len());
    let accepted = grounded(nodes, graph);
    let rejected = attacks(accepted, nodes, graph);
    for argument in result["arguments"].as_array().unwrap() {
        let index: usize = argument["id"]
            .as_str()
            .unwrap()
            .strip_prefix('A')
            .unwrap()
            .parse()
            .unwrap();
        if offset == 1 && index == 0 {
            assert_eq!(argument["label"], "in");
            continue;
        }
        let node = index - offset;
        assert!(node < nodes);
        let label = if accepted & (1 << node) != 0 {
            "in"
        } else if rejected & (1 << node) != 0 {
            "out"
        } else {
            "undecided"
        };
        assert_eq!(argument["label"], label);
    }
}

fn check(nodes: usize, graph: u16, premise_encoding: bool, all_goals: bool) -> usize {
    let ground = grounded(nodes, graph);
    let ground_out = attacks(ground, nodes, graph);
    let mut count = 0;
    for semantics in ["grounded", "preferred", "stable"] {
        let expected = if semantics == "grounded" {
            vec![ground]
        } else {
            extensions(nodes, graph, semantics == "stable")
        };
        for mode in ["credulous", "sceptical"] {
            for goal in 0..=if all_goals { nodes } else { 0 } {
                let constructed = goal < nodes;
                let mut theory = encoded(nodes, graph, premise_encoding);
                theory["goal"] = json!(if constructed {
                    format!("node_{goal}")
                } else {
                    "absent".to_owned()
                });
                theory["options"] = json!({"semantics":semantics,"query_mode":mode});
                let result = solve_aspic(&json!({"theory":theory}), &ExecutionLimits::default()).unwrap_or_else(|error| panic!("nodes={nodes} graph={graph} semantics={semantics} mode={mode} goal={goal}: {error}"));
                let context = format!(
                    "nodes={nodes} graph={graph} semantics={semantics} mode={mode} goal={goal} premise={premise_encoding}"
                );
                check_encoded_graph(&result, nodes, graph, premise_encoding);
                assert_eq!(
                    family(&result["extensions"], nodes, premise_encoding),
                    expected,
                    "{context}"
                );
                let status = if expected.is_empty() {
                    "no_extension"
                } else if !constructed {
                    "unconstructed"
                } else {
                    let accepted = if mode == "credulous" {
                        expected.iter().any(|set| set & (1 << goal) != 0)
                    } else {
                        expected.iter().all(|set| set & (1 << goal) != 0)
                    };
                    if accepted {
                        "accepted"
                    } else if expected
                        .iter()
                        .all(|&set| attacks(set, nodes, graph) & (1 << goal) != 0)
                    {
                        "rejected"
                    } else {
                        "undecided"
                    }
                };
                let ground_status = if !constructed {
                    "unconstructed"
                } else if ground & (1 << goal) != 0 {
                    "accepted"
                } else if ground_out & (1 << goal) != 0 {
                    "rejected"
                } else {
                    "undecided"
                };
                assert_eq!(result["query_status"], status, "{context}");
                assert_eq!(result["query_accepted"], status == "accepted", "{context}");
                assert_eq!(result["query_rejected"], status == "rejected", "{context}");
                assert_eq!(result["grounded_status"], ground_status, "{context}");
                assert_eq!(
                    result["grounded_accepted"],
                    ground_status == "accepted",
                    "{context}"
                );
                assert_eq!(
                    result["grounded_rejected"],
                    ground_status == "rejected",
                    "{context}"
                );
                assert_eq!(
                    result["argument_count"],
                    nodes + usize::from(!premise_encoding),
                    "{context}"
                );
                assert_eq!(result["defeat_count"], graph.count_ones(), "{context}");
                count += 1;
            }
        }
    }
    count
}

#[test]
fn every_directed_graph_through_three_nodes_including_self_attacks_matches_subset_definitions() {
    let (mut graphs, mut queries) = (0, 0);
    for nodes in 0..=3 {
        for graph in 0..1_u16 << (nodes * nodes) {
            graphs += 1;
            queries += check(nodes, graph, false, true);
        }
    }
    assert_eq!(graphs, 531);
    assert_eq!(queries, 12_606);
}

#[test]
fn ordinary_premise_encoding_independently_checks_every_irreflexive_three_node_graph() {
    let (mut graphs, mut queries) = (0, 0);
    for nodes in 1..=3 {
        for graph in 0..1_u16 << (nodes * nodes) {
            if (0..nodes).any(|node| graph & (1 << (node * nodes + node)) != 0) {
                continue;
            }
            graphs += 1;
            queries += check(nodes, graph, true, false);
        }
    }
    assert_eq!((graphs, queries), (69, 414));
}
