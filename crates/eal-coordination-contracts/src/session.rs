//! Atomic complete-round state transitions and graph-limited payload routing.

use crate::{
    CallContext, Dispatch, Error, Graph, MAX_PAYLOAD_BYTES, PeerValue, RoundResult, SCHEMA_VERSION,
    SessionSpec, frame_budget, payload,
};
use serde_json::Value;

/// One frozen topology session. Round 0 is independent, followed by peer rounds.
pub struct Session {
    spec: SessionSpec,
    graphs: Vec<Graph>,
    next_round: usize,
    awaiting_results: bool,
    previous: Vec<PeerValue>,
    previous_payload_bytes: Vec<usize>,
}

impl Session {
    /// Admit all shape, schedule and payload bounds before issuing any calls.
    pub fn new(spec: SessionSpec) -> Result<Self, Error> {
        if spec.schema_version != SCHEMA_VERSION {
            return Err(Error::InvalidInput(format!(
                "debate schema_version must be {SCHEMA_VERSION}"
            )));
        }
        if !(1..=MAX_PAYLOAD_BYTES).contains(&spec.max_payload_bytes) {
            return Err(Error::InvalidInput(format!(
                "max_payload_bytes must be 1–{MAX_PAYLOAD_BYTES}"
            )));
        }
        let graphs = spec.debate.graphs()?;
        Ok(Self {
            spec,
            graphs,
            next_round: 0,
            awaiting_results: false,
            previous: Vec::new(),
            previous_payload_bytes: Vec::new(),
        })
    }

    /// Whether the next round may be dispatched.
    pub fn ready(&self) -> bool {
        !self.done() && !self.awaiting_results
    }

    /// Whether every independent and peer round has been accepted.
    pub fn done(&self) -> bool {
        self.next_round == self.rounds()
    }

    /// Number of independent and peer rounds, including round 0.
    pub fn rounds(&self) -> usize {
        usize::from(self.spec.debate.rounds) + 1
    }

    /// Number of participant calls reserved by the admitted configuration.
    /// Application synthesis, criticism and adjudication are outside the session.
    pub fn total_calls(&self) -> usize {
        usize::from(self.spec.debate.participants) * self.rounds()
    }

    /// Number of participant calls belonging to complete accepted rounds.
    pub fn completed_calls(&self) -> usize {
        usize::from(self.spec.debate.participants) * self.next_round
    }

    /// Issue one complete round and wait for its identified results.
    /// All peer contexts derive from the same accepted previous-round snapshot.
    pub fn dispatch(&mut self) -> Result<Dispatch, Error> {
        if !self.ready() {
            return Err(Error::Protocol(if self.done() {
                "session has completed".into()
            } else {
                "the dispatched round still requires complete results".into()
            }));
        }
        let edges = if self.next_round == 0 {
            &[][..]
        } else {
            &self.graphs[self.next_round - 1].edges
        };
        // Build only small metadata and null placeholders first. Dense graphs
        // repeat payloads, so per-payload limits alone cannot bound a dispatch.
        let calls = (0..usize::from(self.spec.debate.participants))
            .map(|participant| {
                let incoming: Vec<_> = edges
                    .iter()
                    .filter(|edge| edge.to == participant)
                    .map(|edge| edge.from)
                    .collect();
                CallContext {
                    participant,
                    round: self.next_round,
                    edges: edges.to_vec(),
                    peers: incoming
                        .iter()
                        .map(|&sender| PeerValue {
                            participant: sender,
                            round: self.previous[sender].round,
                            payload: Value::Null,
                        })
                        .collect(),
                    incoming,
                    previous: self.previous.get(participant).map(|_| Value::Null),
                }
            })
            .collect();
        let mut dispatch = Dispatch {
            round: self.next_round,
            calls,
        };
        frame_budget::check(&dispatch, &self.previous_payload_bytes)?;
        for call in &mut dispatch.calls {
            if call.previous.is_some() {
                call.previous = Some(self.previous[call.participant].payload.clone());
            }
            for peer in &mut call.peers {
                peer.payload = self.previous[peer.participant].payload.clone();
            }
        }
        self.awaiting_results = true;
        Ok(dispatch)
    }

    /// Validate a whole result without advancing state on any failure.
    /// Response ordering is immaterial. Accepted snapshots use participant order.
    pub fn accept(&mut self, result: RoundResult) -> Result<(), Error> {
        if !self.awaiting_results {
            return Err(Error::Protocol(
                "results require a preceding uncompleted dispatch".into(),
            ));
        }
        if result.round != self.next_round {
            return Err(Error::Protocol(format!(
                "expected round {}, received round {}",
                self.next_round, result.round
            )));
        }
        let participants = usize::from(self.spec.debate.participants);
        if result.responses.len() != participants {
            return Err(Error::Protocol(format!(
                "round {} requires exactly {participants} responses, received {}",
                self.next_round,
                result.responses.len()
            )));
        }
        let mut admitted = vec![None; participants];
        for response in result.responses {
            if response.participant >= participants || response.round != self.next_round {
                return Err(Error::Protocol(format!(
                    "response identity ({}, {}) does not belong to dispatched round {} with {participants} participants",
                    response.participant, response.round, self.next_round
                )));
            }
            let participant = response.participant;
            if admitted[participant].is_some() {
                return Err(Error::Protocol(format!(
                    "round {} contains duplicate participant {participant}",
                    self.next_round
                )));
            }
            payload::validate(&response.payload, self.spec.max_payload_bytes)?;
            let bytes = serde_json::to_vec(&response.payload)
                .map_err(|error| Error::Protocol(format!("payload cannot be encoded: {error}")))?
                .len();
            if bytes > self.spec.max_payload_bytes {
                return Err(Error::Protocol(format!(
                    "participant {participant} payload uses {bytes} encoded bytes, exceeding {}",
                    self.spec.max_payload_bytes
                )));
            }
            admitted[participant] = Some((response, bytes));
        }
        let admitted = admitted
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| Error::Protocol("round omits a participant".into()))?;
        (self.previous, self.previous_payload_bytes) = admitted.into_iter().unzip();
        self.next_round += 1;
        self.awaiting_results = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
