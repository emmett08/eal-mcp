use super::{RegisteredAssessmentHost, Selection};
use chrono::{DateTime, Utc};
use eal_core::evaluation::{assess_environment, assess_evidence_record};
use serde_json::Value;
use std::collections::BTreeMap;

impl RegisteredAssessmentHost {
    pub(super) fn candidate(
        &self,
        selected: &Selection,
        name: &str,
        instant: DateTime<Utc>,
        binding: Option<&str>,
        execution: Option<&str>,
    ) -> Result<Option<Value>, String> {
        let Some(binding) = binding else {
            return Ok(None);
        };
        let environment =
            &selected.program.environments[&selected.program.evidence[name].environment];
        if assess_environment(environment, &selected.context)?["status"] != "matched" {
            return Ok(None);
        }
        for original in self.rebinder.candidates(
            &selected.program,
            &selected.context,
            name,
            binding,
            execution,
            100,
        )? {
            let Ok(derived) = self.rebinder.prepare_record(
                &selected.program,
                &selected.context,
                name,
                &original,
                Some(binding),
                execution,
            ) else {
                continue;
            };
            let expected = Value::String(binding.to_owned());
            let verdict = assess_evidence_record(
                &selected.program,
                name,
                Some(&derived),
                instant,
                &selected.context,
                true,
                Some(&expected),
                true,
            )?;
            if verdict.complete {
                return Ok(Some(derived));
            }
        }
        Ok(None)
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn expired(
        &self,
        selected: &Selection,
        reused: &BTreeMap<String, Value>,
        instant: DateTime<Utc>,
        bindings: &Value,
        executions: &Value,
        current_bindings: &Value,
        current_executions: &Value,
    ) -> Result<Vec<String>, String> {
        let mut expired = Vec::new();
        for name in &selected.names {
            let Some(record) = reused.get(name) else {
                continue;
            };
            if bindings[name] != current_bindings[name]
                || executions[name] != current_executions[name]
                || !assess_evidence_record(
                    &selected.program,
                    name,
                    Some(record),
                    instant,
                    &selected.context,
                    true,
                    current_bindings.get(name),
                    true,
                )?
                .complete
            {
                expired.push(name.clone());
            }
        }
        Ok(expired)
    }
}
