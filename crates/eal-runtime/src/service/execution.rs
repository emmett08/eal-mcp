//! Collect and assess one compiled program under one acquisition lease.
use super::*;

impl ReasoningService {
    /// Execute the source with the operator's concrete tool bindings. The source
    /// is compiled once, so an import cannot change between collection and use.
    pub fn execute(
        &self,
        source: &str,
        context: &Value,
        claim: Option<&str>,
        now: Option<&str>,
    ) -> Result<Value, String> {
        self.execute_with_lease(source, context, claim, now, self.acquisition_lease()?)
    }

    pub fn execute_with_lease(
        &self,
        source: &str,
        context: &Value,
        claim: Option<&str>,
        now: Option<&str>,
        lease: Arc<crate::lease::AcquisitionLease>,
    ) -> Result<Value, String> {
        if let Some(now) = now {
            validation::parse_time(now)?;
        }
        let program = self.parse(source)?;
        let executable_digest =
            eal_core::semantic_format::executable_digest(&program, &self.methods)
                .map_err(|error| error.to_string())?;
        let selected = claim
            .map(|claim| {
                EvidencePlanner::new_with_registry(&program, &self.methods)?
                    .plan(claim)
                    .map(|plan| plan.evidence_ids())
            })
            .transpose()?;
        let runtime = self.runtime()?;
        let collection = runtime.collect_with_lease(
            &program,
            context,
            selected.as_deref(),
            Some(Arc::clone(&lease)),
        )?;
        let collection_id = collection["collection_id"]
            .as_str()
            .ok_or("Collection did not retain its identity")?;
        let assessment = self.reason_program(&program, context, Some(collection_id), now)?;
        Ok(json!({
            "schema": "EAL/execution/1",
            "source_digest": program.source_digest,
            "executable_digest": executable_digest,
            "claim": claim,
            "collection": collection,
            "assessment": assessment
        }))
    }
}
