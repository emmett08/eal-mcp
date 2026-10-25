//! Hard compatibility checks against qualified contracts, independently of cost.
use super::types::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CompatibilityIssue {
    InvalidRequirement {
        diagnostic: String,
    },
    InvalidCapability {
        diagnostic: String,
    },
    MissingQualifiedSemantics,
    SubjectKindMismatch {
        required: String,
    },
    MeasurementMethodMismatch {
        acceptable: Vec<String>,
    },
    UnsupportedSupportType {
        required: SupportType,
    },
    MissingIdentityDimension {
        dimension: String,
    },
    ScopeBindingUnavailable {
        dimension: String,
    },
    UnboundScopeSelector {
        dimension: String,
    },
    ScopeMismatch {
        dimension: String,
    },
    MissingQuantity {
        quantity: String,
    },
    UnitMismatch {
        quantity: String,
        required: String,
        offered: String,
    },
    CoverageMismatch {
        required: LogicalStrength,
        population: Option<String>,
    },
    HistoricalIntervalUnsupported,
    HistoricalInputBindingUnavailable,
    OriginalObservationTimeUnavailable,
    TemporalUncertaintyUnknown,
    TemporalUncertaintyExceeded {
        required_seconds: u64,
        offered_seconds: u64,
    },
    MissingSourceAuthority {
        authority: String,
    },
    TransformationHistoryUnavailable,
    OutputSchemaMismatch,
}

impl CompatibilityIssue {
    /// A missing scope selection requires configuration rather than new source code.
    pub fn is_configuration_gap(&self) -> bool {
        matches!(self, Self::UnboundScopeSelector { .. })
    }

    pub fn requires_qualification(&self) -> bool {
        matches!(
            self,
            Self::MissingQualifiedSemantics
                | Self::TemporalUncertaintyUnknown
                | Self::ScopeBindingUnavailable { .. }
                | Self::HistoricalInputBindingUnavailable
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityResult {
    /// Contract compatibility does not assert a successful acquisition or acceptance.
    pub adequate: bool,
    pub issues: Vec<CompatibilityIssue>,
    pub limitations: Vec<String>,
}

pub fn compatible(
    requirement: &SupportContract,
    capability: &CapabilityDescriptor,
) -> CompatibilityResult {
    let mut issues = Vec::new();
    if let Err(diagnostic) = requirement.validate() {
        issues.push(CompatibilityIssue::InvalidRequirement { diagnostic });
    }
    if let Err(diagnostic) = capability.validate() {
        issues.push(CompatibilityIssue::InvalidCapability { diagnostic });
    }
    if !issues.is_empty() {
        return CompatibilityResult {
            adequate: false,
            issues,
            limitations: Vec::new(),
        };
    }
    compatible_admitted(requirement, capability)
}

/// Call only after both contracts have passed their public admission checks.
pub(super) fn compatible_admitted(
    requirement: &SupportContract,
    capability: &CapabilityDescriptor,
) -> CompatibilityResult {
    let mut issues = Vec::new();
    let Some(semantics) = &capability.semantics else {
        return CompatibilityResult {
            adequate: false,
            issues: vec![CompatibilityIssue::MissingQualifiedSemantics],
            limitations: Vec::new(),
        };
    };
    if !semantics.subject_kinds.contains(&requirement.scope.kind) {
        issues.push(CompatibilityIssue::SubjectKindMismatch {
            required: requirement.scope.kind.clone(),
        });
    }
    if !requirement
        .acceptable_methods
        .iter()
        .any(|method| semantics.methods.contains(method))
    {
        issues.push(CompatibilityIssue::MeasurementMethodMismatch {
            acceptable: requirement.acceptable_methods.clone(),
        });
    }
    if !semantics.support_types.contains(&requirement.support_type) {
        issues.push(CompatibilityIssue::UnsupportedSupportType {
            required: requirement.support_type,
        });
    }
    for dimension in &requirement.scope.identity_dimensions {
        if !semantics.identity_dimensions.contains(dimension) {
            issues.push(CompatibilityIssue::MissingIdentityDimension {
                dimension: dimension.clone(),
            });
        }
    }
    for dimension in requirement.scope.selectors.keys() {
        if !semantics.scope_bindings.contains_key(dimension)
            && !semantics.scope_constraints.contains_key(dimension)
        {
            issues.push(CompatibilityIssue::ScopeBindingUnavailable {
                dimension: dimension.clone(),
            });
        }
    }
    for (dimension, value) in &semantics.scope_constraints {
        match requirement.scope.selectors.get(dimension) {
            None => issues.push(CompatibilityIssue::UnboundScopeSelector {
                dimension: dimension.clone(),
            }),
            Some(required) if required != value => issues.push(CompatibilityIssue::ScopeMismatch {
                dimension: dimension.clone(),
            }),
            Some(_) => {}
        }
    }
    for quantity in &requirement.quantities {
        match semantics
            .quantities
            .iter()
            .find(|offered| offered.name == quantity.name)
        {
            None => issues.push(CompatibilityIssue::MissingQuantity {
                quantity: quantity.name.clone(),
            }),
            Some(offered) if offered.unit != quantity.unit => {
                issues.push(CompatibilityIssue::UnitMismatch {
                    quantity: quantity.name.clone(),
                    required: quantity.unit.clone(),
                    offered: offered.unit.clone(),
                })
            }
            Some(_) => {}
        }
    }
    if !semantics.coverage.iter().any(|offered| {
        (offered.strength == requirement.logical_strength
            || (requirement.logical_strength == LogicalStrength::ExistentialWitness
                && offered.strength == LogicalStrength::CompleteEnumeration))
            && requirement
                .population
                .as_ref()
                .is_none_or(|population| offered.population.as_ref() == Some(population))
    }) {
        issues.push(CompatibilityIssue::CoverageMismatch {
            required: requirement.logical_strength,
            population: requirement.population.clone(),
        });
    }
    if requirement.time.historical_interval.is_some() && !semantics.time.historical_intervals {
        issues.push(CompatibilityIssue::HistoricalIntervalUnsupported);
    }
    if requirement.time.historical_interval.is_some()
        && semantics.time.historical_intervals
        && semantics.time.historical_input.is_none()
    {
        issues.push(CompatibilityIssue::HistoricalInputBindingUnavailable);
    }
    if (requirement.time.require_original_observation_time
        || requirement.time.max_age_seconds.is_some())
        && !semantics.time.original_observation_time
    {
        issues.push(CompatibilityIssue::OriginalObservationTimeUnavailable);
    }
    if let Some(required) = requirement.time.max_uncertainty_seconds {
        match semantics.time.uncertainty_seconds {
            None => issues.push(CompatibilityIssue::TemporalUncertaintyUnknown),
            Some(offered) if offered > required => {
                issues.push(CompatibilityIssue::TemporalUncertaintyExceeded {
                    required_seconds: required,
                    offered_seconds: offered,
                })
            }
            Some(_) => {}
        }
    }
    for authority in &requirement.provenance.required_authorities {
        if !semantics.provenance.authorities.contains(authority) {
            issues.push(CompatibilityIssue::MissingSourceAuthority {
                authority: authority.clone(),
            });
        }
    }
    if requirement.provenance.transformation_history && !semantics.provenance.transformation_history
    {
        issues.push(CompatibilityIssue::TransformationHistoryUnavailable);
    }
    if requirement
        .expected_output_schema_sha256
        .as_ref()
        .is_some_and(|identity| identity != &capability.output_schema_sha256)
    {
        issues.push(CompatibilityIssue::OutputSchemaMismatch);
    }
    CompatibilityResult {
        adequate: issues.is_empty(),
        issues,
        limitations: semantics.limitations.clone(),
    }
}
