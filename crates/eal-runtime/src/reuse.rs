//! Source-bound reuse preserves original measurement time and acquisition identity.
use crate::{
    collection::acquisition_request,
    store::{ObservationFilter, RunStore, utc_now},
};
use eal_core::{canonical, evaluation, semantic::Program, validation};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Clone)]
pub struct ObservationRebinder {
    store: RunStore,
}

impl ObservationRebinder {
    pub fn new(store: RunStore) -> Self {
        Self { store }
    }

    fn expected(
        program: &Program,
        context: &Value,
        evidence_id: &str,
        binding_digest: Option<&str>,
        execution_digest: Option<&str>,
    ) -> Result<Value, String> {
        let quoted =
            || eal_core::expressions::diagnostic_repr(&json!(evidence_id), &program.limits);
        let evidence = program.evidence.get(evidence_id).ok_or_else(|| {
            quoted()
                .map(|id| format!("Undeclared evidence identifier {id}"))
                .unwrap_or_else(|e| e)
        })?;
        let binding = binding_digest
            .filter(|digest| valid_digest(digest))
            .ok_or_else(|| {
                quoted()
                    .map(|id| format!("No current tool binding for evidence {id}"))
                    .unwrap_or_else(|e| e)
            })?;
        canonical::canonical_digest(context, &program.limits)?;
        let context = evaluation::environment_context(&evidence.environment, context)?;
        let acquisition = acquisition_request(program, evidence_id, &context)?;
        let request =
            crate::collection::full_request(evidence_id, &evidence.environment, &acquisition);
        let mut expected = json!({
            "evidence_id":evidence_id,"tool":acquisition["tool"],"tool_version":acquisition["tool_version"],
            "evidence_kind":evidence.kind,"environment":evidence.environment,
            "environment_fingerprint":evaluation::environment_fingerprint(&evidence.environment,&context,&program.limits)?,
            "input":evidence.input,"input_digest":canonical::canonical_digest(&evidence.input,&program.limits)?,
            "context":context,"acquisition_request":acquisition,
            "acquisition_request_digest":canonical::canonical_digest(&acquisition,&program.limits)?,
            "request_digest":canonical::canonical_digest(&request,&program.limits)?,
            "tool_binding_digest":binding
        });
        if let Some(execution) = execution_digest {
            if !valid_digest(execution) {
                return Err(format!(
                    "Invalid current process environment for evidence {}",
                    quoted()?
                ));
            }
            expected["process_environment_digest"] = json!(execution);
        }
        Ok(expected)
    }

    fn verify(record: &Value, expected: &Value, program: &Program) -> Result<(), String> {
        if record["status"] != "ok" {
            return Err("Only a successful observation can be rebound".into());
        }
        if record["schema"] != "EAL/observation-record/1" {
            return Err("Rebinding requires an EAL/observation-record/1 observation".into());
        }
        if record.get("process_environment_digest").is_some()
            && expected.get("process_environment_digest").is_none()
        {
            return Err(
                "Current process environment identity is required for command reuse".into(),
            );
        }
        for (key, value) in expected
            .as_object()
            .expect("expected acquisition is an object")
        {
            if !record
                .get(key)
                .is_some_and(|actual| canonical::same_json(actual, value, &program.limits))
            {
                return Err(format!(
                    "Observation {key} differs from the requested acquisition"
                ));
            }
        }
        if record["run_id"].as_str().is_none_or(str::is_empty) {
            return Err("Observation requires a run_id".into());
        }
        let collected = record["collected_at"]
            .as_str()
            .ok_or("Timestamp must be an ISO-8601 timestamp with timezone")?;
        validation::parse_time(collected)?;
        if let Some(observed) = record.get("observed_at") {
            validation::parse_time(
                observed
                    .as_str()
                    .ok_or("Timestamp must be an ISO-8601 timestamp with timezone")?,
            )?;
        }
        let value = record
            .get("value")
            .ok_or("Observation data digest differs from its value")?;
        if record["data_digest"].as_str()
            != Some(canonical::canonical_digest(value, &program.limits)?.as_str())
        {
            return Err("Observation data digest differs from its value".into());
        }
        Ok(())
    }

    /// Acquisition-compatible candidates, newest first. Freshness and current
    /// predicates belong to the later explicit-time authored assessment.
    pub fn candidates(
        &self,
        program: &Program,
        context: &Value,
        evidence_id: &str,
        current_binding_digest: &str,
        current_execution_digest: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Value>, String> {
        let expected = Self::expected(
            program,
            context,
            evidence_id,
            Some(current_binding_digest),
            current_execution_digest,
        )?;
        let filter = ObservationFilter {
            evidence_id,
            environment: expected["environment"]
                .as_str()
                .expect("declared environment"),
            request_digest: expected["request_digest"]
                .as_str()
                .expect("acquisition digest"),
            tool_binding_digest: current_binding_digest,
            limit,
        };
        Ok(self
            .store
            .find_observations(&filter)?
            .into_iter()
            .filter(|record| Self::verify(record, &expected, program).is_ok())
            .collect())
    }

    /// Prepare without writing. The exact stored original is checked before a
    /// new source-bound record is created; callers can commit an atomic batch.
    pub fn prepare_record(
        &self,
        program: &Program,
        context: &Value,
        evidence_id: &str,
        original: &Value,
        current_binding_digest: Option<&str>,
        current_execution_digest: Option<&str>,
    ) -> Result<Value, String> {
        let expected = Self::expected(
            program,
            context,
            evidence_id,
            current_binding_digest,
            current_execution_digest,
        )?;
        let quoted = eal_core::expressions::diagnostic_repr(&json!(evidence_id), &program.limits)?;
        let run_id = original
            .as_object()
            .and_then(|o| o.get("run_id"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("Observation {quoted} has no run_id"))?;
        let stored = self.store.get(run_id, Some("observation"))?;
        // Whole immutable records include request/provenance metadata as well
        // as independently admitted values. Compare them under the store's
        // persistence envelope, not one payload's JSON-node allowance.
        let stored_limits = eal_core::ExecutionLimits {
            record_bytes: crate::store::MAX_STORED_RECORD_BYTES,
            json_nodes: crate::store::MAX_STORED_RECORD_BYTES,
            json_depth: eal_core::json::MAX_JSON_DEPTH,
            ..eal_core::ExecutionLimits::default()
        };
        if !canonical::same_json(&stored, original, &stored_limits) {
            return Err(format!(
                "Observation {quoted} differs from stored acquisition"
            ));
        }
        Self::verify(original, &expected, program)?;
        let mut copy = stored.clone();
        let object = copy
            .as_object_mut()
            .ok_or("Stored observation must be a JSON object")?;
        object.remove("stdout");
        object.remove("stderr");
        object.insert("run_id".into(), json!(Uuid::new_v4().to_string()));
        object.insert("source_digest".into(), json!(program.source_digest));
        object.insert(
            "origin_run_id".into(),
            stored
                .get("origin_run_id")
                .unwrap_or(&stored["run_id"])
                .clone(),
        );
        object.insert("reused_from_run_id".into(), stored["run_id"].clone());
        object.insert("rebound_at".into(), json!(utc_now()));
        Ok(copy)
    }
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
