//! Canonical exact JSON identities shared by capability discovery and execution.
use super::CapabilityDescriptor;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

impl CapabilityDescriptor {
    /// Identity of the qualified measurement, independent of routing prose and costs.
    pub fn measurement_identity(&self) -> Result<String, String> {
        self.validate()?;
        let semantics = self
            .semantics
            .as_ref()
            .ok_or("A measurement identity requires qualified semantics")?;
        canonical_identity(&serde_json::json!({
            "schema": "eal.measurement-identity/1",
            "capability_id": self.id,
            "capability_version": self.version,
            "provider": self.provider,
            "input_schema_sha256": self.input_schema_sha256,
            "output_schema_sha256": self.output_schema_sha256,
            "semantics": semantics,
        }))
    }
}

pub fn schema_identity(schema: &Value) -> Result<String, String> {
    crate::schema::validate_schema(schema)?;
    canonical_identity(schema)
}

/// Object order is immaterial; numeric tokens retain their exact JSON meaning.
pub fn canonical_identity<T: Serialize + ?Sized>(value: &T) -> Result<String, String> {
    let value = serde_json::to_value(value).map_err(|_| "Cannot encode capability identity")?;
    let mut pending = vec![(&value, 0usize)];
    let mut nodes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        nodes += 1;
        if nodes > 100_000 || depth > 128 {
            return Err("Capability identity exceeds node or depth allowance".into());
        }
        match value {
            Value::Object(map) => pending.extend(map.values().map(|value| (value, depth + 1))),
            Value::Array(array) => pending.extend(array.iter().map(|value| (value, depth + 1))),
            _ => {}
        }
    }
    fn ordered(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut entries = map.iter().collect::<Vec<_>>();
                entries.sort_by(|left, right| left.0.cmp(right.0));
                Value::Object(
                    entries
                        .into_iter()
                        .map(|(key, value)| (key.clone(), ordered(value)))
                        .collect(),
                )
            }
            Value::Array(values) => Value::Array(values.iter().map(ordered).collect()),
            scalar => scalar.clone(),
        }
    }
    let bytes =
        serde_json::to_vec(&ordered(&value)).map_err(|_| "Cannot encode capability identity")?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("Capability identity exceeds 16 MiB".into());
    }
    Ok(Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
