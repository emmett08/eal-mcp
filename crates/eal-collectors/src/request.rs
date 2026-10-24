//! Exact acquisition requests with bounded provider-specific inputs.
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub evidence_id: String,
    pub environment: String,
    pub tool: String,
    pub tool_version: String,
    pub input: Value,
    pub context: Value,
}
impl Request {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|_| "Request must be UTF-8")?;
        let value = eal_runtime::acquisition::strict_json(text)
            .map_err(|_| "Request must be strict JSON")?;
        let result: Self =
            serde_json::from_value(value).map_err(|_| "Invalid EAL acquisition request fields")?;
        for value in [
            &result.evidence_id,
            &result.environment,
            &result.tool,
            &result.tool_version,
        ] {
            if value.is_empty() || value.len() > 1024 {
                return Err("Request identity must be nonempty and bounded".into());
            }
        }
        if !result.context.is_object() || !result.input.is_object() {
            return Err("Request context and input must be JSON objects".into());
        }
        Ok(result)
    }
    pub fn acquisition(&self) -> Value {
        json!({"tool":self.tool,"tool_version":self.tool_version,"input":self.input,"context":self.context})
    }
    pub(crate) fn fields(&self, expected: &[&str]) -> Result<(), String> {
        let object = self.input.as_object().ok_or("Input must be an object")?;
        if object.len() != expected.len() || expected.iter().any(|key| !object.contains_key(*key)) {
            return Err("Input fields differ from the selected collector contract".into());
        }
        Ok(())
    }
    pub(crate) fn matches(&self, key: &str, expected: &str) -> Result<(), String> {
        if self.input[key].as_str() != Some(expected) {
            return Err("Source input differs from the operator-selected resource".into());
        }
        Ok(())
    }
}
