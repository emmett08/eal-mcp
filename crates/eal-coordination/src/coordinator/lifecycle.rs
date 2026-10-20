//! Reservation, publication and acknowledgement transitions under one lifecycle lock.
use super::*;
use crate::{
    bus::{MailboxLimits, envelope},
    ledger::{EffectStatus, canonical_json},
};
use std::collections::BTreeMap;

impl Coordinator {
    pub fn resume(&self) -> Result<Value, String> {
        let _guard = self.lifecycle.hold(Duration::from_secs(30))?;
        self.resume_inner()
    }
    fn ensure_bus(&self) -> Result<(), String> {
        if self
            .state
            .events()?
            .iter()
            .any(|event| event["kind"] == "bus_initialized")
        {
            return Ok(());
        }
        self.bus.create(
            string(&self.config, "run_id")?,
            &["assessor".into(), "reviewer".into()],
            MailboxLimits {
                max_messages: self.budget("max_messages")?,
                max_bytes: self.budget("max_bytes")?,
                max_calls: self.budget("max_worker_calls")?,
            },
        )?;
        self.state.event("bus_initialized", &json!({}))
    }
    pub(super) fn resume_inner(&self) -> Result<Value, String> {
        self.ensure_bus()?;
        let initial = self.state.reserve_effect("initial", &self.request())?;
        if initial.status != EffectStatus::Committed {
            self.state.complete_effect(
                "initial",
                &json!({"message":self.request()}),
                &[self.request()],
            )?;
        }
        let run = string(&self.config, "run_id")?;
        let status = self.bus.status(run)?;
        let mut calls = BTreeMap::new();
        for call in array(&status, "calls")? {
            calls.insert(string(call, "id")?.to_owned(), call.clone());
        }
        for role in ["assessor", "reviewer"] {
            if let Some(effect) = self.state.effect(&format!("worker:{role}"))? {
                let call_id = string(&effect["payload"], "call_id")?;
                if let Some(existing) = calls.get(call_id) {
                    if existing["actor"] != role {
                        return Err("Transport reservation belongs to another role".into());
                    }
                } else {
                    self.bus.reserve(run, role, call_id)?;
                    calls.insert(
                        call_id.to_owned(),
                        json!({"id":call_id,"actor":role,"state":"reserved","evidence":null}),
                    );
                }
            }
        }
        self.state.drain_outbox(
            |message| {
                self.bus.send(run, message, string(message, "sender")?)?;
                Ok(())
            },
            |key| {
                if key.starts_with("worker:") {
                    let effect = self
                        .state
                        .effect(key)?
                        .ok_or("Worker effect disappeared during publication")?;
                    for message in array(&effect["payload"]["context"], "received_messages")? {
                        self.bus.acknowledge(
                            run,
                            string(message, "message_id")?,
                            string(&effect["payload"], "role")?,
                        )?;
                    }
                }
                Ok(())
            },
        )?;
        for role in ["assessor", "reviewer"] {
            let Some(effect) = self.state.effect(&format!("worker:{role}"))? else {
                continue;
            };
            if effect["status"] != "committed" {
                continue;
            }
            let result = &effect["result"];
            let call_id = string(&effect["payload"], "call_id")?;
            let evidence = json!({"execution_id":result["execution_id"],"execution_mode":result["execution_mode"],"response_digest":result["raw_digest"],"outcome":result["outcome"],"tokens":null,"cost_usd":null});
            let prior = calls
                .get(call_id)
                .ok_or("Committed worker has no prelaunch transport reservation")?;
            if prior["state"] == "recorded" {
                if prior["evidence"] != evidence {
                    return Err(
                        "Transport completion evidence differs from the committed worker".into(),
                    );
                }
            } else {
                self.bus.finish(run, role, call_id, &evidence)?;
            }
            if !self.state.events()?.iter().any(|event| {
                event["kind"] == "transport_finished" && event["data"]["call_id"] == call_id
            }) {
                self.state
                    .event("transport_finished", &json!({"call_id":call_id}))?;
            }
        }
        self.status()
    }
    fn incoming(&self, role: &str) -> Result<Vec<Value>, String> {
        if role == "assessor" {
            return Ok(vec![self.request()]);
        }
        let effect = self
            .state
            .effect("worker:assessor")?
            .ok_or("No accepted assessor handoff is ready")?;
        if effect["status"] != "committed" || effect["result"]["accepted"] != true {
            return Err("No accepted assessor handoff is ready".into());
        }
        Ok(array(&effect, "outbox")?.clone())
    }
    fn receive(&self, role: &str, expected: &[Value]) -> Result<Vec<Value>, String> {
        let run = string(&self.config, "run_id")?;
        let wrappers = self.bus.receive(run, role, 8)?;
        let wrappers = wrappers
            .as_array()
            .ok_or("Addressed delivery must be an array")?;
        if wrappers.len() != expected.len() {
            return Err("Addressed delivery differs from the expected handoff".into());
        }
        let mut received = Vec::new();
        for (wrapper, payload) in wrappers.iter().zip(expected) {
            closed(
                wrapper,
                &["message", "sequence", "redelivered"],
                "Canonical delivery wrapper",
            )?;
            if number(wrapper, "sequence")? == 0
                || !wrapper["redelivered"].is_boolean()
                || canonical_json(&wrapper["message"])? != canonical_json(&envelope(run, payload)?)?
            {
                return Err("Canonical delivery envelope or receipt metadata differs".into());
            }
            received.push(wrapper["message"]["payload"].clone());
        }
        self.state
            .event("received", &json!({"role":role,"receipts":wrappers}))?;
        Ok(received)
    }
    pub fn reserve_worker(&self, role: &str, execution_mode: &str) -> Result<Value, String> {
        if !["assessor", "reviewer"].contains(&role)
            || !["actual_host_agent", "provider", "software_test"].contains(&execution_mode)
        {
            return Err("Unknown worker role or execution mode".into());
        }
        let _guard = self.lifecycle.hold(Duration::from_secs(30))?;
        self.active()?;
        if let Some(prior) = self.state.effect(&format!("worker:{role}"))? {
            if prior["payload"]["execution_mode"] != execution_mode {
                return Err("A frozen worker execution mode cannot change".into());
            }
            return Ok(
                json!({"new":false,"state":prior["status"],"call_id":prior["payload"]["call_id"],"context":prior["payload"]["context"]}),
            );
        }
        self.resume_inner()?;
        let incoming = self.receive(role, &self.incoming(role)?)?;
        let count = self
            .state
            .effects()?
            .iter()
            .filter(|effect| {
                effect["key"]
                    .as_str()
                    .is_some_and(|key| key.starts_with("worker:"))
            })
            .count();
        if count >= self.budget("max_worker_calls")? {
            return Err("Worker call budget exhausted".into());
        }
        let run = string(&self.config, "run_id")?;
        let call_id = format!("{run}:{role}");
        let schema = eal_runtime::private_sqlite::ensure_immutable_file(
            self.directory.join("response-schema.json"),
            crate::RESPONSE_SCHEMA.as_bytes(),
        )?;
        let gateway = self
            .host
            .eal_coordinate
            .to_str()
            .ok_or("Gateway executable path must be UTF-8")?;
        let context = json!({"schema":"EAL/coordination-worker-context/1","role":role,"call_id":call_id,"task":self.anchor,"received_messages":incoming,
            "gateway":{"argv":[gateway,if role=="assessor"{"query"}else{"lookup"},"--run",self.directory,"--call-id",call_id]},
            "response_schema":schema,"instructions":if role=="assessor"{"Inspect your addressed messages. Request a finding using the gateway argv and author an outcome or disposition with its exact trusted references. Return message and consumed_message_ids."}else{"Inspect your addressed handoff. Retrieve its finding by adding --finding-id to the gateway argv. Independently author the scoped answer, preserving every claim status, assessment ID and assessment time. Return answer and consumed_message_ids."},
            "qualifications":"EAL status is conditional on the registered argument. Unsupported is not proof of the opposite. Authored prose and measurements are not independently certified by registration."});
        let payload = json!({"role":role,"call_id":call_id,"execution_mode":execution_mode,"context":context});
        let reserved = self
            .state
            .reserve_effect(&format!("worker:{role}"), &payload)?;
        if reserved.status != EffectStatus::New {
            return Ok(
                json!({"new":false,"state":reserved.status,"call_id":call_id,"context":context}),
            );
        }
        self.bus.reserve(run, role, &call_id)?;
        self.state.event(
            "worker_reserved",
            &json!({"role":role,"call_id":call_id,"execution_mode":execution_mode}),
        )?;
        Ok(json!({"new":true,"state":"reserved","call_id":call_id,"context":context}))
    }
}
