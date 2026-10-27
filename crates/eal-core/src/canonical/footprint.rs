//! Count canonical values without allocating an encoded copy.
use super::encode;
use crate::ExecutionLimits;
use serde_json::Value;
use std::io::{self, Write};

#[derive(Clone, Copy, Debug)]
pub(crate) struct JsonFootprint {
    pub nodes: usize,
    pub bytes: usize,
    /// Root scalars have depth zero, matching the canonical encoder.
    pub depth: usize,
}

pub(crate) fn canonical_json_footprint(
    value: &Value,
    limits: &ExecutionLimits,
) -> Result<JsonFootprint, String> {
    limits.validate()?;
    let mut output = CountingWriter {
        bytes: 0,
        allowance: limits.record_bytes,
    };
    let mut remaining = limits.json_nodes;
    let mut maximum_depth = 0;
    encode(
        value,
        0,
        &mut remaining,
        &mut maximum_depth,
        limits,
        &mut output,
    )?;
    Ok(JsonFootprint {
        nodes: limits.json_nodes - remaining,
        bytes: output.bytes,
        depth: maximum_depth,
    })
}

struct CountingWriter {
    bytes: usize,
    allowance: usize,
}

impl Write for CountingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let total = self
            .bytes
            .checked_add(buffer.len())
            .filter(|total| *total <= self.allowance)
            .ok_or_else(|| io::Error::other("JSON value exceeds the record byte budget"))?;
        self.bytes = total;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::canonical_json_footprint;
    use crate::{ExecutionLimits, canonical};
    use serde_json::json;

    #[test]
    fn measured_footprints_match_canonical_escape_and_numeric_boundaries() {
        for (value, expected, nodes, depth) in [
            (json!(null), "null", 1, 0),
            (json!("🫨\n\"\\\u{0001}"), r#""🫨\n\"\\\u0001""#, 1, 0),
            (json!(-0.0), "-0.0", 1, 0),
            (json!(1.0e-9), "1e-09", 1, 0),
            (json!(1.0e20), "1e+20", 1, 0),
            (
                json!({"z":[1,{"a":"🫨\n"}],"a":true}),
                r#"{"a":true,"z":[1,{"a":"🫨\n"}]}"#,
                6,
                3,
            ),
            (json!({"\n🫨\"":0}), r#"{"\n🫨\"":0}"#, 2, 1),
        ] {
            assert_eq!(
                canonical::canonical_json_bytes(&value, &ExecutionLimits::default()).unwrap(),
                expected.as_bytes()
            );
            let bytes = expected.len();
            let limits = ExecutionLimits {
                json_nodes: nodes,
                record_bytes: bytes,
                ..ExecutionLimits::default()
            };
            let actual = canonical_json_footprint(&value, &limits).unwrap();
            assert_eq!(actual.bytes, bytes);
            assert_eq!(actual.nodes, nodes);
            assert_eq!(actual.depth, depth);
            assert!(
                canonical_json_footprint(
                    &value,
                    &ExecutionLimits {
                        record_bytes: bytes - 1,
                        ..limits
                    },
                )
                .is_err()
            );
        }
    }
}
