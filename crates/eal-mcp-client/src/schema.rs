//! Bounded JSON Schema validation with no external references.
use eal_core::{ExecutionLimits, expressions::diagnostic_repr};
use jsonschema::{
    Draft, Retrieve, Uri, Validator,
    error::{TypeKind, ValidationErrorKind},
};
use serde_json::Value;
struct NoExternalReferences;
impl Retrieve for NoExternalReferences {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Err(format!("External schema references are not allowed: {uri}").into())
    }
}

/// JSON Schema never resolves network or filesystem references.
pub fn compile_schema(schema: &Value) -> Result<Validator, String> {
    bounded_json(schema)?;
    jsonschema::options()
        .with_draft(Draft::Draft202012)
        .with_retriever(NoExternalReferences)
        .build(schema)
        .map_err(|error| error.to_string())
}

pub fn validate_schema(arguments: &Value, schema: &Value) -> Result<(), String> {
    validate(arguments, &compile_schema(schema)?)
}

pub fn validate(arguments: &Value, validator: &Validator) -> Result<(), String> {
    bounded_json(arguments)?;
    if let Some(error) = validator.iter_errors(arguments).next() {
        let repr = |value: &Value| diagnostic_repr(value, &ExecutionLimits::default());
        let message = match error.kind() {
            ValidationErrorKind::Type {
                kind: TypeKind::Single(kind),
            } => format!("{} is not of type '{kind}'", repr(error.instance())?),
            ValidationErrorKind::Type {
                kind: TypeKind::Multiple(kinds),
            } => {
                let kinds = kinds
                    .iter()
                    .map(|kind| format!("'{kind}'"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} is not of type {kinds}", repr(error.instance())?)
            }
            ValidationErrorKind::Required { property } => {
                format!("{} is a required property", repr(property)?)
            }
            ValidationErrorKind::AnyOf { .. } => format!(
                "{} is not valid under any of the given schemas",
                repr(error.instance())?
            ),
            ValidationErrorKind::AdditionalProperties { unexpected } => {
                let mut names = unexpected.iter().collect::<Vec<_>>();
                names.sort();
                let names = names
                    .iter()
                    .map(|name| repr(&Value::String((*name).clone())))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                let suffix = if unexpected.len() == 1 { "was" } else { "were" };
                format!("Additional properties are not allowed ({names} {suffix} unexpected)")
            }
            _ => error.to_string(),
        };
        return Err(format!("Tool input schema rejected request: {message}"));
    }
    Ok(())
}

/// Bound direct typed callers before recursive schema/library processing.
pub fn bounded_json(value: &Value) -> Result<(), String> {
    let mut pending = vec![(value, 0usize)];
    let mut nodes = 0usize;
    let mut bytes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        nodes += 1;
        if depth > 128 || nodes > 100_000 {
            return Err("MCP JSON resource limit exceeded".into());
        }
        bytes = bytes.saturating_add(match value {
            Value::String(text) => text.len(),
            Value::Number(number) => number.as_str().len(),
            Value::Object(object) => {
                if object.len() > 100_000usize.saturating_sub(nodes + pending.len()) {
                    return Err("MCP JSON resource limit exceeded".into());
                }
                for (key, value) in object {
                    bytes = bytes.saturating_add(key.len());
                    pending.push((value, depth + 1));
                }
                object.len().saturating_mul(4)
            }
            Value::Array(array) => {
                if array.len() > 100_000usize.saturating_sub(nodes + pending.len()) {
                    return Err("MCP JSON resource limit exceeded".into());
                }
                pending.extend(array.iter().map(|value| (value, depth + 1)));
                array.len()
            }
            _ => 5,
        });
        if bytes > 4 * 1024 * 1024 {
            return Err("MCP JSON resource limit exceeded".into());
        }
    }
    Ok(())
}
