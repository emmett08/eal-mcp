//! Admit object fields before retaining a constructed value larger than its policy.
use super::ExpressionResult;
use crate::{ExecutionLimits, Expression, canonical};
use serde_json::{Map, Value};

pub(super) fn object(
    arguments: &[Expression],
    limits: &ExecutionLimits,
    mut evaluate: impl FnMut(&Expression) -> Result<ExpressionResult, String>,
) -> Result<ExpressionResult, String> {
    if !arguments.len().is_multiple_of(2) {
        return Err("Function \"object\" has an invalid argument count".into());
    }
    let mut footprint = canonical::canonical_json_footprint(&Value::Object(Map::new()), limits)?;
    let mut fields = Map::new();
    let mut unknown = None;
    let mut invalid = None;
    for pair in arguments.chunks_exact(2) {
        let key = evaluate(&pair[0])?;
        let value = evaluate(&pair[1])?;
        if !key.known() || !value.known() {
            if unknown.is_none() {
                unknown = Some(if !key.known() { key } else { value });
            }
            fields.clear();
            continue;
        }
        if unknown.is_some() || invalid.is_some() {
            continue;
        }
        let Some(Value::String(key)) = key.value else {
            invalid = Some("object keys must be strings".to_owned());
            fields.clear();
            continue;
        };
        let Some(value) = value.value else {
            invalid = Some(
                "object values must be JSON; quantities require explicit unit fields".to_owned(),
            );
            fields.clear();
            continue;
        };
        if fields.contains_key(&key) {
            invalid = Some("object keys must be unique".to_owned());
            fields.clear();
            continue;
        }
        let key_bytes =
            canonical::canonical_json_footprint(&Value::String(key.clone()), limits)?.bytes;
        let value_size = canonical::canonical_json_footprint(&value, limits)?;
        footprint = extend(footprint, key_bytes, value_size, !fields.is_empty(), limits)?;
        fields.insert(key, value);
    }
    if let Some(unknown) = unknown {
        return Ok(unknown);
    }
    if let Some(error) = invalid {
        return Err(error);
    }
    Ok(ExpressionResult::json(Value::Object(fields)))
}

fn extend(
    current: canonical::JsonFootprint,
    key_bytes: usize,
    value: canonical::JsonFootprint,
    comma: bool,
    limits: &ExecutionLimits,
) -> Result<canonical::JsonFootprint, String> {
    let nodes = current
        .nodes
        .checked_add(value.nodes)
        .filter(|nodes| *nodes <= limits.json_nodes)
        .ok_or("Object construction exceeds the JSON node budget")?;
    let depth = value
        .depth
        .checked_add(1)
        .map(|depth| depth.max(current.depth))
        .filter(|depth| *depth <= limits.json_depth.min(crate::json::MAX_JSON_DEPTH))
        .ok_or("Object construction exceeds the JSON depth budget")?;
    let bytes = current
        .bytes
        .checked_add(key_bytes)
        .and_then(|bytes| bytes.checked_add(1 + usize::from(comma)))
        .and_then(|bytes| bytes.checked_add(value.bytes))
        .filter(|bytes| *bytes <= limits.record_bytes)
        .ok_or("Object construction exceeds the record byte budget")?;
    Ok(canonical::JsonFootprint {
        nodes,
        bytes,
        depth,
    })
}
