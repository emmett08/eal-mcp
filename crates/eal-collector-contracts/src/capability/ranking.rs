//! Explicit lexicographic estimate ranking after hard contract admission.
use super::{matching::compatible_admitted, types::*};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CostDimension {
    ConfigurationEffortSeconds,
    ImplementationEffortSeconds,
    ExecutionRequests,
    ExecutionLatencyMs,
    MonetaryMicrounits,
    TransferBytes,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RankingPolicy {
    /// Earlier dimensions take precedence. Unknown estimates rank after known values.
    pub dimensions: Vec<CostDimension>,
}

impl RankingPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.dimensions.is_empty()
            || self.dimensions.len() > 6
            || self.dimensions.iter().collect::<BTreeSet<_>>().len() != self.dimensions.len()
        {
            return Err("Ranking requires one to six distinct explicit cost dimensions".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RankedCapability {
    pub capability_id: String,
    pub capability_version: String,
    pub estimates: Vec<CostEstimate>,
    pub unknown_dimensions: Vec<CostDimension>,
}

fn estimate(costs: &CapabilityCosts, dimension: CostDimension) -> &CostEstimate {
    match dimension {
        CostDimension::ConfigurationEffortSeconds => &costs.configuration_effort_seconds,
        CostDimension::ImplementationEffortSeconds => &costs.implementation_effort_seconds,
        CostDimension::ExecutionRequests => &costs.execution_requests,
        CostDimension::ExecutionLatencyMs => &costs.execution_latency_ms,
        CostDimension::MonetaryMicrounits => &costs.monetary_microunits,
        CostDimension::TransferBytes => &costs.transfer_bytes,
    }
}

/// Return only contract-adequate candidates. Transport kind supplies no preference.
pub fn rank_adequate(
    requirement: &SupportContract,
    candidates: &[CapabilityDescriptor],
    policy: &RankingPolicy,
) -> Result<Vec<RankedCapability>, String> {
    requirement.validate()?;
    policy.validate()?;
    let mut ids = BTreeSet::new();
    let mut ranked = Vec::new();
    for candidate in candidates {
        candidate.validate()?;
        if !ids.insert(&candidate.id) {
            return Err("Ranking repeats a capability identifier".into());
        }
        if !compatible_admitted(requirement, candidate).adequate {
            continue;
        }
        ranked.push(RankedCapability {
            capability_id: candidate.id.clone(),
            capability_version: candidate.version.clone(),
            estimates: policy
                .dimensions
                .iter()
                .map(|dimension| estimate(&candidate.costs, *dimension).clone())
                .collect(),
            unknown_dimensions: policy
                .dimensions
                .iter()
                .copied()
                .filter(|dimension| estimate(&candidate.costs, *dimension).value.is_none())
                .collect(),
        });
    }
    ranked.sort_by(|left, right| {
        left.estimates
            .iter()
            .map(|estimate| (estimate.value.is_none(), estimate.value.unwrap_or(0)))
            .cmp(
                right
                    .estimates
                    .iter()
                    .map(|estimate| (estimate.value.is_none(), estimate.value.unwrap_or(0))),
            )
            .then_with(|| left.capability_id.cmp(&right.capability_id))
            .then_with(|| left.capability_version.cmp(&right.capability_version))
    });
    Ok(ranked)
}
