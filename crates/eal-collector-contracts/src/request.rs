//! Shared acquisition identity and observation-envelope field ownership.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const ACQUISITION_CONTRACT: &str = "eal.acquisition/1";

/// Decode this model only after strict, bounded JSON parsing at the IO boundary.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AcquisitionRequest {
    pub evidence_id: String,
    pub environment: String,
    pub tool: String,
    pub tool_version: String,
    pub input: Value,
    pub context: Value,
}
impl AcquisitionRequest {
    pub fn validate(&self) -> Result<(), String> {
        for value in [
            &self.evidence_id,
            &self.environment,
            &self.tool,
            &self.tool_version,
        ] {
            if value.is_empty() {
                return Err("Request identity must be nonempty".into());
            }
        }
        if !self.context.is_object() {
            return Err("Request context must be a JSON object".into());
        }
        Ok(())
    }
    pub fn acquisition(&self) -> Value {
        json!({"tool":self.tool,"tool_version":self.tool_version,"input":self.input,"context":self.context})
    }
    pub fn fields(&self, expected: &[&str]) -> Result<(), String> {
        let object = self.input.as_object().ok_or("Input must be an object")?;
        if object.len() != expected.len() || expected.iter().any(|key| !object.contains_key(*key)) {
            return Err("Input fields differ from the selected collector contract".into());
        }
        Ok(())
    }
    pub fn matches(&self, key: &str, expected: &str) -> Result<(), String> {
        if self.input[key].as_str() != Some(expected) {
            return Err("Source input differs from the operator-selected resource".into());
        }
        Ok(())
    }
}

/// IO adapters retain age and correspondence checks in the runtime.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationEnvelope {
    pub value: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}
