//! One registered claim: complete dependency planning, compatible reuse and durable assessment.
use crate::{
    catalogue::WorkspaceKnowledgeCatalogue, lease::AcquisitionLease, packets::PacketLimits,
    reuse::ObservationRebinder, service::ReasoningService, store::utc_now,
};
use eal_core::{canonical, semantic::Program, validation};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};

mod persistence;
mod reuse;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReusePolicy {
    #[default]
    Compatible,
    Fresh,
}
impl TryFrom<&str> for ReusePolicy {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, String> {
        match value {
            "compatible" => Ok(Self::Compatible),
            "fresh" => Ok(Self::Fresh),
            _ => Err("reuse must be 'compatible' or 'fresh'".into()),
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct RegisteredAssessmentOptions<'a> {
    pub context: Option<&'a Value>,
    pub now: Option<&'a str>,
    pub reuse: ReusePolicy,
    /// Presentation limits only. Omission uses the existing packet contract.
    pub packet_limits: Option<&'a PacketLimits>,
}

#[derive(Clone)]
pub struct RegisteredAssessmentHost {
    pub service: Arc<ReasoningService>,
    pub catalogue: Arc<WorkspaceKnowledgeCatalogue>,
    rebinder: ObservationRebinder,
    clock: Arc<dyn Fn() -> String + Send + Sync>,
}
struct Selection {
    entry: Value,
    source: String,
    program: Program,
    context: Value,
    names: Vec<String>,
}
impl RegisteredAssessmentHost {
    pub fn new(
        service: Arc<ReasoningService>,
        catalogue: Arc<WorkspaceKnowledgeCatalogue>,
    ) -> Result<Self, String> {
        Self::with_clock(service, catalogue, Arc::new(utc_now))
    }
    /// An operator-owned clock permits deterministic expiry checks without changing observation times.
    pub fn with_clock(
        service: Arc<ReasoningService>,
        catalogue: Arc<WorkspaceKnowledgeCatalogue>,
        clock: Arc<dyn Fn() -> String + Send + Sync>,
    ) -> Result<Self, String> {
        if !Arc::ptr_eq(catalogue.service(), &service) {
            return Err("Catalogue and assessment host must use the same service".into());
        }
        let rebinder = ObservationRebinder::new(service.store().clone());
        Ok(Self {
            service,
            catalogue,
            rebinder,
            clock,
        })
    }
    pub fn assess(
        &self,
        entry_id: &str,
        claim: &str,
        options: RegisteredAssessmentOptions<'_>,
    ) -> Result<Value, String> {
        if let Some(limits) = options.packet_limits {
            limits.validate()?;
        }
        self.assess_with_lease(entry_id, claim, options, self.service.acquisition_lease()?)
    }
    /// An admitted caller retains the same descriptor through every acquisition, recheck and commit.
    pub fn assess_with_lease(
        &self,
        entry_id: &str,
        claim: &str,
        options: RegisteredAssessmentOptions<'_>,
        lease: Arc<AcquisitionLease>,
    ) -> Result<Value, String> {
        let packet_limits = options.packet_limits.cloned().unwrap_or_default();
        packet_limits.validate()?;
        let now = options
            .now
            .map(str::to_owned)
            .unwrap_or_else(|| (self.clock)());
        let mut instant = validation::parse_time(&now)?;
        let selected = self.select(entry_id, claim, options.context)?;
        let mut reused = BTreeMap::new();
        let mut bindings = json!({});
        let mut executions = json!({});
        if options.reuse == ReusePolicy::Compatible {
            bindings = self.service.current_binding_digests(&selected.program)?;
            executions = self.service.current_execution_digests(&selected.program)?;
            for name in &selected.names {
                if let Some(record) = self.candidate(
                    &selected,
                    name,
                    instant,
                    bindings[name].as_str(),
                    executions[name].as_str(),
                )? {
                    reused.insert(name.clone(), record);
                }
            }
        }
        let missing = selected
            .names
            .iter()
            .filter(|name| !reused.contains_key(*name))
            .cloned()
            .collect::<Vec<_>>();
        let collection = if !missing.is_empty() || reused.is_empty() {
            Some(self.service.runtime()?.collect_with_lease(
                &selected.program,
                &selected.context,
                Some(&missing),
                Some(Arc::clone(&lease)),
            )?)
        } else {
            None
        };
        let mut fresh: BTreeMap<String, Value> = collection
            .as_ref()
            .and_then(|v| v["records"].as_object())
            .map(|v| v.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        if !reused.is_empty() {
            if options.now.is_none() {
                instant = validation::parse_time(&(self.clock)())?;
            }
            let current_bindings = self.service.current_binding_digests(&selected.program)?;
            let current_executions = self.service.current_execution_digests(&selected.program)?;
            let expired = self.expired(
                &selected,
                &reused,
                instant,
                &bindings,
                &executions,
                &current_bindings,
                &current_executions,
            )?;
            if !expired.is_empty() {
                for name in &expired {
                    reused.remove(name);
                }
                let replacement = self.service.runtime()?.collect_with_lease(
                    &selected.program,
                    &selected.context,
                    Some(&expired),
                    Some(Arc::clone(&lease)),
                )?;
                for (name, record) in replacement["records"]
                    .as_object()
                    .ok_or("Invalid replacement collection")?
                {
                    fresh.insert(name.clone(), record.clone());
                }
            }
        }
        // An external source/import edit must not be returned as an assessment of the new revision.
        self.check_source(entry_id, &selected)?;
        let collection_id = if !reused.is_empty() {
            self.persist_mixed(&selected, &reused, &fresh)?
        } else if let Some(collection) = &collection
            && same_names(&collection["records"], &selected.names)
            && same_names_map(&fresh, &selected.names)
        {
            collection["collection_id"]
                .as_str()
                .ok_or("Collection has no durable identity")?
                .to_owned()
        } else {
            self.persist_mixed(&selected, &BTreeMap::new(), &fresh)?
        };
        let assessed_at = options
            .now
            .map(str::to_owned)
            .unwrap_or_else(|| (self.clock)());
        let assessment = self.service.reason(
            &selected.source,
            &selected.context,
            Some(&collection_id),
            Some(&assessed_at),
        )?;
        if assessment["source_digest"] != selected.program.source_digest {
            return Err("Registered source changed during assessment".into());
        }
        self.check_source(entry_id, &selected)?;
        let assessment_id = assessment["assessment_id"]
            .as_str()
            .ok_or("Assessment has no durable identity")?;
        let packet = self
            .service
            .packet_with_limits(assessment_id, Some(claim), packet_limits)?;
        Ok(
            json!({"schema":"EAL/registered-assessment/1","entry_id":entry_id,"claim":claim,
            "source_digest":selected.program.source_digest,"context_fingerprint":assessment["context_fingerprint"],
            "collection_id":collection_id,"assessment_id":assessment_id,"assessed_at":assessment["assessed_at"],
            "status":assessment["claims"][claim]["status"],"reused_count":reused.len(),"collected_count":fresh.len(),
            "packet":packet,"full_explanation":packet["full_explanation"]}),
        )
    }

    /// Read a bounded packet for a selected claim in an exact registered revision.
    /// The caller enforces its entry allowlist. This operation neither refreshes
    /// source files nor collects evidence, and retains the original assessment time.
    pub fn retrieve_packet(
        &self,
        entry_id: &str,
        assessment_id: &str,
        claim: &str,
        limits: PacketLimits,
    ) -> Result<Value, String> {
        limits.validate()?;
        let assessment = self.service.explain(assessment_id, None)?;
        let source_digest = assessment["source_digest"]
            .as_str()
            .ok_or("Stored assessment has no source identity")?;
        let method = assessment["method_registry_fingerprint"]
            .as_str()
            .ok_or("Stored assessment has no method registry identity")?;
        let revision = self
            .catalogue
            .get_revision(entry_id, source_digest, Some(method), false)?;
        if !revision["claims"]
            .as_array()
            .is_some_and(|claims| claims.iter().any(|item| item == claim))
        {
            return Err("Claim is not selected in the registered source revision".into());
        }
        let context_fingerprint =
            canonical::canonical_digest(&revision["context"], self.service.limits())?;
        if assessment["context_fingerprint"] != context_fingerprint {
            return Err("Stored assessment context differs from the registered context".into());
        }
        // The service also checks the exact current method output contract. Old
        // registries require their original allowlists before they can be projected.
        let packet = self
            .service
            .packet_with_limits(assessment_id, Some(claim), limits)?;
        Ok(json!({
            "schema": "EAL/registered-packet/1", "entry_id": entry_id, "claim": claim,
            "assessment_id": assessment_id, "assessed_at": assessment["assessed_at"],
            "status": assessment["claims"][claim]["status"], "source_digest": source_digest,
            "context_fingerprint": context_fingerprint, "method_registry_fingerprint": method,
            "packet": packet
        }))
    }
    pub fn history(
        &self,
        entry_id: &str,
        source_digest: Option<&str>,
        limit: usize,
    ) -> Result<Value, String> {
        self.catalogue.runs(entry_id, source_digest, limit)
    }

    fn select(
        &self,
        entry_id: &str,
        claim: &str,
        context: Option<&Value>,
    ) -> Result<Selection, String> {
        let entry = self.catalogue.get(entry_id, true)?;
        if !entry["claims"]
            .as_array()
            .is_some_and(|claims| claims.iter().any(|v| v == claim))
        {
            let repr = |value: &str| {
                eal_core::expressions::diagnostic_repr(&json!(value), self.service.limits())
            };
            return Err(format!(
                "Claim {} is not selected in registered entry {}",
                repr(claim)?,
                repr(entry_id)?
            ));
        }
        if entry["method_registry_fingerprint"] != self.service.method_registry().fingerprint() {
            return Err("Registered source method registry differs from the current host".into());
        }
        let context = context.unwrap_or(&entry["context"]);
        if !context.is_object() {
            return Err("context must be a JSON object".into());
        }
        canonical::canonical_digest(context, self.service.limits())?;
        let source = entry["source"]
            .as_str()
            .ok_or("Registered entry has no captured source")?
            .to_owned();
        let plan = self.service.plan(&source, claim)?;
        let names = plan["evidence_ids"]
            .as_array()
            .ok_or("Invalid evidence plan")?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or("Invalid evidence identifier")
            })
            .collect::<Result<Vec<_>, _>>()?;
        if names.len() > self.service.limits().collection_evidence {
            return Err(format!(
                "Collection exceeds {} evidence requests",
                self.service.limits().collection_evidence
            ));
        }
        let program = self.service.parse(&source)?;
        if program.source_digest != entry["source_digest"]
            || plan["source_digest"] != entry["source_digest"]
        {
            return Err("Registered source digest differs from its snapshot".into());
        }
        let context = context.clone();
        Ok(Selection {
            entry,
            source,
            program,
            context,
            names,
        })
    }
    fn check_source(&self, entry_id: &str, selected: &Selection) -> Result<(), String> {
        let current = self.catalogue.get(entry_id, true)?;
        if current["source_digest"] != selected.entry["source_digest"]
            || current["method_registry_fingerprint"]
                != selected.entry["method_registry_fingerprint"]
            || current["source"] != selected.source
            || self.service.parse(&selected.source)?.source_digest != selected.program.source_digest
        {
            return Err("Registered source changed during assessment".into());
        }
        Ok(())
    }
}
fn same_names(records: &Value, names: &[String]) -> bool {
    records
        .as_object()
        .is_some_and(|v| v.len() == names.len() && names.iter().all(|n| v.contains_key(n)))
}
fn same_names_map(records: &BTreeMap<String, Value>, names: &[String]) -> bool {
    records.len() == names.len() && names.iter().all(|n| records.contains_key(n))
}
