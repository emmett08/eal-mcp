//! Exact Python json.dumps sizes, without materialising a second large JSON tree.
use eal_core::{ExecutionLimits, canonical};
use serde_json::Value;

pub(super) fn json_size(
    value: &Value,
    ascii: bool,
    limits: &ExecutionLimits,
) -> Result<usize, String> {
    measure(value, ascii, limits, true)
}

pub(super) fn collection_size(value: &Value, limits: &ExecutionLimits) -> Result<usize, String> {
    // Records already passed their individual admission checks. The source's
    // final collection bound is aggregate bytes across independent observations.
    measure(value, false, limits, false)
}

fn measure(
    value: &Value,
    ascii: bool,
    limits: &ExecutionLimits,
    admission: bool,
) -> Result<usize, String> {
    let mut pending = vec![(value, 0usize)];
    let (mut nodes, mut bytes) = (0usize, 0usize);
    while let Some((value, depth)) = pending.pop() {
        nodes = nodes.saturating_add(1);
        if admission
            && (nodes > limits.json_nodes
                || depth > limits.json_depth.min(eal_core::json::MAX_JSON_DEPTH))
        {
            return Err("JSON resource limit exceeded".into());
        }
        let size = match value {
            Value::Null => 4,
            Value::Bool(true) => 4,
            Value::Bool(false) => 5,
            Value::Number(number) => {
                canonical::canonical_json_bytes(&Value::Number(number.clone()), limits)?.len()
            }
            Value::String(text) => string_size(text, ascii),
            Value::Array(values) => {
                pending.extend(values.iter().map(|v| (v, depth + 1)));
                2 + values.len().saturating_sub(1) * 2
            }
            Value::Object(values) => {
                pending.extend(values.values().map(|v| (v, depth + 1)));
                2 + values.len().saturating_sub(1) * 2
                    + values
                        .keys()
                        .map(|key| string_size(key, ascii) + 2)
                        .sum::<usize>()
            }
        };
        bytes = bytes
            .checked_add(size)
            .ok_or("JSON size exceeds the represented byte range")?;
    }
    Ok(bytes)
}
fn string_size(text: &str, ascii: bool) -> usize {
    2 + text
        .chars()
        .map(|c| match c {
            '"' | '\\' | '\u{08}' | '\u{0c}' | '\n' | '\r' | '\t' => 2,
            c if (c as u32) < 32 => 6,
            c if ascii && !c.is_ascii() => {
                if (c as u32) > 0xffff {
                    12
                } else {
                    6
                }
            }
            c => c.len_utf8(),
        })
        .sum::<usize>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn sizes_retain_spaces_ascii_surrogates_and_binary64_rendering() {
        let limits = ExecutionLimits::default();
        assert_eq!(
            json_size(&json!({"a":["🦀",1.0,-0.0],"b":"é\n"}), true, &limits).unwrap(),
            51
        );
        assert_eq!(
            json_size(&json!({"a":["🦀",1.0,-0.0],"b":"é\n"}), false, &limits).unwrap(),
            39
        );
    }
}
