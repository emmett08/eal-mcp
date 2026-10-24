//! Local-only schema admission with literal annotation data kept distinct from schemas.
use serde_json::Value;
#[derive(Clone, Copy)]
enum Position {
    Schema,
    Map,
    Literal,
}
pub(crate) fn validate_schema(schema: &Value) -> Result<(), String> {
    let mut pending = vec![(schema, 0usize, Position::Schema)];
    let mut count = 0usize;
    while let Some((value, depth, position)) = pending.pop() {
        count += 1;
        if count > 100_000 || depth > 128 {
            return Err("EAL_SCHEMA_LIMIT: schema exceeds node or depth allowance".into());
        }
        match value {
            Value::Object(object) => {
                for (name, value) in object {
                    if matches!(position, Position::Schema)
                        && matches!(name.as_str(), "$ref" | "$dynamicRef")
                        && value
                            .as_str()
                            .is_none_or(|reference| !reference.starts_with('#'))
                    {
                        return Err(
                            "EAL_SCHEMA_REFERENCE: collector schemas require local references"
                                .into(),
                        );
                    }
                    let child = match position {
                        Position::Literal => Position::Literal,
                        Position::Map => Position::Schema,
                        Position::Schema => match name.as_str() {
                            "properties" | "patternProperties" | "$defs" | "definitions"
                            | "dependentSchemas" | "dependencies" => Position::Map,
                            "items"
                            | "prefixItems"
                            | "contains"
                            | "not"
                            | "additionalItems"
                            | "additionalProperties"
                            | "unevaluatedItems"
                            | "unevaluatedProperties"
                            | "propertyNames"
                            | "if"
                            | "then"
                            | "else"
                            | "allOf"
                            | "anyOf"
                            | "oneOf"
                            | "contentSchema" => Position::Schema,
                            _ => Position::Literal,
                        },
                    };
                    pending.push((value, depth + 1, child));
                }
            }
            Value::Array(array) => {
                pending.extend(array.iter().map(|value| (value, depth + 1, position)))
            }
            _ => {}
        }
    }
    jsonschema::validator_for(schema)
        .map_err(|_| "EAL_SCHEMA_INVALID: malformed collector schema")?;
    Ok(())
}
