//! Reconstruct call receipts from immutable frames before reporting journal state.
use super::{canonical_json, digest, key, read_frame};
use eal_coordination_contracts::{ClientMessage, RoundResult, ServerMessage, Session};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

struct ExpectedCall {
    payload: Value,
    result: Option<Value>,
    required: bool,
}

#[derive(Default)]
struct Replay {
    session: Option<Session>,
    calls: BTreeMap<String, ExpectedCall>,
    pending: Option<usize>,
    accepted: BTreeMap<usize, usize>,
    required_rounds: BTreeSet<usize>,
    rejected: bool,
    terminal: bool,
    completed_frame: bool,
    failed_message: Option<String>,
}

pub(super) fn validate(
    directory: &Path,
    config: Option<&Value>,
    events: &[Value],
    effects: &[Value],
) -> Result<(), String> {
    let mut replay = Replay::default();
    for (frame_count, event) in events
        .iter()
        .filter(|event| event["kind"] == "session_frame")
        .enumerate()
    {
        let receipt = &event["data"];
        if receipt["index"].as_u64() != Some(frame_count as u64) {
            return Err("journal frame sequence differs".into());
        }
        let bytes = read_frame(directory, receipt)?;
        if replay.terminal {
            return Err("journal contains frames after its terminal frame".into());
        }
        if receipt["complete"] == false {
            if receipt["direction"] != "in" {
                return Err("only rejected input may be an incomplete frame".into());
            }
            replay.require_pending();
            replay.rejected = true;
            continue;
        }
        if receipt["complete"] != true || bytes.last() != Some(&b'\n') {
            return Err("journal complete frame lacks a terminating newline".into());
        }
        match receipt["direction"].as_str() {
            Some("in") => replay.input(&bytes, receipt, config)?,
            Some("out") => replay.output(&bytes, receipt, config)?,
            _ => return Err("journal direction differs".into()),
        }
    }
    if config.is_some() && replay.session.is_none() {
        return Err("frozen configuration lacks its valid retained start frame".into());
    }
    replay.check_effects(effects)?;
    replay.check_events(events, config, effects)
}

impl Replay {
    fn require_pending(&mut self) {
        if let Some(round) = self.pending {
            for call in self.calls.values_mut() {
                if call.payload["round"].as_u64() == Some(round as u64) {
                    call.required = true;
                }
            }
        }
    }

    fn input(
        &mut self,
        bytes: &[u8],
        receipt: &Value,
        config: Option<&Value>,
    ) -> Result<(), String> {
        if self.rejected {
            return Err("journal continues input after its rejected frame".into());
        }
        self.require_pending();
        let Ok(message) = serde_json::from_slice::<ClientMessage>(bytes) else {
            self.rejected = true;
            return Ok(());
        };
        match message {
            ClientMessage::Start { spec } if receipt["index"] == 0 => {
                if let Some(config) = config
                    && serde_json::to_value(&spec).map_err(|error| error.to_string())?
                        != config["spec"]
                {
                    return Err("journal start differs from its frozen specification".into());
                }
                match Session::new(spec) {
                    Ok(session) => self.session = Some(session),
                    Err(_) => self.rejected = true,
                }
            }
            ClientMessage::Results { result } if self.pending.is_some() => {
                self.results(result, receipt)?;
            }
            _ => self.rejected = true,
        }
        Ok(())
    }

    fn results(&mut self, result: RoundResult, receipt: &Value) -> Result<(), String> {
        let session = self
            .session
            .as_mut()
            .ok_or("journal results lack an admitted session")?;
        let responses = result.responses.clone();
        if session.accept(result).is_err() {
            self.rejected = true;
            return Ok(());
        }
        let round = self
            .pending
            .take()
            .ok_or("journal results lack a dispatch")?;
        self.accepted.insert(round, responses.len());
        for response in responses {
            let expected = self
                .calls
                .get_mut(&key(response.round, response.participant))
                .ok_or("journal result lacks its dispatch reservation")?;
            expected.result = Some(json!({
                "participant":response.participant,
                "round":response.round,
                "payload_sha256":digest(&serde_json::to_vec(&response.payload).map_err(|error|error.to_string())?),
                "response_frame":receipt,
            }));
        }
        Ok(())
    }

    fn output(
        &mut self,
        bytes: &[u8],
        receipt: &Value,
        config: Option<&Value>,
    ) -> Result<(), String> {
        let message: ServerMessage = serde_json::from_slice(bytes)
            .map_err(|_| "journal output frame is not a valid server message")?;
        match message {
            ServerMessage::Failed { message } => {
                self.failed_message = Some(message);
                self.terminal = true;
            }
            ServerMessage::Dispatch { dispatch } => {
                if self.rejected || config.is_none() {
                    return Err("journal dispatch lacks a frozen admitted session".into());
                }
                let session = self
                    .session
                    .as_mut()
                    .ok_or("journal dispatch lacks its start frame")?;
                let expected = session
                    .dispatch()
                    .map_err(|_| "journal dispatch violates its round barrier")?;
                if dispatch != expected {
                    return Err("journal dispatch differs from the admitted topology".into());
                }
                if dispatch.round > 0 {
                    self.required_rounds.insert(dispatch.round - 1);
                }
                self.pending = Some(dispatch.round);
                for call in dispatch.calls {
                    self.calls.insert(key(call.round, call.participant), ExpectedCall {
                        payload: json!({"participant":call.participant,"round":call.round,"dispatch_frame":receipt}),
                        result: None,
                        // A crash during reservation can retain only a prefix.
                        required: false,
                    });
                }
            }
            ServerMessage::Completed { rounds, calls } => {
                let session = self
                    .session
                    .as_ref()
                    .ok_or("journal completion lacks its start frame")?;
                if self.rejected
                    || !session.done()
                    || rounds != session.rounds()
                    || calls != session.completed_calls()
                {
                    return Err("journal completion differs from accepted rounds".into());
                }
                self.completed_frame = true;
                self.terminal = true;
            }
        }
        Ok(())
    }

    fn check_effects(&self, effects: &[Value]) -> Result<(), String> {
        let mut observed = BTreeSet::new();
        for effect in effects {
            let identity = effect["key"]
                .as_str()
                .ok_or("journal effect lacks an identity")?;
            if !observed.insert(identity) {
                return Err("journal repeats an effect identity".into());
            }
            let expected = self
                .calls
                .get(identity)
                .ok_or("journal effect has no retained dispatch")?;
            if effect["payload"] != expected.payload
                || effect["payload_digest"].as_str()
                    != Some(digest(canonical_json(&expected.payload)?.as_bytes()).as_str())
                || effect["outbox"] != json!([])
                || effect["acknowledged"] != false
            {
                return Err("journal reservation differs from its retained dispatch".into());
            }
            match effect["status"].as_str() {
                Some("committed") if expected.result.as_ref() == Some(&effect["result"]) => {}
                Some("unknown") if effect["result"].is_null() => {}
                _ => {
                    return Err(
                        "journal response receipt differs from its retained result frame".into(),
                    );
                }
            }
        }
        if self
            .calls
            .iter()
            .any(|(identity, call)| call.required && !observed.contains(identity.as_str()))
        {
            return Err("journal is missing a published dispatch reservation".into());
        }
        Ok(())
    }

    fn check_events(
        &self,
        events: &[Value],
        config: Option<&Value>,
        effects: &[Value],
    ) -> Result<(), String> {
        let mut round_events = BTreeSet::new();
        let mut completions = 0;
        let mut admissions = 0;
        let mut failures = Vec::new();
        for event in events {
            match event["kind"].as_str() {
                Some("session_frame") => {}
                Some("session_admitted") => {
                    let config = config.ok_or("journal admission lacks frozen configuration")?;
                    if event["data"]
                        != json!({"spec_sha256":config["spec_sha256"],"expected_calls":config["expected_calls"]})
                    {
                        return Err(
                            "journal admission event differs from frozen configuration".into()
                        );
                    }
                    admissions += 1;
                }
                Some("round_completed") => {
                    let round = event["data"]["round"]
                        .as_u64()
                        .ok_or("journal round event lacks its round")?
                        as usize;
                    if !self.accepted.contains_key(&round)
                        || self.accepted.get(&round).copied()
                            != event["data"]["calls"].as_u64().map(|n| n as usize)
                        || !round_events.insert(round)
                        || self
                            .calls
                            .iter()
                            .filter(|(_, call)| call.payload["round"] == round)
                            .any(|(identity, _)| {
                                !effects.iter().any(|effect| {
                                    effect["key"] == *identity && effect["status"] == "committed"
                                })
                            })
                    {
                        return Err(
                            "journal round completion differs from committed response receipts"
                                .into(),
                        );
                    }
                }
                Some("session_completed") => {
                    let session = self
                        .session
                        .as_ref()
                        .ok_or("journal completion lacks a retained session")?;
                    if !session.done()
                        || event["data"]
                            != json!({"rounds":session.rounds(),"calls":session.total_calls()})
                    {
                        return Err("journal completion event differs from accepted rounds".into());
                    }
                    completions += 1;
                }
                Some("session_failed") => failures.push(&event["data"]),
                _ => return Err("journal contains an unknown event kind".into()),
            }
        }
        if (!self.calls.is_empty() && admissions != 1)
            || admissions > 1
            || completions > 1
            || failures.len() > 1
            || self
                .required_rounds
                .iter()
                .any(|round| !round_events.contains(round))
            || (self.completed_frame && completions != 1)
            || (completions == 1
                && (self.accepted.len() != round_events.len()
                    || effects.len() != self.calls.len()
                    || effects.iter().any(|effect| effect["status"] != "committed")))
        {
            return Err("journal lifecycle events differ from its retained execution".into());
        }
        if let Some(message) = &self.failed_message
            && failures.first().map(|failure| &failure["message"])
                != Some(&Value::String(message.clone()))
        {
            return Err("journal failure frame differs from its retained diagnostic".into());
        }
        Ok(())
    }
}
