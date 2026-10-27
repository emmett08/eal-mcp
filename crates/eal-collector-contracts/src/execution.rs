//! Versioned source-operation graph with requirement-specific evaluation contracts.
pub use crate::retained_judgement::{
    JudgementVerdict, RETAINED_JUDGEMENT_SCHEMA, RetainedJudgement, retained_judgement_schema,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const PLAN_SCHEMA: &str = "eal.evidence-execution-plan/1";

#[path = "execution_dependency.rs"]
mod dependency;
pub use dependency::DependencyRole;

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceExecutionPlan {
    pub schema: String,
    pub acquisitions: Vec<AcquisitionStep>,
    pub derivations: Vec<DerivationStep>,
    pub evaluations: Vec<RequirementEvaluation>,
    #[serde(default)]
    pub scheduling: SchedulingPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SchedulingPolicy {
    pub maximum_concurrent_sources: usize,
    pub per_provider: BTreeMap<String, usize>,
}
impl Default for SchedulingPolicy {
    fn default() -> Self {
        Self {
            maximum_concurrent_sources: 4,
            per_provider: BTreeMap::new(),
        }
    }
}

/// Source identity excludes requirement IDs, prose and acceptance thresholds.
/// Runtime binding supplies actual configuration and authentication fingerprints.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub provider_id: String,
    pub capability_id: String,
    pub capability_version: String,
    pub input_schema_sha256: String,
    pub output_schema_sha256: String,
    pub qualified_contract_sha256: String,
    pub configuration_sha256: String,
    pub authentication_scope_sha256: String,
    #[serde(deserialize_with = "crate::deserialize_value_exact")]
    #[schemars(with = "Value")]
    pub scope: Value,
    #[serde(default)]
    pub historical_interval: Option<HistoricalInterval>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HistoricalInterval {
    pub start: String,
    pub end: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AcquisitionStep {
    pub id: String,
    pub source: SourceIdentity,
    #[serde(deserialize_with = "crate::deserialize_value_exact")]
    #[schemars(with = "Value")]
    pub arguments: Value,
    #[serde(default)]
    pub bindings: Vec<DependencyBinding>,
    #[serde(default)]
    pub for_each: Option<ForEachBinding>,
    #[serde(default)]
    pub availability: SourceAvailability,
    #[serde(default)]
    pub reuse: ReusePolicy,
    /// Original acquisition contract age limit; independent of source identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_age_seconds: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceAvailability {
    #[default]
    Configured,
    Unavailable {
        reason: String,
    },
    ConfigurationMissing {
        requirements: Vec<String>,
    },
    HistoricalUnavailable {
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReusePolicy {
    #[default]
    FreshOrAcquire,
    Refresh,
    RetainedOnly,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputReference {
    pub step_id: String,
    pub pointer: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DependencyBinding {
    pub output: OutputReference,
    pub argument_pointer: String,
    /// Old bindings retain their conservative record-selection meaning.
    #[serde(default, skip_serializing_if = "DependencyRole::is_selection")]
    pub role: DependencyRole,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForEachBinding {
    pub output: OutputReference,
    pub argument_pointer: String,
    pub maximum_items: usize,
    /// An explicit existential route may stop at a validated positive witness.
    /// Requirement evaluations still check their own applicability and challenges.
    pub stop_on_witness: Option<WitnessPredicate>,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WitnessPredicate {
    pub pointer: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present_value"
    )]
    #[schemars(with = "Option<Value>")]
    pub equals: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equals_output: Option<OutputReference>,
}
fn deserialize_present_value<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Value>, D::Error> {
    crate::deserialize_value_exact(deserializer).map(Some)
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DerivationStep {
    pub id: String,
    pub inputs: Vec<OutputReference>,
    pub operation: DerivationOperation,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DerivationOperation {
    Collect,
    Extract {
        pointer: String,
    },
    /// Select named quantities from one result. For a fan-out array, each
    /// field is an array of per-record selections (including empty arrays).
    Project {
        fields: BTreeMap<String, String>,
    },
    /// Select named quantities from one whole result. Pointers address the
    /// result itself even when it is an array, rather than each fan-out record.
    ProjectValue {
        fields: BTreeMap<String, String>,
    },
    Count,
    Sum,
    Mean,
    Minimum,
    Maximum,
    IdentityEqual,
    AnyIdentityEqual,
    Scale {
        numerator: String,
        denominator: String,
        from_unit: String,
        to_unit: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RequirementEvaluation {
    pub id: String,
    pub requirement_id: String,
    pub inputs: Vec<OutputReference>,
    pub criterion: EvaluationCriterion,
    #[serde(default)]
    pub max_age_seconds: Option<u64>,
    #[serde(default)]
    pub required_coverage: RequiredCoverage,
    #[serde(default)]
    pub applicability: Vec<OutputReference>,
    /// Explicit observations or independently derived predicates that establish
    /// the declared coverage. Names and provider assertions imply no coverage.
    #[serde(default)]
    pub coverage: Vec<OutputReference>,
    #[serde(default)]
    pub challenges: Vec<OutputReference>,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RequiredCoverage {
    #[default]
    Unspecified,
    Witness,
    CompleteEnumeration,
    Statistical,
}

#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EvaluationCriterion {
    Equals {
        #[serde(deserialize_with = "crate::deserialize_value_exact")]
        #[schemars(with = "Value")]
        expected: Value,
    },
    Numeric {
        comparison: NumericComparison,
        #[serde(deserialize_with = "crate::deserialize_value_exact")]
        #[schemars(with = "Value")]
        threshold: Value,
        value_unit: String,
        threshold_unit: String,
    },
    AllTrue,
    AnyTrue,
    Judgement {
        criterion: String,
        /// Missing scope represents an unresolved judgement obligation.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        scope: Option<crate::capability::SubjectScope>,
    },
    Unavailable {
        reason: String,
    },
}

impl<'de> Deserialize<'de> for EvaluationCriterion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let value = crate::deserialize_value_exact(deserializer)?;
        let mut fields = value
            .as_object()
            .cloned()
            .ok_or_else(|| D::Error::custom("evaluation criterion requires an object"))?;
        let kind = fields
            .remove("kind")
            .and_then(|value| value.as_str().map(str::to_owned))
            .ok_or_else(|| D::Error::custom("evaluation criterion requires kind"))?;
        let mut take = |name: &str| {
            fields
                .remove(name)
                .ok_or_else(|| D::Error::custom(format!("evaluation criterion requires {name}")))
        };
        let result = match kind.as_str() {
            "equals" => Self::Equals {
                expected: take("expected")?,
            },
            "numeric" => Self::Numeric {
                comparison: serde_json::from_value(take("comparison")?)
                    .map_err(D::Error::custom)?,
                threshold: take("threshold")?,
                value_unit: serde_json::from_value(take("value_unit")?)
                    .map_err(D::Error::custom)?,
                threshold_unit: serde_json::from_value(take("threshold_unit")?)
                    .map_err(D::Error::custom)?,
            },
            "all_true" => Self::AllTrue,
            "any_true" => Self::AnyTrue,
            "judgement" => {
                let criterion =
                    serde_json::from_value(take("criterion")?).map_err(D::Error::custom)?;
                let scope = fields
                    .remove("scope")
                    .map(serde_json::from_value)
                    .transpose()
                    .map_err(D::Error::custom)?
                    .flatten();
                Self::Judgement { criterion, scope }
            }
            "unavailable" => Self::Unavailable {
                reason: serde_json::from_value(take("reason")?).map_err(D::Error::custom)?,
            },
            _ => return Err(D::Error::custom("unknown evaluation criterion kind")),
        };
        if !fields.is_empty() {
            return Err(D::Error::custom("unknown evaluation criterion fields"));
        }
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NumericComparison {
    LessThan,
    AtMost,
    Equal,
    AtLeast,
    GreaterThan,
}

pub fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn identifier(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}
pub fn valid_pointer(value: &str) -> bool {
    if value.len() > 2048 || (!value.is_empty() && !value.starts_with('/')) {
        return false;
    }
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character == '~' && !matches!(chars.next(), Some('0' | '1')) {
            return false;
        }
    }
    true
}

/// Canonical identity uses the shared exact JSON contract.
pub fn canonical_digest(value: &impl Serialize) -> Result<String, String> {
    crate::capability::canonical_identity(value)
}

pub fn plan_schema() -> Value {
    serde_json::to_value(schemars::schema_for!(EvidenceExecutionPlan))
        .expect("execution schema serialises")
}

impl SourceIdentity {
    /// Check only declared measurement identity. IO adapters separately bind
    /// actual authentication/configuration fingerprints and derived scope.
    pub fn validate_capability(
        &self,
        capability: &crate::capability::CapabilityDescriptor,
    ) -> Result<(), String> {
        if self.provider_id != capability.provider.id
            || self.capability_id != capability.id
            || self.capability_version != capability.version
            || self.input_schema_sha256 != capability.input_schema_sha256
            || self.output_schema_sha256 != capability.output_schema_sha256
            || self.qualified_contract_sha256 != capability.measurement_identity()?
        {
            return Err("planned source differs from the qualified capability identity".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        if ![
            &self.provider_id,
            &self.capability_id,
            &self.capability_version,
        ]
        .into_iter()
        .all(|value| identifier(value))
            || ![
                &self.input_schema_sha256,
                &self.output_schema_sha256,
                &self.qualified_contract_sha256,
                &self.configuration_sha256,
                &self.authentication_scope_sha256,
            ]
            .into_iter()
            .all(|value| valid_digest(value))
            || !self.scope.is_object()
        {
            return Err("source identity requires provider, version, schema, configuration, authentication and object scope".into());
        }
        if let Some(interval) = &self.historical_interval {
            let start = chrono::DateTime::parse_from_rfc3339(&interval.start)
                .map_err(|_| "historical start requires RFC3339")?;
            let end = chrono::DateTime::parse_from_rfc3339(&interval.end)
                .map_err(|_| "historical end requires RFC3339")?;
            if start >= end {
                return Err("historical interval must be ordered".into());
            }
        }
        Ok(())
    }
}

impl EvidenceExecutionPlan {
    // Physical collectors can expose discovery quantities through pure projections.
    // Arithmetic, joins and fan-out must never disguise record selection as metadata.
    fn discovery_source(&self, id: &str) -> Result<&AcquisitionStep, String> {
        let mut current = id;
        let mut visited = BTreeSet::new();
        while visited.insert(current) && visited.len() <= 4096 {
            if let Some(source) = self.acquisitions.iter().find(|source| source.id == current) {
                if source.for_each.is_some()
                    || source
                        .bindings
                        .iter()
                        .any(|binding| binding.role.is_selection())
                {
                    return Err("discovery metadata cannot originate from a fan-out or selection-dependent source".into());
                }
                return Ok(source);
            }
            let derived = self
                .derivations
                .iter()
                .find(|derived| derived.id == current)
                .ok_or("discovery dependency source is unavailable")?;
            if derived.inputs.len() != 1
                || !matches!(
                    derived.operation,
                    DerivationOperation::Extract { .. }
                        | DerivationOperation::Project { .. }
                        | DerivationOperation::ProjectValue { .. }
                )
            {
                return Err(
                    "discovery metadata requires one source with only projection or extraction"
                        .into(),
                );
            }
            current = &derived.inputs[0].step_id;
        }
        Err("discovery dependency lineage is cyclic or exceeds its node bound".into())
    }

    fn discovery_source_scope(&self, id: &str) -> Result<&Value, String> {
        Ok(&self.discovery_source(id)?.source.scope)
    }

    /// Validate graph references and finite resource limits before performing IO.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != PLAN_SCHEMA {
            return Err("unsupported evidence execution plan schema".into());
        }
        if self.acquisitions.len() + self.derivations.len() > 4096
            || self.evaluations.len() > 4096
            || !(1..=64).contains(&self.scheduling.maximum_concurrent_sources)
            || self
                .scheduling
                .per_provider
                .iter()
                .any(|(id, limit)| !identifier(id) || !(1..=64).contains(limit))
        {
            return Err("evidence graph exceeds declared scheduling or node limits".into());
        }
        canonical_digest(self)?;
        let mut nodes = BTreeMap::<&str, Vec<&OutputReference>>::new();
        for step in &self.acquisitions {
            step.source.validate()?;
            match &step.availability {
                SourceAvailability::Unavailable { reason }
                | SourceAvailability::HistoricalUnavailable { reason }
                    if reason.trim().is_empty() || reason.len() > 4096 =>
                {
                    return Err("unavailable source requires a bounded explicit gap".into());
                }
                SourceAvailability::ConfigurationMissing { requirements }
                    if requirements.is_empty()
                        || requirements.len() > 128
                        || requirements
                            .iter()
                            .any(|value| value.trim().is_empty() || value.len() > 4096) =>
                {
                    return Err(
                        "missing source configuration requires bounded explicit prerequisites"
                            .into(),
                    );
                }
                _ => {}
            }
            if !step.arguments.is_object() {
                return Err("acquisition arguments require a JSON object".into());
            }
            let mut references: Vec<_> = step
                .bindings
                .iter()
                .map(|binding| &binding.output)
                .collect();
            if step.bindings.iter().any(|binding| {
                !valid_pointer(&binding.argument_pointer)
                    || step.arguments.pointer(&binding.argument_pointer).is_none()
            }) {
                return Err("dependency binding requires an existing JSON argument pointer".into());
            }
            for binding in &step.bindings {
                if !binding.role.is_selection() {
                    let scope = self
                        .discovery_source_scope(&binding.output.step_id)
                        .map_err(|error| {
                            format!(
                                "acquisition {} discovery dependency {}: {error}",
                                step.id, binding.output.step_id
                            )
                        })?;
                    binding
                        .role
                        .validate_scopes(scope, &step.source.scope)
                        .map_err(|error| {
                            format!(
                                "acquisition {} discovery dependency {}: {error}",
                                step.id, binding.output.step_id
                            )
                        })?;
                }
            }
            if let Some(fanout) = &step.for_each {
                if !(1..=1024).contains(&fanout.maximum_items)
                    || !valid_pointer(&fanout.argument_pointer)
                    || step.arguments.pointer(&fanout.argument_pointer).is_none()
                    || fanout
                        .stop_on_witness
                        .as_ref()
                        .is_some_and(|predicate| !valid_pointer(&predicate.pointer))
                {
                    return Err(
                        "fan-out requires bounded items and valid argument/witness pointers".into(),
                    );
                }
                if let Some(predicate) = &fanout.stop_on_witness {
                    if predicate.equals.is_some() == predicate.equals_output.is_some() {
                        return Err(
                            "witness predicate requires exactly one literal or upstream output"
                                .into(),
                        );
                    }
                    if let Some(output) = &predicate.equals_output {
                        references.push(output);
                    }
                }
                references.push(&fanout.output);
            }
            if !identifier(&step.id) || nodes.insert(&step.id, references).is_some() {
                return Err("execution step IDs must be unique and bounded".into());
            }
        }
        for step in &self.derivations {
            if step.inputs.is_empty()
                || !identifier(&step.id)
                || nodes
                    .insert(&step.id, step.inputs.iter().collect())
                    .is_some()
            {
                return Err("derivation requires inputs and a unique bounded step ID".into());
            }
            if matches!(&step.operation,DerivationOperation::Extract{pointer} if !valid_pointer(pointer))
            {
                return Err("derivation requires a valid JSON pointer".into());
            }
        }
        for step in &self.derivations {
            match &step.operation {
                DerivationOperation::Project { fields }
                | DerivationOperation::ProjectValue { fields }
                    if step.inputs.len() != 1
                        || fields.is_empty()
                        || fields.len() > 256
                        || fields.iter().any(|(name, pointer)| {
                            !identifier(name) || !valid_pointer(pointer)
                        }) =>
                {
                    return Err(
                        "projection requires one input and bounded explicit field names/pointers"
                            .into(),
                    );
                }
                DerivationOperation::Extract { .. } | DerivationOperation::Scale { .. }
                    if step.inputs.len() != 1 =>
                {
                    return Err("unary derivation requires exactly one input".into());
                }
                DerivationOperation::AnyIdentityEqual if step.inputs.len() != 2 => {
                    return Err("identity membership requires exactly two inputs".into());
                }
                DerivationOperation::IdentityEqual if step.inputs.len() < 2 => {
                    return Err("identity comparison requires at least two inputs".into());
                }
                _ => {}
            }
        }
        for (id, references) in &nodes {
            if references.iter().any(|reference| {
                reference.step_id == *id
                    || !nodes.contains_key(reference.step_id.as_str())
                    || !valid_pointer(&reference.pointer)
            }) {
                return Err(
                    "execution graph references an unknown step, itself or an invalid JSON pointer"
                        .into(),
                );
            }
        }
        let mut admitted = BTreeSet::new();
        loop {
            let before = admitted.len();
            for (id, refs) in &nodes {
                if refs
                    .iter()
                    .all(|reference| admitted.contains(reference.step_id.as_str()))
                {
                    admitted.insert(*id);
                }
            }
            if admitted.len() == nodes.len() {
                break;
            }
            if admitted.len() == before {
                return Err("execution graph contains a dependency cycle".into());
            }
        }
        let mut evaluations = BTreeSet::new();
        for evaluation in &self.evaluations {
            if !identifier(&evaluation.id)
                || !identifier(&evaluation.requirement_id)
                || !evaluations.insert(evaluation.id.as_str())
                || (evaluation.inputs.is_empty()
                    && !matches!(
                        evaluation.criterion,
                        EvaluationCriterion::Judgement { .. }
                            | EvaluationCriterion::Unavailable { .. }
                    ))
            {
                return Err(
                    "evaluations require unique IDs, requirement IDs and source inputs".into(),
                );
            }
            for reference in evaluation
                .inputs
                .iter()
                .chain(&evaluation.applicability)
                .chain(&evaluation.challenges)
                .chain(&evaluation.coverage)
            {
                if !nodes.contains_key(reference.step_id.as_str())
                    || !valid_pointer(&reference.pointer)
                {
                    return Err("evaluation references an unknown step or invalid pointer".into());
                }
            }
            if matches!(
                evaluation.criterion,
                EvaluationCriterion::Equals { .. } | EvaluationCriterion::Numeric { .. }
            ) && evaluation.inputs.len() != 1
            {
                return Err("scalar evaluation requires exactly one input".into());
            }
            if let EvaluationCriterion::Judgement {
                scope: Some(scope), ..
            } = &evaluation.criterion
            {
                scope.validate()?;
            }
            match &evaluation.criterion {
                EvaluationCriterion::Judgement { criterion, .. } if criterion.trim().is_empty() => {
                    return Err("judgement requires an explicit criterion".into());
                }
                EvaluationCriterion::Unavailable { reason } if reason.trim().is_empty() => {
                    return Err("unavailable support requires an explicit gap".into());
                }
                EvaluationCriterion::Numeric {
                    value_unit,
                    threshold_unit,
                    ..
                } if value_unit.trim().is_empty() || threshold_unit.trim().is_empty() => {
                    return Err("numeric evaluation requires explicit units".into());
                }
                _ => {}
            }
        }
        Ok(())
    }
}
