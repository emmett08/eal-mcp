//! Reuse dependent acquisitions as one immutable closure, never by mixing parents.
use super::{RegisteredAssessmentHost, Selection};
use chrono::{DateTime, Utc};
use eal_core::evaluation;

impl RegisteredAssessmentHost {
    pub(super) fn reuse_dependency_collection(
        &self,
        selected: &Selection,
        instant: DateTime<Utc>,
    ) -> Result<Option<String>, String> {
        let bindings = self.service.current_binding_digests(&selected.program)?;
        let executions = self.service.current_execution_digests(&selected.program)?;
        for entry in self.service.store().list(Some("collection"), 100)? {
            let collection = self.service.store().get(&entry.id, Some("collection"))?;
            let Ok(records) = crate::identity::CollectionIdentityValidator.validate(
                &collection,
                &selected.program.source_digest,
                &selected.context,
                &selected.program.limits,
            ) else {
                continue;
            };
            if selected
                .names
                .iter()
                .any(|name| records.get(name).is_none())
            {
                continue;
            }
            if selected.names.iter().any(|name| {
                records[name].get("process_environment_digest")
                    != executions.get(name).filter(|value| !value.is_null())
            }) {
                continue;
            }
            let verdicts = evaluation::assess_evidence_records(
                &selected.program,
                &selected.names,
                &records,
                instant,
                &selected.context,
                Some(&bindings),
            )?;
            if selected.names.iter().all(|name| {
                verdicts
                    .get(name)
                    .is_some_and(|verdict| verdict.data_available)
            }) {
                return Ok(Some(entry.id));
            }
        }
        Ok(None)
    }
}
