//! Pure qualified identity/provenance shared by IO backends and consumers.
use super::{CapabilityDescriptor, LogicalStrength, SupportType};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualifiedSupportClaims {
    pub qualified_support_types: Vec<SupportType>,
    pub qualified_logical_strengths: Vec<LogicalStrength>,
}
impl QualifiedSupportClaims {
    pub fn permits_judgement(&self) -> bool {
        self.qualified_support_types
            .contains(&SupportType::Judgement)
            && self
                .qualified_logical_strengths
                .contains(&LogicalStrength::Judgement)
    }
    /// Parse only executor-owned metadata. Source response fields do not confer qualification.
    pub fn from_provenance(value: &Value) -> Result<Self, String> {
        serde_json::from_value(value.clone())
            .map_err(|_| "qualified support provenance is absent or malformed".into())
    }
}
impl CapabilityDescriptor {
    pub fn qualified_provenance(&self) -> Result<Value, String> {
        let identity = self.measurement_identity()?;
        let semantics = self
            .semantics
            .as_ref()
            .ok_or("qualified measurement semantics are absent")?;
        let claims = QualifiedSupportClaims {
            qualified_support_types: semantics.support_types.clone(),
            qualified_logical_strengths: semantics
                .coverage
                .iter()
                .map(|coverage| coverage.strength)
                .collect(),
        };
        let mut value = json!({"provider":self.provider.id,"capability":self.id,"capability_version":self.version,"qualification_reference":semantics.qualification_reference,"input_schema_sha256":self.input_schema_sha256,"output_schema_sha256":self.output_schema_sha256,"qualified_contract_sha256":identity});
        // This executor-owned metadata is retained with observations. It binds
        // population coverage to the qualified quantity and output pointers.
        if semantics
            .coverage
            .iter()
            .any(|entry| entry.population_quantity.is_some())
        {
            value["qualified_quantities"] = serde_json::to_value(&semantics.quantities)
                .map_err(|_| "cannot encode qualified quantities")?;
            value["qualified_coverage"] = serde_json::to_value(&semantics.coverage)
                .map_err(|_| "cannot encode qualified coverage")?;
        }
        value.as_object_mut().expect("provenance object").extend(
            serde_json::to_value(claims)
                .map_err(|_| "cannot encode qualified support claims")?
                .as_object()
                .expect("claims object")
                .clone(),
        );
        Ok(value)
    }
}
