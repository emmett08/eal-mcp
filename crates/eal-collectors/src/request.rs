//! Native object-input profile over the authoritative acquisition request model.
use eal_collector_contracts::{AcquisitionRequest, JsonLimits};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Request(AcquisitionRequest);
impl Request {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let request = Self(AcquisitionRequest::parse(
            bytes,
            JsonLimits {
                max_bytes: 1_048_576,
                ..JsonLimits::default()
            },
        )?);
        request.validate_native()?;
        Ok(request)
    }
    pub(crate) fn validate_native(&self) -> Result<(), String> {
        self.0.validate()?;
        if [
            &self.evidence_id,
            &self.environment,
            &self.tool,
            &self.tool_version,
        ]
        .iter()
        .any(|s| s.len() > 1024)
        {
            return Err("Native request identity must be bounded".into());
        }
        if !self.input.is_object() {
            return Err("Native collector input must be a JSON object".into());
        }
        Ok(())
    }
}
impl Deref for Request {
    type Target = AcquisitionRequest;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Request {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
