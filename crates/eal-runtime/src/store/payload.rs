use serde_json::Value;

/// Independent persistence envelope, accommodating complete 32 MiB collections
/// and 16 MiB observations without inheriting a per-observation execution budget.
pub const MAX_STORED_RECORD_BYTES: usize = 64 * 1024 * 1024;
fn storage_limits() -> eal_core::ExecutionLimits {
    eal_core::ExecutionLimits {
        record_bytes: MAX_STORED_RECORD_BYTES,
        json_nodes: MAX_STORED_RECORD_BYTES,
        json_depth: eal_core::json::MAX_JSON_DEPTH,
        ..eal_core::ExecutionLimits::default()
    }
}

pub(super) fn decode(encoded: &str) -> Result<Value, String> {
    eal_core::json::parse(encoded, &storage_limits())
        .map_err(|error| format!("Stored record is not bounded strict JSON: {error}"))
}
pub(super) fn encode(payload: &Value) -> Result<String, String> {
    let limits = storage_limits();
    fn bounds(value: &Value, depth: usize, remaining: &mut usize) -> Result<(), String> {
        if depth > eal_core::json::MAX_JSON_DEPTH || *remaining == 0 {
            return Err("Stored record exceeds the JSON traversal budget".into());
        }
        *remaining -= 1;
        match value {
            Value::Array(values) => {
                for value in values {
                    bounds(value, depth + 1, remaining)?;
                }
            }
            Value::Object(values) => {
                for value in values.values() {
                    bounds(value, depth + 1, remaining)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut remaining = limits.json_nodes;
    bounds(payload, 0, &mut remaining)?;
    struct Counter {
        remaining: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.remaining {
                return Err(std::io::Error::other(
                    "Stored record exceeds the byte budget",
                ));
            }
            self.remaining -= bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(
        Counter {
            remaining: limits.record_bytes,
        },
        payload,
    )
    .map_err(|e| e.to_string())?;
    let compact = eal_core::canonical::canonical_json_bytes(payload, &limits)?;
    if compact.len() > limits.record_bytes {
        return Err("Stored record exceeds the byte budget".into());
    }
    let mut output = Vec::with_capacity(compact.len());
    let mut quoted = false;
    let mut escape = false;
    for byte in compact {
        if output.len() >= limits.record_bytes {
            return Err("Stored record exceeds the byte budget".into());
        }
        output.push(byte);
        if quoted {
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else if byte == b'"' {
            quoted = true;
        } else if matches!(byte, b',' | b':') {
            if output.len() >= limits.record_bytes {
                return Err("Stored record exceeds the byte budget".into());
            }
            output.push(b' ');
        }
    }
    if output.len() > limits.record_bytes {
        return Err("Stored record exceeds the byte budget".into());
    }
    String::from_utf8(output).map_err(|e| e.to_string())
}
