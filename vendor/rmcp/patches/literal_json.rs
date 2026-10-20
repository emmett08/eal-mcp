//! Lossless JSON literal keys at the SDK's Serde buffering boundary.
//!
//! Serde's arbitrary-precision JSON numbers use a private map internally. Its
//! untagged buffering cannot distinguish that map from the same ordinary JSON
//! object. Escape actual wire keys before buffering, and restore only arbitrary
//! payload fields after their Value decoding. Numbers never pass through f64.
use serde::{
    Deserialize, Deserializer,
    de::{DeserializeOwned, Error as _, MapAccess, Visitor},
};
use serde_json::{Map, Value, value::RawValue};
use std::{
    borrow::Cow,
    cell::Cell,
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt,
    sync::Arc,
};

const PREFIX: &str = "$eal.literal-key/";
const NUMBER: &str = "$serde_json::private::Number";
const RAW: &str = "$serde_json::private::RawValue";
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_DEPTH: usize = 128;
const MAX_NODES: usize = 100_000;

thread_local! { static WIRE_DECODE: Cell<bool> = const { Cell::new(false) }; }
struct DecodeScope(bool);
impl DecodeScope {
    fn enter() -> Self {
        Self(WIRE_DECODE.replace(true))
    }
}
impl Drop for DecodeScope {
    fn drop(&mut self) {
        WIRE_DECODE.set(self.0);
    }
}

pub(crate) fn from_slice<T: DeserializeOwned>(source: &[u8]) -> serde_json::Result<T> {
    let source = std::str::from_utf8(source).map_err(serde_json::Error::custom)?;
    from_str(source)
}
pub(crate) fn from_str<T: DeserializeOwned>(source: &str) -> serde_json::Result<T> {
    if source.len() > MAX_BYTES {
        return Err(serde_json::Error::custom("MCP JSON byte limit exceeded"));
    }
    let raw: &RawValue = serde_json::from_str(source)?;
    let mut encoded = String::with_capacity(source.len());
    let mut remaining = MAX_NODES;
    encode(raw, &mut encoded, &mut remaining, 0)?;
    let _scope = DecodeScope::enter();
    serde_json::from_str(&encoded)
}

struct Pairs<'a>(Vec<(String, &'a RawValue)>);
impl<'de> Deserialize<'de> for Pairs<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PairVisitor;
        impl<'de> Visitor<'de> for PairVisitor {
            type Value = Pairs<'de>;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut pairs = Vec::new();
                while let Some(pair) = map.next_entry()? {
                    if pairs.len() == MAX_NODES {
                        return Err(A::Error::custom("MCP JSON node limit exceeded"));
                    }
                    pairs.push(pair);
                }
                Ok(Pairs(pairs))
            }
        }
        deserializer.deserialize_map(PairVisitor)
    }
}

fn encode(
    raw: &RawValue,
    target: &mut String,
    remaining: &mut usize,
    depth: usize,
) -> serde_json::Result<()> {
    if depth > MAX_DEPTH || *remaining == 0 {
        return Err(serde_json::Error::custom(
            "MCP JSON resource limit exceeded",
        ));
    }
    *remaining -= 1;
    let source = raw.get();
    match source.as_bytes().first() {
        Some(b'{') => {
            let pairs: Pairs<'_> = serde_json::from_str(source)?;
            let mut seen = BTreeSet::new();
            target.push('{');
            for (index, (key, value)) in pairs.0.into_iter().enumerate() {
                if !seen.insert(key.clone()) {
                    return Err(serde_json::Error::custom("Duplicate JSON object key"));
                }
                if index > 0 {
                    target.push(',');
                }
                let key = if key == NUMBER || key == RAW || key.starts_with(PREFIX) {
                    format!("{PREFIX}{key}")
                } else {
                    key
                };
                target.push_str(&serde_json::to_string(&key)?);
                target.push(':');
                encode(value, target, remaining, depth + 1)?;
            }
            target.push('}');
        }
        Some(b'[') => {
            let values: Vec<&RawValue> = serde_json::from_str(source)?;
            if values.len() > *remaining {
                return Err(serde_json::Error::custom("MCP JSON node limit exceeded"));
            }
            target.push('[');
            for (index, value) in values.into_iter().enumerate() {
                if index > 0 {
                    target.push(',');
                }
                encode(value, target, remaining, depth + 1)?;
            }
            target.push(']');
        }
        _ => target.push_str(source),
    }
    // Encoding adds at most one prefix per key; allow its bounded overhead so
    // the public byte ceiling applies to the original wire message.
    if target.len() > MAX_BYTES * 2 {
        return Err(serde_json::Error::custom("MCP JSON byte limit exceeded"));
    }
    Ok(())
}

pub(crate) trait Restore {
    fn restore(&mut self);
}
impl Restore for Value {
    fn restore(&mut self) {
        match self {
            Value::Object(object) => object.restore(),
            Value::Array(values) => values.iter_mut().for_each(Restore::restore),
            _ => {}
        }
    }
}
impl Restore for Map<String, Value> {
    fn restore(&mut self) {
        let original = std::mem::take(self);
        for (key, mut value) in original {
            value.restore();
            let key = key.strip_prefix(PREFIX).map(str::to_owned).unwrap_or(key);
            self.insert(key, value);
        }
    }
}
impl<T: Restore> Restore for Option<T> {
    fn restore(&mut self) {
        if let Some(value) = self {
            value.restore();
        }
    }
}
impl<T: Restore + Clone> Restore for Arc<T> {
    fn restore(&mut self) {
        Arc::make_mut(self).restore();
    }
}
impl Restore for Cow<'_, Map<String, Value>> {
    fn restore(&mut self) {
        self.to_mut().restore();
    }
}
impl<T: Restore> Restore for BTreeMap<String, T> {
    fn restore(&mut self) {
        let original = std::mem::take(self);
        for (key, mut value) in original {
            value.restore();
            let key = key.strip_prefix(PREFIX).map(str::to_owned).unwrap_or(key);
            self.insert(key, value);
        }
    }
}

/// Used only for arbitrary JSON payloads, not protocol envelopes or identifiers.
pub(crate) fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Restore,
{
    let mut value = T::deserialize(deserializer)?;
    restore_if_wire(&mut value);
    Ok(value)
}

pub(crate) fn restore_if_wire<T: Restore>(value: &mut T) {
    if WIRE_DECODE.get() {
        value.restore();
    }
}

// Typed map values perform their own field decoding; restore only their keys.
pub(crate) trait RestoreKeys {
    fn restore_keys(&mut self);
}
impl<T> RestoreKeys for BTreeMap<String, T> {
    fn restore_keys(&mut self) {
        *self = std::mem::take(self)
            .into_iter()
            .map(|(key, value)| {
                (
                    key.strip_prefix(PREFIX).map(str::to_owned).unwrap_or(key),
                    value,
                )
            })
            .collect();
    }
}
impl<T> RestoreKeys for HashMap<String, T> {
    fn restore_keys(&mut self) {
        *self = std::mem::take(self)
            .into_iter()
            .map(|(key, value)| {
                (
                    key.strip_prefix(PREFIX).map(str::to_owned).unwrap_or(key),
                    value,
                )
            })
            .collect();
    }
}
impl<T> RestoreKeys for indexmap::IndexMap<String, T> {
    fn restore_keys(&mut self) {
        *self = std::mem::take(self)
            .into_iter()
            .map(|(key, value)| {
                (
                    key.strip_prefix(PREFIX).map(str::to_owned).unwrap_or(key),
                    value,
                )
            })
            .collect();
    }
}
impl<T: RestoreKeys> RestoreKeys for Option<T> {
    fn restore_keys(&mut self) {
        if let Some(value) = self {
            value.restore_keys();
        }
    }
}
pub(crate) fn deserialize_keys<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + RestoreKeys,
{
    let mut value = T::deserialize(deserializer)?;
    if WIRE_DECODE.get() {
        value.restore_keys();
    }
    Ok(value)
}
