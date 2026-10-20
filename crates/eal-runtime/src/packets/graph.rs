use super::shape::{identifier, ids, record_identifier};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn by_conclusion(
    arguments: &Map<String, Value>,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut result = BTreeMap::<String, Vec<String>>::new();
    for (id, item) in arguments {
        if let Some(conclusion) = item.get("conclusion").and_then(Value::as_str) {
            result
                .entry(conclusion.into())
                .or_default()
                .push(identifier(&Value::String(id.clone()))?);
        }
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn claim_observations(
    claim: &str,
    claims: &Map<String, Value>,
    arguments: &Map<String, Value>,
    assumptions: &Map<String, Value>,
    objections: &Map<String, Value>,
    reasoning: &Map<String, Value>,
    evidence: &Map<String, Value>,
) -> Result<(Vec<Value>, bool), String> {
    let by_conclusion = by_conclusion(arguments)?;
    let mut by_target = BTreeMap::<(String, String), Vec<String>>::new();
    for (id, item) in objections {
        if let (Some(kind), Some(target)) = (
            item.get("target_kind").and_then(Value::as_str),
            item.get("target").and_then(Value::as_str),
        ) {
            by_target
                .entry((kind.into(), target.into()))
                .or_default()
                .push(identifier(&Value::String(id.clone()))?);
        }
    }
    let mut pending = vec![("claim".to_owned(), claim.to_owned())];
    let mut visited = BTreeSet::new();
    let mut evidence_ids = BTreeSet::new();
    while !pending.is_empty() && visited.len() < 256 {
        let node = pending.pop().expect("nonempty");
        if !visited.insert(node.clone()) {
            continue;
        }
        pending.extend(
            by_target
                .get(&node)
                .into_iter()
                .flatten()
                .map(|name| ("objection".into(), name.clone())),
        );
        let (kind, id) = node;
        match kind.as_str() {
            "claim" => {
                if claims.contains_key(&id) {
                    pending.extend(
                        by_conclusion
                            .get(&id)
                            .into_iter()
                            .flatten()
                            .map(|name| ("argument".into(), name.clone())),
                    );
                }
            }
            "argument" => {
                let Some(item) = arguments.get(&id).and_then(Value::as_object) else {
                    continue;
                };
                let Some(deps) = item.get("dependencies").and_then(Value::as_object) else {
                    continue;
                };
                evidence_ids.extend(ids(deps.get("evidence"), false)?);
                pending.extend(
                    ids(deps.get("premises"), false)?
                        .into_iter()
                        .map(|name| ("claim".into(), name)),
                );
                pending.extend(
                    ids(deps.get("assumptions"), false)?
                        .into_iter()
                        .map(|name| ("assumption".into(), name)),
                );
                if let Some(rule) = deps.get("reasoning").filter(|v| v.is_string()) {
                    pending.push(("reasoning".into(), identifier(rule)?));
                }
                pending.extend(
                    ids(item.get("attackers"), true)?
                        .into_iter()
                        .map(|name| ("objection".into(), name)),
                );
            }
            "assumption" => {
                if let Some(value) = assumptions
                    .get(&id)
                    .and_then(|v| v.get("validation"))
                    .filter(|v| v.is_string())
                {
                    evidence_ids.insert(identifier(value)?);
                }
            }
            "reasoning" => {
                if let Some(item) = reasoning.get(&id).and_then(Value::as_object) {
                    evidence_ids.extend(ids(item.get("backing"), false)?);
                }
            }
            "objection" => {
                if let Some(item) = objections.get(&id).and_then(Value::as_object) {
                    evidence_ids.extend(ids(item.get("evidence"), false)?);
                    pending.extend(
                        ids(item.get("premises"), false)?
                            .into_iter()
                            .map(|name| ("claim".into(), name)),
                    );
                    pending.extend(
                        ids(item.get("attackers"), true)?
                            .into_iter()
                            .map(|name| ("objection".into(), name)),
                    );
                }
            }
            _ => {}
        }
    }
    let references = evidence_ids.into_iter().filter_map(|id| evidence.get(&id).filter(|v| v.is_object()).map(|item| json!({"evidence_id": id, "observation_id": record_identifier(&item["run_id"])}))).collect();
    Ok((references, !pending.is_empty()))
}
