//! Strict, bounded JSON ingestion preserving arbitrary integers and literal keys.
use crate::ExecutionLimits;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use serde_json::{Map, Number, Value};
use std::fmt;

/// Native recursive conversions retain a hard ceiling independently of host settings.
pub const MAX_JSON_DEPTH: usize = 128;

/// Reject duplicate decoded keys, non-finite numbers and resource exhaustion.
///
/// Raw values prevent serde_json's arbitrary-number interchange keys from
/// reinterpreting an ordinary user-authored JSON object as a number.
pub fn parse(source: &str, limits: &ExecutionLimits) -> Result<Value, String> {
    limits.validate()?;
    if source.len() > limits.record_bytes {
        return Err("JSON input exceeds the record byte budget".into());
    }
    let raw: &RawValue = serde_json::from_str(source).map_err(|error| error.to_string())?;
    let mut remaining = limits.json_nodes;
    decode(raw, 0, &mut remaining, limits)
}

struct ObjectPairs<'a>(Vec<(String, &'a RawValue)>);

impl<'de> Deserialize<'de> for ObjectPairs<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PairsVisitor;
        impl<'de> Visitor<'de> for PairsVisitor {
            type Value = ObjectPairs<'de>;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object with unique keys")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut pairs = Vec::new();
                while let Some(pair) = map.next_entry()? {
                    pairs.push(pair);
                }
                Ok(ObjectPairs(pairs))
            }
        }
        deserializer.deserialize_map(PairsVisitor)
    }
}

fn decode(
    raw: &RawValue,
    depth: usize,
    remaining: &mut usize,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    if depth > limits.json_depth.min(MAX_JSON_DEPTH) || *remaining == 0 {
        return Err("JSON resource limit exceeded".into());
    }
    *remaining -= 1;
    let source = raw.get();
    match source.as_bytes().first() {
        Some(b'n') => Ok(Value::Null),
        Some(b't') => Ok(Value::Bool(true)),
        Some(b'f') => Ok(Value::Bool(false)),
        Some(b'"') => serde_json::from_str::<String>(source)
            .map(Value::String)
            .map_err(|e| e.to_string()),
        Some(b'[') => {
            let values: Vec<&RawValue> = serde_json::from_str(source).map_err(|e| e.to_string())?;
            values
                .into_iter()
                .map(|value| decode(value, depth + 1, remaining, limits))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array)
        }
        Some(b'{') => {
            let pairs: ObjectPairs<'_> = serde_json::from_str(source).map_err(|e| e.to_string())?;
            let mut object = Map::new();
            for (key, value) in pairs.0 {
                if object.contains_key(&key) {
                    return Err(format!("Duplicate JSON object key {key:?}"));
                }
                object.insert(key, decode(value, depth + 1, remaining, limits)?);
            }
            Ok(Value::Object(object))
        }
        _ => {
            let number = if source.contains(['.', 'e', 'E']) {
                let value: f64 = source.parse().map_err(|_| "JSON numbers must be finite")?;
                Number::from_f64(value).ok_or("JSON numbers must be finite")?
            } else if source == "-0" {
                Number::from(0)
            } else {
                source.parse::<Number>().map_err(|e| e.to_string())?
            };
            Ok(Value::Number(number))
        }
    }
}
