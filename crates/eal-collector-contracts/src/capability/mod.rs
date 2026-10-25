//! Qualified measurement contracts for comparing evidence acquisition routes.
mod identity;
mod matching;
mod ranking;
mod scope;
mod types;
mod validation;

pub use identity::{canonical_identity, schema_identity};
pub use matching::{CompatibilityIssue, CompatibilityResult, compatible};
pub use ranking::{CostDimension, RankedCapability, RankingPolicy, rank_adequate};
pub use scope::{scope_from_input, validate_historical_input, validate_scope_input};
pub use types::*;

pub const CAPABILITY_CATALOGUE_SCHEMA: &str = "eal.capability-catalogue/1";
pub const SUPPORT_CONTRACT_SCHEMA: &str = "eal.support-contract/1";
