//! A stored collection is assessed only against its exact source and typed context.
use eal_core::{ExecutionLimits, canonical};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Default)]
pub struct CollectionIdentityValidator;

impl CollectionIdentityValidator {
    pub fn validate(
        &self,
        collection: &Value,
        source_digest: &str,
        context: &Value,
        limits: &ExecutionLimits,
    ) -> Result<Value, String> {
        let object = collection
            .as_object()
            .ok_or("Stored collection must be a JSON object")?;
        if object.get("source_digest").and_then(Value::as_str) != Some(source_digest) {
            return Err("Collection differs from the assessment source".into());
        }
        if canonical::canonical_digest(object.get("context").unwrap_or(&Value::Null), limits)?
            != canonical::canonical_digest(context, limits)?
        {
            return Err("Collection differs from the assessment context".into());
        }
        let records_value = object
            .get("records")
            .ok_or("Collection records must map evidence IDs to observations")?;
        let records = object
            .get("records")
            .and_then(Value::as_object)
            .filter(|records| records.values().all(Value::is_object))
            .ok_or("Collection records must map evidence IDs to observations")?;
        // A collection combines independently bounded records. Its admission
        // uses the store's aggregate representation ceiling, rather than
        // spending one observation's JSON allowance across every record.
        let admission = ExecutionLimits {
            json_nodes: crate::store::MAX_STORED_RECORD_BYTES,
            json_depth: eal_core::json::MAX_JSON_DEPTH,
            ..ExecutionLimits::default()
        };
        eal_core::validation::check_json_resources(records_value, &admission)?;
        Ok(Value::Object(records.clone()))
    }
}
