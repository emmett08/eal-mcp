//! Import one scoped review decision without inferring reviewer authority or truth.
use super::invalid;
use crate::Error;
use eal_collector_contracts::{AcquisitionRequest, capability::canonical_identity};
use serde::Deserialize;
use serde_json::Value;

pub const REVIEW_INPUT_SCHEMA: &str = "eal.review-inputs/2";
const MAX_REVIEWS: usize = 4096;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewInputs {
    pub schema: String,
    pub reviews: Vec<Review>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub request: Value,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub value: Value,
}

/// `reviewed_at` is the one original clock. Import time never refreshes it.
pub fn import_review(
    request: &AcquisitionRequest,
    reviews: Vec<Review>,
) -> Result<(Value, String), Error> {
    request.validate().map_err(Error::InvalidInput)?;
    eal_collector_contracts::scoped_review::validate_expectation(&request.input)
        .map_err(Error::InvalidInput)?;
    if reviews.len() > MAX_REVIEWS {
        return Err(invalid("invalid retained review count"));
    }
    let identity = canonical_identity(&request.acquisition()).map_err(Error::InvalidInput)?;
    let mut matched = None;
    for retained in reviews {
        if canonical_identity(&retained.request).map_err(Error::InvalidInput)? != identity {
            continue;
        }
        if matched.is_some() {
            return Err(invalid(
                "retained review repeats the exact acquisition request",
            ));
        }
        let reviewed_at =
            eal_collector_contracts::scoped_review::validate(&request.input, &retained.value)
                .map_err(Error::InvalidInput)?;
        matched = Some((retained.value, reviewed_at));
    }
    matched.ok_or_else(|| invalid("no retained review matches the exact request and context"))
}
