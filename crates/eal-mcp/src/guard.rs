//! Closed input schemas and shared admission, before operation effects.
use eal_core::{ExecutionLimits, expressions::diagnostic_repr};
use jsonschema::{
    Draft, Retrieve, Uri, Validator,
    error::{TypeKind, ValidationErrorKind},
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;

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

pub(crate) fn validate(arguments: &Value, validator: &Validator) -> Result<(), String> {
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

/// The source SDK runs strict parameter checking after JSON Schema. Its integer
/// parameters reject integral binary64 values, which JSON Schema calls integers.
pub(crate) fn strict_parameters(name: &str, arguments: &Value) -> Result<(), String> {
    let fields: &[&str] = match name {
        "eal_sources" => &["limit", "offset"],
        "eal_find_claims" => &["limit"],
        _ => &[],
    };
    for key in fields {
        if let Some(Value::Number(number)) = arguments.get(*key)
            && number.as_str().contains(['.', 'e', 'E'])
        {
            return Err(format!("Tool input requires a strict integer for {key}"));
        }
    }
    Ok(())
}

/// Bound direct typed callers before recursive schema/library processing.
fn bounded_json(value: &Value) -> Result<(), String> {
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

/// Rejected direct Rust callers can supply deeper values than a wire decoder.
/// Tear those down without invoking Value's recursive container destructor.
pub(crate) fn discard(value: Value) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Array(values) => pending.extend(values),
            Value::Object(values) => pending.extend(values.into_values()),
            _ => {}
        }
    }
}

#[derive(Clone)]
pub(crate) struct Admission(Arc<Semaphore>);
impl Admission {
    pub fn new(max_in_flight: usize) -> Result<Self, String> {
        if !(1..=1024).contains(&max_in_flight) {
            return Err("max_in_flight must be an integer from 1 through 1024".into());
        }
        Ok(Self(Arc::new(Semaphore::new(max_in_flight))))
    }
    pub async fn acquire(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<OwnedSemaphorePermit, String> {
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => Err("Tool request cancelled before admission".into()),
            permit = Arc::clone(&self.0).acquire_owned() => permit.map_err(|_| "Tool admission closed".into()),
        }
    }
}
