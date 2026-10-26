//! Strict versioned messages for complete-round coordination.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, value::RawValue};

use crate::{DebateConfig, Edge, MAX_FRAME_BYTES};

/// Frozen admission settings for one bounded session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSpec {
    pub schema_version: String,
    pub debate: DebateConfig,
    pub max_payload_bytes: usize,
}

/// One participant's opaque response, tied to its participant and source round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeerValue {
    pub participant: usize,
    pub round: usize,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub payload: Value,
}

/// Coordinator-selected context for exactly one participant call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallContext {
    pub participant: usize,
    pub round: usize,
    /// The complete admitted directed graph for this round.
    pub edges: Vec<Edge>,
    /// Permitted senders, in participant order.
    pub incoming: Vec<usize>,
    /// This participant's own previous-round response, separate from peer edges.
    /// Absence means no previous round. A present JSON null is an opaque payload.
    #[serde(
        default,
        deserialize_with = "previous_payload",
        skip_serializing_if = "Option::is_none"
    )]
    pub previous: Option<Value>,
    /// Only the previous-round responses of admitted incoming neighbours.
    pub peers: Vec<PeerValue>,
}

fn previous_payload<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    eal_collector_contracts::deserialize_value_exact(deserializer).map(Some)
}

/// The complete set of independent calls issued at one round barrier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dispatch {
    pub round: usize,
    pub calls: Vec<CallContext>,
}

/// A complete round response. Partial, duplicate or misidentified results fail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundResult {
    pub round: usize,
    pub responses: Vec<PeerValue>,
}

/// Client commands cannot revise the frozen specification after admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientMessage {
    Start { spec: SessionSpec },
    Results { result: RoundResult },
}

/// Coordinator output keeps completion and rejection distinct from dispatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ServerMessage {
    Dispatch {
        dispatch: Dispatch,
    },
    /// Counts include independent round 0 and its participant calls.
    Completed {
        rounds: usize,
        calls: usize,
    },
    Failed {
        message: String,
    },
}

// Internally tagged enum derives buffer nested values through Serde's generic
// Content representation. That representation cannot support the RawValue
// visitors needed to retain literal reserved keys and exact JSON numbers.
impl<'de> Deserialize<'de> for ClientMessage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut fields = wire_fields(deserializer)?;
        let kind: String = take(&mut fields, "kind")?;
        let message = match kind.as_str() {
            "start" => Self::Start {
                spec: take(&mut fields, "spec")?,
            },
            "results" => Self::Results {
                result: take(&mut fields, "result")?,
            },
            _ => {
                return Err(serde::de::Error::custom(
                    "unknown client debate message kind",
                ));
            }
        };
        finish(fields, message)
    }
}

impl<'de> Deserialize<'de> for ServerMessage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut fields = wire_fields(deserializer)?;
        let kind: String = take(&mut fields, "kind")?;
        let message = match kind.as_str() {
            "dispatch" => Self::Dispatch {
                dispatch: take(&mut fields, "dispatch")?,
            },
            "completed" => Self::Completed {
                rounds: take(&mut fields, "rounds")?,
                calls: take(&mut fields, "calls")?,
            },
            "failed" => Self::Failed {
                message: take(&mut fields, "message")?,
            },
            _ => {
                return Err(serde::de::Error::custom(
                    "unknown server debate message kind",
                ));
            }
        };
        finish(fields, message)
    }
}

fn wire_fields<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Map<String, Value>, D::Error> {
    let raw = Box::<RawValue>::deserialize(deserializer)?;
    let value = eal_collector_contracts::strict_json_exact(
        raw.get().as_bytes(),
        eal_collector_contracts::JsonLimits {
            max_bytes: MAX_FRAME_BYTES - 1,
            max_depth: 128,
            max_nodes: MAX_FRAME_BYTES,
        },
    )
    .map_err(serde::de::Error::custom)?;
    match value {
        Value::Object(fields) => Ok(fields),
        _ => Err(serde::de::Error::custom(
            "debate message requires an object",
        )),
    }
}

fn take<T: DeserializeOwned, E: serde::de::Error>(
    fields: &mut Map<String, Value>,
    name: &'static str,
) -> Result<T, E> {
    let value = fields.remove(name).ok_or_else(|| E::missing_field(name))?;
    serde_json::from_value(value).map_err(E::custom)
}

fn finish<T, E: serde::de::Error>(fields: Map<String, Value>, message: T) -> Result<T, E> {
    if !fields.is_empty() {
        return Err(E::custom("unexpected debate envelope field"));
    }
    Ok(message)
}
