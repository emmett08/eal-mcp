//! Deserialize opaque JSON with exact numbers and literal reserved object keys.
use serde::{Deserialize, Deserializer, de::Error};
use serde_json::{Value, value::RawValue};
use std::collections::BTreeMap;

/// Admit raw JSON through the same recursive, duplicate-rejecting exact parser.
/// RawValue's Value-deserializer support also preserves already-admitted opaque
/// objects; this intentionally never invokes Value's private-number map visitor.
pub fn deserialize_value_exact<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Value, D::Error> {
    let raw = Box::<RawValue>::deserialize(deserializer)?;
    crate::strict_json_exact(raw.get().as_bytes(), crate::JsonLimits::default())
        .map_err(D::Error::custom)
}
pub fn deserialize_value_map_exact<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Value>, D::Error> {
    let value = deserialize_value_exact(deserializer)?;
    match value {
        Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err(D::Error::custom(
            "opaque JSON selector map requires an object",
        )),
    }
}
pub fn deserialize_optional_value_exact<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Value>, D::Error> {
    let raw = Option::<Box<RawValue>>::deserialize(deserializer)?;
    raw.map(|raw| {
        crate::strict_json_exact(raw.get().as_bytes(), crate::JsonLimits::default())
            .map_err(D::Error::custom)
    })
    .transpose()
}
