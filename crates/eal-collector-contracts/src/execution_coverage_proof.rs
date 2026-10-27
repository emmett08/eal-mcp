//! Explicit witnesses for narrowly verified evidence transformations.
use super::{
    AcquisitionStep, DerivationOperation, EvaluationCriterion, EvidenceExecutionPlan,
    RequiredCoverage, identifier, valid_pointer,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// A per-observation proof, not a claim about a calculator's whole input space.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CoverageProof {
    IdentifierAuditV1 {
        source_step_id: String,
        source_acquisition_id: String,
        source_qualified_contract_sha256: String,
        population: String,
        population_quantity: String,
        source_raw_items_pointer: String,
        source_raw_complete_pointer: String,
        source_items_pointer: String,
        source_complete_pointer: String,
        source_identity_pointer: String,
        argument_items_pointer: String,
        argument_complete_pointer: String,
        argument_identity_pointer: String,
        result_count_pointer: String,
        result_distinct_pointer: String,
        result_complete_pointer: String,
        result_identity_pointer: String,
        item_id_pointer: String,
        identity_scope_key: String,
        #[serde(default)]
        echoes: BTreeMap<String, String>,
    },
}

impl CoverageProof {
    pub fn source_step_id(&self) -> &str {
        match self {
            Self::IdentifierAuditV1 { source_step_id, .. } => source_step_id,
        }
    }

    /// Bind each required calculator input to the same selected source.
    pub(super) fn validate(
        &self,
        step: &AcquisitionStep,
        source_step: &AcquisitionStep,
    ) -> Result<(), String> {
        let Self::IdentifierAuditV1 {
            source_step_id,
            source_acquisition_id,
            source_qualified_contract_sha256,
            population,
            population_quantity,
            source_raw_items_pointer,
            source_raw_complete_pointer,
            source_items_pointer,
            source_complete_pointer,
            source_identity_pointer,
            argument_items_pointer,
            argument_complete_pointer,
            argument_identity_pointer,
            result_count_pointer,
            result_distinct_pointer,
            result_complete_pointer,
            result_identity_pointer,
            item_id_pointer,
            identity_scope_key,
            echoes,
        } = self;
        let all_pointers = [
            source_raw_items_pointer,
            source_raw_complete_pointer,
            source_items_pointer,
            source_complete_pointer,
            source_identity_pointer,
            argument_items_pointer,
            argument_complete_pointer,
            argument_identity_pointer,
            result_count_pointer,
            result_distinct_pointer,
            result_complete_pointer,
            result_identity_pointer,
            item_id_pointer,
        ];
        if !identifier(source_step_id)
            || !identifier(source_acquisition_id)
            || !identifier(identity_scope_key)
            || !super::valid_digest(source_qualified_contract_sha256)
            || population.trim().is_empty()
            || population.len() > 4096
            || population_quantity.trim().is_empty()
            || population_quantity.len() > 1024
            || all_pointers
                .iter()
                .any(|p| p.is_empty() || !valid_pointer(p))
            || echoes.len() > 32
            || echoes.iter().any(|(a, b)| {
                a.is_empty()
                    || b.is_empty()
                    || !valid_pointer(a)
                    || !valid_pointer(b)
                    || step.arguments.pointer(a).is_none()
            })
            || step.for_each.is_some()
        {
            return Err("identifier audit coverage proof has invalid bounded pointers, identifiers or fan-out".into());
        }
        let destinations = [
            argument_items_pointer,
            argument_complete_pointer,
            argument_identity_pointer,
        ];
        let source_pointers = [
            source_items_pointer,
            source_complete_pointer,
            source_identity_pointer,
        ];
        if destinations.iter().collect::<BTreeSet<_>>().len() != 3
            || source_pointers.iter().collect::<BTreeSet<_>>().len() != 3
            || step
                .bindings
                .iter()
                .filter(|binding| binding.role.is_selection())
                .count()
                != 3
            || destinations
                .iter()
                .zip(source_pointers)
                .any(|(destination, source)| {
                    step.bindings
                        .iter()
                        .filter(|binding| {
                            binding.argument_pointer.as_str() == destination.as_str()
                                && binding.output.step_id.as_str() == source_step_id.as_str()
                                && binding.output.pointer.as_str() == source.as_str()
                                && binding.role.is_selection()
                        })
                        .count()
                        != 1
                })
        {
            return Err(
                "identifier audit proof requires three exact selection bindings from one source"
                    .into(),
            );
        }
        if source_step.id != *source_acquisition_id
            || source_step.source.qualified_contract_sha256 != *source_qualified_contract_sha256
        {
            return Err(
                "identifier audit proof source acquisition or qualification identity differs"
                    .into(),
            );
        }
        let source = source_step
            .source
            .scope
            .as_object()
            .ok_or("identifier audit source scope requires an object")?;
        let target = step
            .source
            .scope
            .as_object()
            .ok_or("identifier audit target scope requires an object")?;
        if source != target || source.get(identity_scope_key).is_none_or(Value::is_null) {
            return Err(
                "identifier audit source and calculation scopes must agree on their identity"
                    .into(),
            );
        }
        Ok(())
    }

    /// Bound whole-step coverage to independently checked result fields.
    pub(super) fn validate_consumers(
        &self,
        step: &AcquisitionStep,
        plan: &EvidenceExecutionPlan,
    ) -> Result<(), String> {
        let Self::IdentifierAuditV1 {
            result_count_pointer,
            result_distinct_pointer,
            result_complete_pointer,
            result_identity_pointer,
            ..
        } = self;
        let verified = [
            result_count_pointer.as_str(),
            result_distinct_pointer.as_str(),
            result_complete_pointer.as_str(),
            result_identity_pointer.as_str(),
        ];
        let mut covered = BTreeMap::<String, BTreeSet<String>>::new();
        covered.insert(
            step.id.clone(),
            verified
                .iter()
                .map(|pointer| (*pointer).to_owned())
                .collect(),
        );
        let mut projection_count = 0usize;
        for derivation in &plan.derivations {
            if derivation
                .inputs
                .iter()
                .any(|input| input.step_id == step.id)
            {
                if derivation.inputs.len() != 1 || !derivation.inputs[0].pointer.is_empty() {
                    return Err(
                        "audit coverage supports only a whole-result direct projection".into(),
                    );
                }
                let DerivationOperation::ProjectValue { fields } = &derivation.operation else {
                    return Err("audit coverage supports only direct named projections".into());
                };
                projection_count += 1;
                let pointers = fields
                    .iter()
                    .filter(|(_, pointer)| verified.contains(&pointer.as_str()))
                    .map(|(name, _)| format!("/{}", name.replace('~', "~0").replace('/', "~1")))
                    .collect();
                covered.insert(derivation.id.clone(), pointers);
            }
        }
        if projection_count > 1 {
            return Err("audit coverage permits at most one direct projection".into());
        }
        if plan.derivations.iter().any(|derivation| {
            derivation
                .inputs
                .iter()
                .any(|input| input.step_id != step.id && covered.contains_key(&input.step_id))
        }) || plan.acquisitions.iter().any(|acquisition| {
            acquisition
                .bindings
                .iter()
                .any(|binding| covered.contains_key(&binding.output.step_id))
                || acquisition.for_each.as_ref().is_some_and(|binding| {
                    covered.contains_key(&binding.output.step_id)
                        || binding
                            .stop_on_witness
                            .as_ref()
                            .and_then(|predicate| predicate.equals_output.as_ref())
                            .is_some_and(|reference| covered.contains_key(&reference.step_id))
                })
        }) {
            return Err(
                "audit coverage cannot flow through a further derivation or acquisition".into(),
            );
        }
        for evaluation in &plan.evaluations {
            // These criteria are permanently pending without a scoped review;
            // they cannot turn whole-result coverage into accepted evidence.
            if matches!(
                &evaluation.criterion,
                EvaluationCriterion::Unavailable { .. }
                    | EvaluationCriterion::Judgement { scope: None, .. }
            ) {
                continue;
            }
            if !matches!(evaluation.required_coverage, RequiredCoverage::Unspecified) {
                for input in &evaluation.inputs {
                    if covered
                        .get(&input.step_id)
                        .is_some_and(|pointers| !pointers.contains(&input.pointer))
                    {
                        return Err(
                            "audit coverage cannot support an unverified output pointer".into()
                        );
                    }
                }
            }
        }
        Ok(())
    }
}
