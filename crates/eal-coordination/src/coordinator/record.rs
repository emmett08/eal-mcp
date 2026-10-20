//! Retain the first exact worker bytes and admission outcome before publishing anything.
use super::*;
use crate::workers::{CommandWorker, WorkerError};
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

impl Coordinator {
    pub fn record(
        &self,
        call_id: &str,
        raw_response: &[u8],
        execution_id: &str,
        execution_mode: &str,
        response: Option<&Value>,
        evidence: Option<&Value>,
    ) -> Result<Value, String> {
        let _guard = self.lifecycle.hold(Duration::from_secs(30))?;
        let (role, worker) = self.worker(call_id, None)?;
        if execution_id
            .trim_matches(|c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c))
            .is_empty()
            || execution_id.chars().count() > 512
        {
            return Err(
                "Exact response bytes and a bounded actual execution identity are required".into(),
            );
        }
        if raw_response.len() > MAX_RAW_RESPONSE_BYTES {
            return Err("Worker raw response exceeds the native 16 MiB retention limit".into());
        }
        if worker["payload"]["execution_mode"] != execution_mode {
            return Err("Execution mode differs from the prelaunch reservation".into());
        }
        let digest = format!("{:x}", Sha256::digest(raw_response));
        if worker["status"] == "committed" {
            let prior = &worker["result"];
            if prior["raw_digest"] != digest || prior["execution_id"] != execution_id {
                return Err("First response cannot be replaced or repaired".into());
            }
            self.resume_inner()?;
            return Ok(prior.clone());
        }
        let mut result = json!({"execution_id":execution_id,"execution_mode":execution_mode,"raw_base64":STANDARD.encode(raw_response),"raw_digest":digest,"evidence":{},"accepted":false,"outcome":"rejected","diagnostics":[],"response":null});
        let admitted = (|| -> Result<Vec<Value>, String> {
            result["evidence"] = evidence
                .map(|value| crate::contracts::json_object(value, "execution evidence", 65536))
                .transpose()?
                .unwrap_or_else(|| json!({}));
            self.active()?;
            if raw_response.len() > self.budget("max_worker_output_bytes")? {
                return Err("Worker response exceeds output budget".into());
            }
            let text = std::str::from_utf8(raw_response).map_err(|e| e.to_string())?;
            let decoded = eal_core::json::parse(
                text,
                &eal_core::ExecutionLimits {
                    record_bytes: MAX_RAW_RESPONSE_BYTES,
                    json_depth: 128,
                    ..Default::default()
                },
            )?;
            let object = decoded
                .as_object()
                .ok_or("Worker response must be one JSON object")?;
            let wrapped = object.len() == 2
                && object.contains_key("response")
                && object.contains_key("execution");
            let parsed = if wrapped {
                if execution_mode == "software_test" {
                    return Err(
                        "Software mode requires a direct response, without a live receipt".into(),
                    );
                }
                let (parsed, receipt) = crate::workers::live_response(&decoded, call_id)?;
                if receipt["execution_id"] != execution_id {
                    return Err(
                        "External execution receipt differs from the recorded identity".into(),
                    );
                }
                parsed
            } else {
                decoded
            };
            if let Some(response) = response
                && crate::ledger::canonical_json(response)?
                    != crate::ledger::canonical_json(&parsed)?
            {
                return Err("Parsed worker response differs from its exact retained bytes".into());
            }
            if !parsed.is_object() {
                return Err("Worker frame must be an object".into());
            }
            if execution_mode == "provider" && !wrapped {
                return Err("Provider execution needs its matching external receipt".into());
            }
            for prior in self.state.effects()? {
                if prior["key"]
                    .as_str()
                    .is_some_and(|key| key.starts_with("worker:"))
                    && prior["status"] == "committed"
                    && prior["result"]["execution_id"] == execution_id
                {
                    return Err(
                        "Separate worker roles require distinct execution identities".into(),
                    );
                }
            }
            result["response"] = parsed;
            self.admit(
                &role,
                call_id,
                &worker["payload"]["context"],
                &result["response"],
            )
        })();
        let outbox = match admitted {
            Ok(outbox) => {
                result["accepted"] = json!(true);
                result["outcome"] = json!("accepted");
                outbox
            }
            Err(error) => {
                result["diagnostics"] = json!([{"code":"response_rejected","detail":error}]);
                vec![]
            }
        };
        self.state
            .complete_effect(&format!("worker:{role}"), &result, &outbox)?;
        self.state.event(
            "worker_recorded",
            &json!({"call_id":call_id,"execution_id":execution_id,"outcome":result["outcome"]}),
        )?;
        self.resume_inner()?;
        Ok(result)
    }
    pub fn run(&self, worker: &CommandWorker) -> Result<Value, String> {
        if worker.max_output_bytes() > self.budget("max_worker_output_bytes")? {
            return Err("Worker output limit exceeds the run's frozen budget".into());
        }
        self.resume()?;
        for role in ["assessor", "reviewer"] {
            if let Some(existing) = self.state.effect(&format!("worker:{role}"))? {
                if existing["status"] != "committed" || existing["result"]["accepted"] != true {
                    return self.status();
                }
                continue;
            }
            let reservation = self.reserve_worker(role, worker.execution_mode())?;
            if reservation["new"] != true {
                return self.status();
            }
            let call_id = string(&reservation, "call_id")?;
            let remaining = self
                .wall_remaining()?
                .min(Duration::from_secs(self.budget("worker_seconds")? as u64));
            if remaining.is_zero() {
                return self.status();
            }
            let stop = || self.state.cancelled();
            match worker.execute(&reservation["context"], call_id, remaining, Some(&stop)) {
                Ok(output) => {
                    self.record(
                        call_id,
                        &output.raw_response,
                        &output.execution_id,
                        &output.execution_mode,
                        Some(&output.response),
                        Some(&output.evidence),
                    )?;
                }
                Err(WorkerError::Invalid(error)) => return Err(error),
                Err(WorkerError::Interrupted(error)) => {
                    let result = json!({"execution_id":error.execution_id,"execution_mode":error.execution_mode,"raw_base64":STANDARD.encode(&error.raw_response),"raw_digest":format!("{:x}",Sha256::digest(&error.raw_response)),"evidence":error.evidence,"accepted":false,"outcome":"interrupted","diagnostics":[{"code":error.reason}],"response":null});
                    self.state
                        .complete_effect(&format!("worker:{role}"), &result, &[])?;
                    self.state.event(
                        "worker_interrupted",
                        &json!({"call_id":call_id,"reason":error.reason}),
                    )?;
                    self.resume()?;
                    return self.status();
                }
            }
            if self
                .state
                .effect(&format!("worker:{role}"))?
                .ok_or("Worker effect disappeared")?["result"]["accepted"]
                != true
            {
                return self.status();
            }
        }
        self.status()
    }
}
