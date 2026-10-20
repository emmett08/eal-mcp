//! Preserve flat semantic diagnostic spans while retaining rich lexical positions.
use eal_parser::{SourcePosition, SourceSpan};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
struct Span {
    line: usize,
    column: usize,
    end_line: usize,
    end_column: usize,
}
impl From<SourceSpan> for Span {
    fn from(span: SourceSpan) -> Self {
        Self {
            line: span.start.line,
            column: span.start.column,
            end_line: span.end.line,
            end_column: span.end.column,
        }
    }
}
impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        Self {
            start: SourcePosition {
                line: span.line,
                column: span.column,
                ..SourcePosition::default()
            },
            end: SourcePosition {
                line: span.end_line,
                column: span.end_column,
                ..SourcePosition::default()
            },
        }
    }
}

pub mod required {
    use super::*;
    pub fn serialize<S: Serializer>(span: &SourceSpan, serializer: S) -> Result<S::Ok, S::Error> {
        Span::from(*span).serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SourceSpan, D::Error> {
        Span::deserialize(deserializer).map(Into::into)
    }
}
pub mod optional {
    use super::*;
    pub fn serialize<S: Serializer>(
        span: &Option<SourceSpan>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        span.map(Span::from).serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<SourceSpan>, D::Error> {
        Option::<Span>::deserialize(deserializer).map(|span| span.map(Into::into))
    }
}
pub mod locations {
    use super::*;
    pub fn serialize<S: Serializer>(
        locations: &BTreeMap<String, SourceSpan>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(locations.len()))?;
        for (key, span) in locations {
            map.serialize_entry(key, &Span::from(*span))?;
        }
        map.end()
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<String, SourceSpan>, D::Error> {
        BTreeMap::<String, Span>::deserialize(deserializer).map(|locations| {
            locations
                .into_iter()
                .map(|(key, span)| (key, span.into()))
                .collect()
        })
    }
}
