//! Versioned, duplicate-key-rejecting JSON framing for one isolated method call.
use eal_core::methods::MethodError;
use eal_core::{ExecutionLimits, canonical, json};
use serde::Serialize;
use serde_json::Value;

pub const VERSION: &str = "eal-method-worker/1";
pub const READY: &[u8] = b"{\"protocol\":\"eal-method-worker/1\",\"ready\":true}\n";
pub const MAX_REQUEST_BYTES: usize = 20 * 1024 * 1024;

#[derive(Serialize)]
pub struct Request {
    pub protocol: String,
    pub method: String,
    pub contract: Value,
    pub payload: Value,
    pub limits: ExecutionLimits,
}
impl Request {
    pub fn decode(mut value: Value) -> Result<Self, MethodError> {
        let fields = ["protocol", "method", "contract", "payload", "limits"];
        let object = value
            .as_object_mut()
            .filter(|o| o.len() == fields.len() && fields.iter().all(|f| o.contains_key(*f)))
            .ok_or("Method request has invalid fields")?;
        let protocol = match object.remove("protocol") {
            Some(Value::String(s)) => s,
            _ => return Err("Method request protocol must be a string".into()),
        };
        let method = match object.remove("method") {
            Some(Value::String(s)) => s,
            _ => return Err("Method request method must be a string".into()),
        };
        // Move arbitrary JSON directly: Value::deserialize has private interchange
        // key semantics that must not reinterpret ordinary user-authored objects.
        let contract = object.remove("contract").unwrap();
        let payload = object.remove("payload").unwrap();
        let limits =
            serde_json::from_value(object.remove("limits").unwrap()).map_err(|e| e.to_string())?;
        Ok(Self {
            protocol,
            method,
            contract,
            payload,
            limits,
        })
    }
}
pub fn wire_limits(bytes: usize) -> ExecutionLimits {
    ExecutionLimits {
        json_depth: 128,
        json_nodes: 1_000_000,
        record_bytes: bytes,
        ..Default::default()
    }
}
pub fn encode(value: &Value, maximum: usize) -> Result<Vec<u8>, MethodError> {
    let mut bytes = canonical::canonical_json_bytes(value, &wire_limits(maximum))?;
    if bytes.len() > maximum {
        return Err("Method protocol message exceeds byte limit".into());
    }
    bytes.push(b'\n');
    Ok(bytes)
}
pub fn strict_value(bytes: &[u8], maximum: usize) -> Result<Value, MethodError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| MethodError::Invalid("Method protocol requires UTF-8 JSON".into()))?;
    json::parse(text, &wire_limits(maximum)).map_err(MethodError::Invalid)
}
pub fn decode_response(stdout: &[u8], maximum: usize) -> Result<Value, MethodError> {
    let remaining = stdout
        .strip_prefix(READY)
        .ok_or("Method worker returned an invalid startup frame")?;
    let frame = remaining
        .strip_suffix(b"\n")
        .ok_or("Method worker response requires a final newline")?;
    if frame.contains(&b'\n') || frame.len() > maximum {
        return Err("Method worker returned an invalid response frame".into());
    }
    let value = strict_value(frame, maximum)?;
    let object = value
        .as_object()
        .ok_or("Method worker response must be an object")?;
    match object.get("ok").and_then(Value::as_bool) {
        Some(true) if object.len() == 2 && object.contains_key("output") => {
            Ok(object["output"].clone())
        }
        Some(false)
            if object.len() == 3
                && object.get("incomplete").is_some_and(Value::is_boolean)
                && object.get("error").is_some_and(Value::is_string) =>
        {
            let error = object["error"].as_str().unwrap();
            if error.len() > 512 {
                return Err("Method worker error exceeds byte limit".into());
            }
            Err(if object["incomplete"] == true {
                MethodError::Incomplete(error.to_owned())
            } else {
                MethodError::Invalid(error.to_owned())
            })
        }
        _ => Err("Method worker returned an invalid response object".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn literal_private_interchange_keys_remain_json_objects() {
        let payload=json::parse("{\"$serde_json::private::Number\":\"1\",\"nested\":{\"$serde_json::private::RawValue\":\"null\"}}",&wire_limits(4096)).unwrap();
        let request = json!({"protocol":VERSION,"method":"example/1","contract":payload.clone(),"payload":payload.clone(),"limits":{}});
        let decoded =
            Request::decode(strict_value(&encode(&request, 4096).unwrap(), 4096).unwrap()).unwrap();
        assert_eq!(decoded.payload, payload);
        assert_eq!(decoded.contract, payload);
    }
}
