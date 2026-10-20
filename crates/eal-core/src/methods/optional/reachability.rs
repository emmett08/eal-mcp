//! Breadth-first finite reachability with an inclusive horizon and shortest witness.
use serde_json::{Value, json};
use std::collections::{BTreeSet, VecDeque};

pub(super) fn assess(payload: &Value) -> Result<Value, String> {
    let count = payload["node_count"]
        .as_u64()
        .ok_or("node_count must be an integer")? as usize;
    let start = payload["start"]
        .as_u64()
        .ok_or("start must be an integer")? as usize;
    let horizon = payload["horizon"]
        .as_u64()
        .ok_or("horizon must be an integer")? as usize;
    let forbidden: Vec<_> = payload["forbidden"]
        .as_array()
        .ok_or("forbidden must be a list")?
        .iter()
        .map(|v| {
            v.as_u64()
                .map(|v| v as usize)
                .ok_or("forbidden state must be an integer")
        })
        .collect::<Result<_, _>>()?;
    let edges: Vec<_> = payload["edges"]
        .as_array()
        .ok_or("edges must be a list")?
        .iter()
        .map(|e| {
            let pair = e
                .as_array()
                .filter(|e| e.len() == 2)
                .ok_or("transition edge requires two states")?;
            Ok((
                pair[0].as_u64().ok_or("edge state must be integer")? as usize,
                pair[1].as_u64().ok_or("edge state must be integer")? as usize,
            ))
        })
        .collect::<Result<_, String>>()?;
    if start >= count || forbidden.iter().any(|n| *n >= count) {
        return Err("Start and forbidden states must belong to the graph".into());
    }
    if forbidden.iter().collect::<BTreeSet<_>>().len() != forbidden.len()
        || edges.iter().collect::<BTreeSet<_>>().len() != edges.len()
    {
        return Err("Duplicate forbidden states or transition edges".into());
    }
    if edges.iter().any(|(a, b)| *a >= count || *b >= count) {
        return Err("A transition endpoint is outside the graph".into());
    }
    let mut adjacency = vec![Vec::new(); count];
    for (a, b) in edges {
        adjacency[a].push(b);
    }
    for successors in &mut adjacency {
        successors.sort();
    }
    let mut queue = VecDeque::from([vec![start]]);
    let mut seen = BTreeSet::from([start]);
    let forbidden: BTreeSet<_> = forbidden.into_iter().collect();
    while let Some(path) = queue.pop_front() {
        let at = *path.last().expect("nonempty path");
        if forbidden.contains(&at) {
            return Ok(
                json!({"reachable":true,"shortest_length":path.len()-1,"counterexample":path,"visited":seen.len()}),
            );
        }
        if path.len() - 1 == horizon {
            continue;
        }
        for successor in &adjacency[at] {
            if seen.insert(*successor) {
                let mut next = path.clone();
                next.push(*successor);
                queue.push_back(next);
            }
        }
    }
    Ok(json!({"reachable":false,"shortest_length":-1,"counterexample":[],"visited":seen.len()}))
}
