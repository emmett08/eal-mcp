//! Local-only schema admission. Validation never follows external references.
use serde_json::Value;
pub(crate) fn validate_schema(schema: &Value) -> Result<(), String> {
    let mut pending = vec![(schema, 0usize)];
    let mut count = 0usize;
    while let Some((value, depth)) = pending.pop() {
        count += 1;
        if count > 100_000 || depth > 128 {
            return Err("EAL_SCHEMA_LIMIT: schema exceeds node or depth allowance".into());
        }
        match value {
            Value::Object(object) => {
                for (name, value) in object {
                    if (name == "$ref" || name == "$dynamicRef")
                        && value
                            .as_str()
                            .is_none_or(|reference| !reference.starts_with('#'))
                    {
                        return Err(
                            "EAL_SCHEMA_REFERENCE: collector schemas require local references"
                                .into(),
                        );
                    }
                    pending.push((value, depth + 1));
                }
            }
            Value::Array(array) => pending.extend(array.iter().map(|value| (value, depth + 1))),
            _ => {}
        }
    }
    jsonschema::validator_for(schema)
        .map_err(|_| "EAL_SCHEMA_INVALID: malformed collector schema")?;
    Ok(())
}
