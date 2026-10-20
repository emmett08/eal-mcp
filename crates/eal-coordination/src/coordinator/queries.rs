//! Gateway issuance, cached findings and pinned correspondence checks.
use super::*;
use crate::{
    gateway::{CallGate, GatewayOptions, McpGateway},
    ledger::EffectStatus,
};
use std::{collections::BTreeMap, sync::Arc, time::Instant};

impl Coordinator {
    fn gateway(&self, call_id: &str, role: &str, operation: &str) -> Result<McpGateway, String> {
        self.active()?;
        let (_, worker) = self.worker(call_id, Some(role))?;
        if worker["status"] == "committed" {
            return Err("The worker's first response has already been recorded".into());
        }
        self.state.reserve_call(
            role,
            "gateway",
            operation,
            self.budget("max_gateway_calls_per_role")?,
        )?;
        let state = self.state.clone();
        let role = role.to_owned();
        let maximum = self.budget("max_mcp_calls_per_role")?;
        let deadline = number(&self.config, "deadline")?;
        let gate: CallGate = Arc::new(move |name, _, _| {
            if state.cancelled()? {
                return Err("Run was cancelled".into());
            }
            if unix_seconds()? >= deadline {
                return Err("Run wall-clock budget exhausted".into());
            }
            state.reserve_call(&role, "mcp", name, maximum)
        });
        let environment = std::env::vars_os()
            .map(|(key, value)| {
                Ok((
                    key.into_string()
                        .map_err(|_| "Environment names must be UTF-8")?,
                    value
                        .into_string()
                        .map_err(|_| "Environment values must be UTF-8")?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        McpGateway::new(
            self.server.clone(),
            self.anchor.clone(),
            GatewayOptions {
                host_paths: self.host.clone(),
                environment,
                operation_timeout: Duration::from_secs(60),
            },
            Some(gate),
        )
    }
    fn operation_deadline(&self) -> Result<Instant, String> {
        let now = Instant::now();
        now.checked_add(self.wall_remaining()?)
            .ok_or("Run deadline is not representable".into())
    }
    fn unknown(&self, calls: &[Value]) -> Result<Value, String> {
        let mut finding = crate::contracts::make_finding(
            &self.anchor,
            &json!({"ok":false,"code":"effect_outcome_unknown","detail":"Assessment effect has no committed complete receipt; automatic reexecution is disabled"}),
            None,
            None,
            None,
            Some(&json!(calls)),
        )?;
        finding["collected_count"] = Value::Null;
        finding["reused_count"] = Value::Null;
        let object = finding.as_object_mut().ok_or("Invalid finding object")?;
        object.remove("finding_id");
        let id = format!("finding_{}", digest(&finding)?);
        finding["finding_id"] = json!(id);
        Ok(finding)
    }
    pub fn query(&self, call_id: &str) -> Result<Value, String> {
        let mut gateway = self.gateway(call_id, "assessor", "query")?;
        let reservation = self.state.reserve_effect(
            "assessment",
            &json!({"task_digest":self.anchor["task_digest"]}),
        )?;
        let finding = match reservation.status {
            EffectStatus::Committed => {
                let current =
                    crate::profiles::check_task_with_host(&self.server, &self.anchor, &self.host)?;
                let result = reservation
                    .result
                    .ok_or("Committed assessment has no receipt")?;
                if current == result["check"] {
                    result
                } else {
                    crate::contracts::make_finding(&self.anchor, &current, None, None, None, None)?
                }
            }
            EffectStatus::Unknown => self.unknown(&[])?,
            EffectStatus::New => match gateway.assess(self.operation_deadline()?) {
                Ok(finding) => {
                    self.state.complete_effect("assessment", &finding, &[])?;
                    finding
                }
                Err(_) => {
                    self.state.event(
                        "assessment_interrupted",
                        &json!({"error_type":"NativeError","calls":gateway.calls()}),
                    )?;
                    self.unknown(gateway.calls())?
                }
            },
        };
        self.state.issue(&finding)?;
        self.state.event(
            "query",
            &json!({"call_id":call_id,"finding_id":finding["finding_id"]}),
        )?;
        Ok(finding)
    }
    pub(super) fn issued(&self, identity: &str) -> Result<Value, String> {
        let finding = self.state.finding(identity)?;
        let mut content = finding.clone();
        content
            .as_object_mut()
            .ok_or("Invalid issued finding")?
            .remove("finding_id");
        if finding["finding_id"] != format!("finding_{}", digest(&content)?) {
            return Err("Issued finding content identity differs".into());
        }
        if finding["task_digest"] != self.anchor["task_digest"] {
            return Err("Finding belongs to another task".into());
        }
        let current =
            crate::profiles::check_task_with_host(&self.server, &self.anchor, &self.host)?;
        if finding["check"]["code"] != "effect_outcome_unknown" && current != finding["check"] {
            return Err("Task correspondence changed since finding issuance".into());
        }
        Ok(finding)
    }
    pub fn lookup(&self, call_id: &str, finding_id: &str) -> Result<Value, String> {
        let mut gateway = self.gateway(call_id, "reviewer", "lookup")?;
        let finding = self.issued(finding_id)?;
        let finding = if finding["check"]["code"] != "effect_outcome_unknown" {
            gateway.lookup(&finding, self.operation_deadline()?)?
        } else {
            finding
        };
        self.state.event(
            "lookup",
            &json!({"call_id":call_id,"finding_id":finding_id,"calls":gateway.calls()}),
        )?;
        Ok(finding)
    }
}
