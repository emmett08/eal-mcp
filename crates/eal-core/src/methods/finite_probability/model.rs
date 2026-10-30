//! Admit finite Bayesian-network definitions and their explicit state mappings.
use super::{MAX_ASSIGNMENTS, rational};
use crate::methods::MethodError;
use crate::methods::validation::{list, name, object};
use num_rational::BigRational;
use num_traits::{One, Zero};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Node {
    pub name: String,
    pub states: Vec<String>,
    pub parents: Vec<usize>,
    pub probabilities: Vec<BigRational>,
}

pub(super) struct Model {
    pub nodes: Vec<Node>,
    pub assignments: usize,
    pub evidence: BTreeMap<usize, usize>,
    pub target: (usize, usize),
    pub operator: String,
    pub threshold: BigRational,
}

fn state(value: &Value) -> Result<&str, String> {
    value
        .as_str()
        .filter(|text| !text.is_empty() && text.trim() == *text && text.chars().count() <= 64)
        .ok_or_else(|| {
            "State labels require 1..64 characters without surrounding whitespace".into()
        })
}

pub(super) fn parse(value: &Value) -> Result<Model, MethodError> {
    let value = object(
        value,
        &["nodes", "evidence", "target", "comparison"],
        "Finite probability query",
    )?;
    let definitions = list(&value["nodes"], "Bayesian nodes", 1, 12)?;
    let mut names = BTreeMap::new();
    let mut nodes = Vec::new();
    let mut assignments = 1usize;
    for definition in definitions {
        let fields = object(
            definition,
            &["name", "states", "parents", "probabilities"],
            "Bayesian node",
        )?;
        let node_name = name(&fields["name"], "Node name")?.to_owned();
        if names.insert(node_name.clone(), nodes.len()).is_some() {
            return Err("Bayesian node names must be unique".into());
        }
        let states: Vec<_> = list(&fields["states"], "Node states", 2, 8)?
            .iter()
            .map(|value| state(value).map(str::to_owned))
            .collect::<Result<_, _>>()?;
        if states.iter().collect::<BTreeSet<_>>().len() != states.len() {
            return Err("Each node's state labels must be unique".into());
        }
        assignments = assignments
            .checked_mul(states.len())
            .filter(|count| *count <= MAX_ASSIGNMENTS)
            .ok_or("Finite probability model exceeds 65536 joint state assignments")?;
        let probabilities: Vec<_> = list(
            &fields["probabilities"],
            "Conditional probabilities",
            2,
            MAX_ASSIGNMENTS,
        )?
        .iter()
        .map(rational::parse)
        .collect::<Result<_, _>>()?;
        if probabilities
            .iter()
            .any(|value| value < &BigRational::zero() || value > &BigRational::one())
        {
            return Err("Every conditional probability must be in [0, 1]".into());
        }
        nodes.push(Node {
            name: node_name,
            states,
            parents: Vec::new(),
            probabilities,
        });
    }
    for (index, definition) in definitions.iter().enumerate() {
        let parents = list(&definition["parents"], "Node parents", 0, 11)?;
        let mut seen = BTreeSet::new();
        let mut resolved = Vec::new();
        for parent in parents {
            let parent = name(parent, "Parent node")?;
            let position = *names
                .get(parent)
                .ok_or("Parent references an unknown node")?;
            if position == index || !seen.insert(position) {
                return Err(
                    "Node parents must be distinct and cannot include the node itself".into(),
                );
            }
            resolved.push(position);
        }
        let rows = resolved
            .iter()
            .fold(1usize, |rows, parent| rows * nodes[*parent].states.len());
        let width = nodes[index].states.len();
        if nodes[index].probabilities.len() != rows * width {
            return Err(format!(
                "Node {} CPD length does not match its ordered parents and child states",
                nodes[index].name
            )
            .into());
        }
        for row in nodes[index].probabilities.chunks_exact(width) {
            let mut total = BigRational::zero();
            for probability in row {
                total = rational::checked(total + probability)?;
            }
            if total != BigRational::one() {
                return Err(format!(
                    "Node {} CPD rows must sum exactly to one",
                    nodes[index].name
                )
                .into());
            }
        }
        nodes[index].parents = resolved;
    }
    check_acyclic(&nodes)?;
    let locate = |variable: &Value, label: &Value| -> Result<(usize, usize), String> {
        let index = *names
            .get(name(variable, "Query variable")?)
            .ok_or("Query references an unknown node")?;
        let selected = nodes[index]
            .states
            .iter()
            .position(|candidate| candidate == state(label).unwrap_or(""))
            .ok_or("Query references an unknown state")?;
        Ok((index, selected))
    };
    let target = object(&value["target"], &["variable", "state"], "Query target")?;
    let target = locate(&target["variable"], &target["state"])?;
    let evidence = value["evidence"]
        .as_object()
        .ok_or("Evidence must map node names to state labels")?;
    let evidence = evidence
        .iter()
        .map(|(name, label)| locate(&json!(name), label))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let comparison = object(
        &value["comparison"],
        &["operator", "threshold"],
        "Exact comparison",
    )?;
    let operator = comparison["operator"]
        .as_str()
        .filter(|value| ["<", "<=", "==", "!=", ">=", ">"].contains(value))
        .ok_or("Unknown exact comparison operator")?
        .to_owned();
    let threshold = rational::parse(&comparison["threshold"])?;
    Ok(Model {
        nodes,
        assignments,
        evidence,
        target,
        operator,
        threshold,
    })
}

fn check_acyclic(nodes: &[Node]) -> Result<(), String> {
    let mut placed = BTreeSet::new();
    for _ in 0..nodes.len() {
        for (index, node) in nodes.iter().enumerate() {
            if node.parents.iter().all(|parent| placed.contains(parent)) {
                placed.insert(index);
            }
        }
    }
    if placed.len() != nodes.len() {
        return Err("Bayesian graph must be acyclic".into());
    }
    Ok(())
}
