//! Validate a specific finite identifier audit against a qualified source observation.
use crate::{
    execution::{SourceRecord, StepResult, StepStatus},
    plan::{AcquisitionStep, CoverageProof},
};
use eal_collector_contracts::capability::{
    CoverageGuarantee, IdentifierAuditSemantics, LogicalStrength, MeasurementQuantity,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

pub(crate) struct AuditInputs {
    count: u64,
    distinct: bool,
    identity: Value,
    count_pointer: String,
    distinct_pointer: String,
    complete_pointer: String,
    identity_pointer: String,
    echoes: Vec<(String, Value)>,
    mappings: IdentifierAuditSemantics,
    minimum_observed_id_chars: Option<usize>,
    maximum_observed_id_chars: usize,
}

pub(crate) fn prepare(
    proof: &CoverageProof,
    step: &AcquisitionStep,
    arguments: &Value,
    steps: &BTreeMap<String, StepResult>,
    observations: &BTreeMap<String, SourceRecord>,
    deadline: Instant,
) -> Result<AuditInputs, String> {
    if Instant::now() >= deadline {
        return Err("audit proof deadline exhausted".into());
    }
    let CoverageProof::IdentifierAuditV1 {
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
    } = proof;
    let parent = steps
        .get(source_step_id)
        .ok_or("audit proof source is unavailable")?;
    let physical = steps
        .get(source_acquisition_id)
        .ok_or("audit proof physical source is unavailable")?;
    if !matches!(parent.status, StepStatus::Measured | StepStatus::Derived)
        || !matches!(physical.status, StepStatus::Measured)
        || !parent.coverage.is_complete()
        || parent.truncated
        || !physical.coverage.is_complete()
        || physical.truncated
        || parent.observation_ids.len() != 1
        || parent.source_ids.len() != 1
        || physical.observation_ids != parent.observation_ids
        || physical.source_ids != parent.source_ids
        || parent.observed_at.is_none()
    {
        return Err("audit proof requires one complete untruncated observed source".into());
    }
    let record = observations
        .get(&parent.observation_ids[0])
        .ok_or("audit proof source observation is absent")?;
    if record.source_id != parent.source_ids[0]
        || record.source.qualified_contract_sha256 != *source_qualified_contract_sha256
        || record
            .provenance
            .get("qualified_contract_sha256")
            .and_then(Value::as_str)
            != Some(source_qualified_contract_sha256.as_str())
        || record.source.scope != step.source.scope
        || !record.coverage.is_complete()
    {
        return Err("audit proof source identity, qualification, scope or coverage differs".into());
    }
    let quantities: Vec<MeasurementQuantity> = serde_json::from_value(
        record
            .provenance
            .get("qualified_quantities")
            .cloned()
            .ok_or("source quantity qualification is absent")?,
    )
    .map_err(|_| "source quantity qualification is invalid")?;
    let coverage: Vec<CoverageGuarantee> = serde_json::from_value(
        record
            .provenance
            .get("qualified_coverage")
            .cloned()
            .ok_or("source coverage qualification is absent")?,
    )
    .map_err(|_| "source coverage qualification is invalid")?;
    if !quantities.iter().any(|quantity| {
        quantity.name == *population_quantity
            && quantity.output_pointer == *source_raw_items_pointer
    }) || !coverage.iter().any(|guarantee| {
        guarantee.strength == LogicalStrength::CompleteEnumeration
            && guarantee.population.as_ref() == Some(population)
            && guarantee.population_quantity.as_ref() == Some(population_quantity)
            && guarantee.completeness_pointer.as_ref() == Some(source_raw_complete_pointer)
    }) {
        return Err(
            "source qualification does not bind the whole population to the selected quantity"
                .into(),
        );
    }
    let parent_value = parent
        .value
        .as_ref()
        .ok_or("audit source value is absent")?;
    let selected = parent_value
        .pointer(source_items_pointer)
        .ok_or("audit source items are absent")?;
    if record.value.pointer(source_raw_items_pointer) != Some(selected)
        || record.value.pointer(source_raw_complete_pointer) != Some(&Value::Bool(true))
        || parent_value.pointer(source_complete_pointer) != Some(&Value::Bool(true))
        || arguments.pointer(argument_complete_pointer) != Some(&Value::Bool(true))
        || arguments.pointer(argument_items_pointer) != Some(selected)
    {
        return Err("audit input differs from the complete qualified source population".into());
    }
    let identity = parent_value
        .pointer(source_identity_pointer)
        .filter(|value| !value.is_null())
        .ok_or("audit source identity is absent")?;
    if arguments.pointer(argument_identity_pointer) != Some(identity)
        || step.source.scope.get(identity_scope_key) != Some(identity)
    {
        return Err("audit input identity differs from the qualified source scope".into());
    }
    let entries = selected
        .as_array()
        .ok_or("audit source quantity is not an array")?;
    if entries.len() > 100_000 {
        return Err("audit source array exceeds the proof work bound".into());
    }
    let mut identifiers = BTreeSet::new();
    let mut distinct = true;
    let mut minimum_observed_id_chars: Option<usize> = None;
    let mut maximum_observed_id_chars = 0;
    for entry in entries {
        if Instant::now() >= deadline {
            return Err("audit proof deadline exhausted".into());
        }
        let id = entry
            .pointer(item_id_pointer)
            .and_then(Value::as_str)
            .ok_or("audit identifier is not a string")?;
        let length = id.chars().count();
        if length > 4096 {
            return Err("audit identifier exceeds the proof work bound".into());
        }
        minimum_observed_id_chars =
            Some(minimum_observed_id_chars.map_or(length, |minimum| minimum.min(length)));
        maximum_observed_id_chars = maximum_observed_id_chars.max(length);
        distinct &= identifiers.insert(id);
    }
    let echo_values = echoes
        .iter()
        .map(|(input, output)| {
            Ok((
                output.clone(),
                arguments
                    .pointer(input)
                    .ok_or("audit echo argument is absent")?
                    .clone(),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(AuditInputs {
        count: u64::try_from(entries.len())
            .map_err(|_| "audit array length is not representable")?,
        distinct,
        identity: identity.clone(),
        count_pointer: result_count_pointer.clone(),
        distinct_pointer: result_distinct_pointer.clone(),
        complete_pointer: result_complete_pointer.clone(),
        identity_pointer: result_identity_pointer.clone(),
        echoes: echo_values,
        mappings: IdentifierAuditSemantics {
            argument_items_pointer: argument_items_pointer.clone(),
            argument_complete_pointer: argument_complete_pointer.clone(),
            argument_identity_pointer: argument_identity_pointer.clone(),
            item_id_pointer: item_id_pointer.clone(),
            result_count_pointer: result_count_pointer.clone(),
            result_distinct_pointer: result_distinct_pointer.clone(),
            result_complete_pointer: result_complete_pointer.clone(),
            result_identity_pointer: result_identity_pointer.clone(),
            maximum_items: 0,
            minimum_id_chars: 0,
            maximum_id_chars: 0,
            echoes: echoes.clone(),
        },
        minimum_observed_id_chars,
        maximum_observed_id_chars,
    })
}

impl AuditInputs {
    pub(crate) fn verify(
        &self,
        record: &SourceRecord,
        step: &AcquisitionStep,
    ) -> Result<(), String> {
        let mut qualified_audit: IdentifierAuditSemantics = serde_json::from_value(
            record
                .provenance
                .get("qualified_identifier_audit")
                .cloned()
                .ok_or("qualified identifier audit semantics are absent")?,
        )
        .map_err(|_| "qualified identifier audit semantics are invalid")?;
        if !(1..=100_000).contains(&qualified_audit.maximum_items)
            || qualified_audit.minimum_id_chars > qualified_audit.maximum_id_chars
            || qualified_audit.maximum_id_chars > 4096
            || qualified_audit.maximum_items < self.count as usize
            || self
                .minimum_observed_id_chars
                .is_some_and(|length| length < qualified_audit.minimum_id_chars)
            || self.maximum_observed_id_chars > qualified_audit.maximum_id_chars
        {
            return Err("audit input exceeds independently qualified operation limits".into());
        }
        qualified_audit.maximum_items = 0;
        qualified_audit.minimum_id_chars = 0;
        qualified_audit.maximum_id_chars = 0;
        let qualified = record
            .provenance
            .get("qualified_logical_strengths")
            .and_then(Value::as_array)
            .is_some_and(|values| {
                values
                    .iter()
                    .any(|value| value.as_str() == Some("exact_calculation"))
            });
        if !qualified
            || qualified_audit != self.mappings
            || record
                .provenance
                .get("qualified_contract_sha256")
                .and_then(Value::as_str)
                != Some(step.source.qualified_contract_sha256.as_str())
            || record.source.qualified_contract_sha256 != step.source.qualified_contract_sha256
            || record
                .value
                .pointer(&self.count_pointer)
                .and_then(Value::as_u64)
                != Some(self.count)
            || record
                .value
                .pointer(&self.distinct_pointer)
                .and_then(Value::as_bool)
                != Some(self.distinct)
            || record.value.pointer(&self.complete_pointer) != Some(&Value::Bool(true))
            || record.value.pointer(&self.identity_pointer) != Some(&self.identity)
            || self
                .echoes
                .iter()
                .any(|(pointer, expected)| record.value.pointer(pointer) != Some(expected))
        {
            return Err(
                "audit output does not match independently recomputed qualified inputs".into(),
            );
        }
        Ok(())
    }
}
