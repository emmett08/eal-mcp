//! Exact opaque JSON admission for application workers outside EAL numeric lowering.
use super::WorkerError;
use serde_json::Value;
use std::io::Write;

pub(super) fn context(value: &Value) -> Result<Vec<u8>, WorkerError> {
    let mut pending = vec![(value, 0usize)];
    let mut visited = 0;
    while let Some((value, depth)) = pending.pop() {
        visited += 1;
        if depth > 128 || visited > 1_000_000 {
            return Err(WorkerError::Invalid(
                "Worker context exceeds structural bounds".into(),
            ));
        }
        let children = match value {
            Value::Object(values) => values.len(),
            Value::Array(values) => values.len(),
            _ => 0,
        };
        if children > 1_000_000 - visited - pending.len() {
            return Err(WorkerError::Invalid(
                "Worker context exceeds structural bounds".into(),
            ));
        }
        match value {
            Value::Object(values) => pending.extend(values.values().map(|v| (v, depth + 1))),
            Value::Array(values) => pending.extend(values.iter().map(|v| (v, depth + 1))),
            _ => {}
        }
    }
    struct Bounded(Vec<u8>);
    impl Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > 4 * 1024 * 1024 - 1 - self.0.len() {
                return Err(std::io::Error::other(
                    "Worker context exceeds its byte bound",
                ));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut bytes = Bounded(Vec::new());
    serde_json::to_writer(&mut bytes, value)
        .map_err(|_| WorkerError::Invalid("Worker context exceeds its byte bound".into()))?;
    bytes.0.push(b'\n');
    Ok(bytes.0)
}
pub(super) fn response(bytes: &[u8]) -> Result<Value, String> {
    let value = eal_collector_contracts::strict_json_exact(
        bytes,
        eal_collector_contracts::JsonLimits {
            max_bytes: 16 * 1024 * 1024,
            max_depth: 128,
            max_nodes: 1_000_000,
        },
    )?;
    if !value.is_object() {
        return Err("Worker response must be an object".into());
    }
    Ok(value)
}
