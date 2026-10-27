//! Standalone import of explicitly scoped reviewer records with their original clock.
use eal_adapters::retained::{REVIEW_INPUT_SCHEMA, ReviewInputs, import_review};
use eal_collector_contracts::{AcquisitionRequest, JsonLimits};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

pub fn collect(arguments: &[String], input: &[u8]) -> Result<Value, String> {
    let [flag, path] = arguments else {
        return Err("review-json requires --inputs FILE".into());
    };
    if flag != "--inputs" {
        return Err("review-json accepts only --inputs FILE".into());
    }
    let request = AcquisitionRequest::parse(input, JsonLimits::default())?;
    let bytes = eal_adapters::files::read(Path::new(path), 1024 * 1024, "retained reviews")
        .map_err(|error| error.to_string())?;
    let document: ReviewInputs = serde_json::from_value(
        eal_adapters::parse_json(&bytes).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if document.schema != REVIEW_INPUT_SCHEMA {
        return Err(format!(
            "review-json requires {REVIEW_INPUT_SCHEMA}; migrate old accepted-boolean records explicitly"
        ));
    }
    let (value, reviewed_at) =
        import_review(&request, document.reviews).map_err(|error| error.to_string())?;
    eal_adapters::observation(&request, value, Some(reviewed_at), Some(json!({
        "collector":"eal-collector/review-json/1",
        "inputs_sha256":format!("{:x}",Sha256::digest(&bytes)),
        "interpretation":"retained scoped judgement; correspondence does not establish truth or reviewer independence"
    }))).map_err(|error| error.to_string())
}
