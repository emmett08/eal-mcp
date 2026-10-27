//! Exact bounded canonical request encoding.
use super::{AcquisitionError, MAX_REQUEST_BYTES};
use eal_core::{ExecutionLimits, canonical};
use serde_json::Value;
/// Python's sorted, ASCII-escaped JSON request plus one newline, bounded while writing.
pub fn encode_request(
    value: &Value,
    limits: &ExecutionLimits,
) -> Result<Vec<u8>, AcquisitionError> {
    limits.validate()?;
    let mut writer = RequestWriter {
        bytes: Vec::new(),
        remaining: limits.json_nodes,
        limits,
    };
    writer.value(value, 0)?;
    writer.append(b"\n")?;
    Ok(writer.bytes)
}
struct RequestWriter<'a> {
    bytes: Vec<u8>,
    remaining: usize,
    limits: &'a ExecutionLimits,
}
impl RequestWriter<'_> {
    fn append(&mut self, bytes: &[u8]) -> Result<(), AcquisitionError> {
        if self.bytes.len().saturating_add(bytes.len()) > MAX_REQUEST_BYTES {
            return Err(format!("Tool request exceeds {MAX_REQUEST_BYTES} bytes").into());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn string(&mut self, text: &str) -> Result<(), AcquisitionError> {
        self.append(b"\"")?;
        for c in text.chars() {
            match c {
                '"' => self.append(b"\\\"")?,
                '\\' => self.append(b"\\\\")?,
                '\u{08}' => self.append(b"\\b")?,
                '\u{0c}' => self.append(b"\\f")?,
                '\n' => self.append(b"\\n")?,
                '\r' => self.append(b"\\r")?,
                '\t' => self.append(b"\\t")?,
                c if (' '..='~').contains(&c) => self.append(&[c as u8])?,
                c if (c as u32) <= 0xffff => {
                    self.append(format!("\\u{:04x}", c as u32).as_bytes())?
                }
                c => {
                    let v = c as u32 - 0x10000;
                    self.append(
                        format!(
                            "\\u{:04x}\\u{:04x}",
                            0xd800 + (v >> 10),
                            0xdc00 + (v & 0x3ff)
                        )
                        .as_bytes(),
                    )?
                }
            }
        }
        self.append(b"\"")
    }
    fn value(&mut self, value: &Value, depth: usize) -> Result<(), AcquisitionError> {
        if self.remaining == 0 || depth > self.limits.json_depth.min(128) {
            return Err("JSON resource limit exceeded".into());
        }
        self.remaining -= 1;
        match value {
            Value::Null => self.append(b"null"),
            Value::Bool(true) => self.append(b"true"),
            Value::Bool(false) => self.append(b"false"),
            Value::String(value) => self.string(value),
            Value::Number(value) => {
                if value.as_str().len() > MAX_REQUEST_BYTES - self.bytes.len() {
                    return Err(format!("Tool request exceeds {MAX_REQUEST_BYTES} bytes").into());
                }
                self.append(&canonical::canonical_json_bytes(
                    &Value::Number(value.clone()),
                    self.limits,
                )?)
            }
            Value::Array(values) => {
                if values.len() > self.remaining {
                    return Err("JSON resource limit exceeded".into());
                }
                self.append(b"[")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        self.append(b", ")?
                    }
                    self.value(value, depth + 1)?
                }
                self.append(b"]")
            }
            Value::Object(values) => {
                if values.len() > self.remaining {
                    return Err("JSON resource limit exceeded".into());
                }
                self.append(b"{")?;
                let mut keys = values.keys().collect::<Vec<_>>();
                keys.sort();
                for (i, key) in keys.into_iter().enumerate() {
                    if i > 0 {
                        self.append(b", ")?
                    }
                    self.string(key)?;
                    self.append(b": ")?;
                    self.value(&values[key], depth + 1)?
                }
                self.append(b"}")
            }
        }
    }
}
