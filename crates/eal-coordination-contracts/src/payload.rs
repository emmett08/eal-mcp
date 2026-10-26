//! Iterative payload-shape admission before recursive JSON serialisation.

use serde_json::Value;

use crate::{Error, MAX_PAYLOAD_DEPTH, MAX_PAYLOAD_NODES};

pub(crate) fn validate(value: &Value, max_bytes: usize) -> Result<(), Error> {
    let max_nodes = MAX_PAYLOAD_NODES.min(max_bytes);
    let mut pending = vec![(value, 0)];
    let mut visited = 0;
    while let Some((value, depth)) = pending.pop() {
        if depth > MAX_PAYLOAD_DEPTH {
            return Err(Error::Protocol(format!(
                "opaque payload exceeds maximum depth {MAX_PAYLOAD_DEPTH}"
            )));
        }
        visited += 1;
        if visited > max_nodes {
            return Err(Error::Protocol(format!(
                "opaque payload exceeds maximum node count {max_nodes}"
            )));
        }
        let children = match value {
            Value::Array(values) => values.len(),
            Value::Object(values) => values.len(),
            _ => 0,
        };
        if children > max_nodes - visited - pending.len() {
            return Err(Error::Protocol(format!(
                "opaque payload exceeds maximum node count {max_nodes}"
            )));
        }
        match value {
            Value::Array(values) => {
                pending.extend(values.iter().map(|value| (value, depth + 1)));
            }
            Value::Object(values) => {
                pending.extend(values.values().map(|value| (value, depth + 1)));
            }
            _ => {}
        }
    }
    Ok(())
}
