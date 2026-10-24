//! Bounded reference expansion that preserves literal data inside schema annotations.
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Openapi,
    Schema,
    SchemaMap,
    Literal,
}
struct Expansion {
    nodes: usize,
    bytes: usize,
    oas30: bool,
}

pub(super) fn resolve(
    value: &Value,
    root: &Value,
    active: &mut BTreeSet<String>,
    depth: usize,
) -> Result<Value, String> {
    expand(
        value,
        root,
        active,
        depth,
        Mode::Openapi,
        &mut Expansion {
            nodes: 100_000,
            bytes: 1_048_576,
            oas30: root["openapi"]
                .as_str()
                .is_some_and(|version| version.starts_with("3.0.")),
        },
    )
}
fn expand(
    value: &Value,
    root: &Value,
    active: &mut BTreeSet<String>,
    depth: usize,
    mode: Mode,
    budget: &mut Expansion,
) -> Result<Value, String> {
    if depth > 64 {
        return Err("OpenAPI reference or schema nesting exceeds its allowance".into());
    }
    budget.nodes = budget
        .nodes
        .checked_sub(1)
        .ok_or("OpenAPI expansion exceeds its node allowance")?;
    let size = match value {
        Value::String(s) => s.len(),
        Value::Object(o) => o.keys().map(|s| s.len()).sum(),
        _ => 8,
    };
    budget.bytes = budget
        .bytes
        .checked_sub(size)
        .ok_or("OpenAPI expansion exceeds its byte allowance")?;
    match value {
        Value::Object(object) => {
            if matches!(mode, Mode::Openapi | Mode::Schema)
                && let Some(reference) = object.get("$ref")
            {
                let reference = reference
                    .as_str()
                    .filter(|s| s.starts_with("#/"))
                    .ok_or("OpenAPI import requires local references")?;
                if object.len() != 1 || !active.insert(reference.into()) {
                    return Err(
                        "OpenAPI import rejects reference cycles and reference siblings".into(),
                    );
                }
                let target = root
                    .pointer(&reference[1..])
                    .ok_or("OpenAPI local reference is unresolved")?;
                let result = expand(target, root, active, depth + 1, mode, budget);
                active.remove(reference);
                return result;
            }
            let mut result = serde_json::Map::new();
            for (name, child) in object {
                if mode == Mode::Schema && matches!(name.as_str(), "$dynamicRef" | "$id") {
                    return Err("OpenAPI import rejects dynamic schema scope".into());
                }
                let child_mode = match mode {
                    Mode::Literal => Mode::Literal,
                    Mode::SchemaMap => Mode::Schema,
                    Mode::Schema => match name.as_str() {
                        "properties" | "patternProperties" | "$defs" | "definitions"
                        | "dependentSchemas" | "dependencies" => Mode::SchemaMap,
                        "items"
                        | "prefixItems"
                        | "contains"
                        | "not"
                        | "additionalProperties"
                        | "unevaluatedProperties"
                        | "propertyNames"
                        | "if"
                        | "then"
                        | "else"
                        | "allOf"
                        | "anyOf"
                        | "oneOf" => Mode::Schema,
                        _ => Mode::Literal,
                    },
                    Mode::Openapi => match name.as_str() {
                        "schema" => Mode::Schema,
                        "schemas" => Mode::SchemaMap,
                        "example" | "examples" | "default" | "enum" | "const" | "value" => {
                            Mode::Literal
                        }
                        _ => Mode::Openapi,
                    },
                };
                result.insert(
                    name.clone(),
                    expand(child, root, active, depth + 1, child_mode, budget)?,
                );
            }
            if mode == Mode::Schema && budget.oas30 {
                for (exclusive, inclusive) in [
                    ("exclusiveMinimum", "minimum"),
                    ("exclusiveMaximum", "maximum"),
                ] {
                    if let Some(flag) = result.get(exclusive).and_then(Value::as_bool) {
                        result.remove(exclusive);
                        if flag {
                            let bound = result
                                .remove(inclusive)
                                .ok_or("OpenAPI exclusive bound lacks a value")?;
                            result.insert(exclusive.into(), bound);
                        }
                    }
                }
                if result.remove("nullable") == Some(json!(true)) {
                    return Ok(json!({"anyOf":[Value::Object(result),{"type":"null"}]}));
                }
            }
            Ok(Value::Object(result))
        }
        Value::Array(values) => values
            .iter()
            .map(|v| expand(v, root, active, depth + 1, mode, budget))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        _ => Ok(value.clone()),
    }
}
