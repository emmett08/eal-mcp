//! Whole-exchange history policies and explicit usage accounting.
mod budget;
mod usage;
pub use budget::TurnBudget;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fmt;
pub use usage::{deadline_restricted_latency, normalise_usage};

pub const DEFAULT_LIMITS: (usize, usize) = (16_384, 1_024);
pub const RELOAD_LADDER: [(usize, usize); 3] = [(4_096, 1_024), DEFAULT_LIMITS, (65_536, 4_096)];

pub fn compact_bytes(value: &Value) -> usize {
    serde_json::to_vec(value).expect("JSON value").len()
}

/// Stable metadata presentation uses sorted keys and explicit spaces.
fn metadata_text(value: &Value) -> String {
    fn sorted(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut pairs: Vec<_> = map.iter().collect();
                pairs.sort_by_key(|(key, _)| *key);
                Value::Object(
                    pairs
                        .into_iter()
                        .map(|(k, v)| (k.clone(), sorted(v)))
                        .collect(),
                )
            }
            Value::Array(values) => Value::Array(values.iter().map(sorted).collect()),
            value => value.clone(),
        }
    }
    let source = serde_json::to_string(&sorted(value)).expect("JSON metadata");
    let mut result = String::new();
    let (mut quoted, mut escaped) = (false, false);
    for c in source.chars() {
        result.push(c);
        if quoted {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
        } else if c == '"' {
            quoted = true;
        } else if c == ',' || c == ':' {
            result.push(' ');
        }
    }
    result
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HistoryError {
    Invalid(&'static str),
    ContextOverflow,
    RetrievalExhausted,
    Permission,
    Missing,
    Timeout,
    Retrieval(String),
}
impl fmt::Display for HistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => f.write_str(message),
            Self::ContextOverflow => {
                f.write_str("protected brief/index/tools and current turn exceed the input cap")
            }
            Self::RetrievalExhausted => f.write_str("retrieval or user-turn cycle limit exhausted"),
            Self::Permission => f.write_str("operation belongs to a different history condition"),
            Self::Missing => f.write_str("exact historical reference is unavailable"),
            Self::Timeout => f.write_str("original user-turn deadline exhausted"),
            Self::Retrieval(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for HistoryError {}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reference {
    pub entry_id: String,
    pub claim: String,
    pub assessment_id: String,
    pub source_digest: String,
    pub context_fingerprint: String,
    pub method_registry_fingerprint: String,
    pub assessed_at: String,
}
impl Reference {
    fn same_key(&self, other: &Self) -> bool {
        self.entry_id == other.entry_id
            && self.assessment_id == other.assessment_id
            && self.claim == other.claim
    }
}
#[derive(Clone, Debug)]
pub struct Block {
    pub messages: Vec<Value>,
    pub reference: Option<Reference>,
    pub packet: bool,
}
#[derive(Clone, Debug)]
pub struct Turn {
    pub number: usize,
    pub blocks: Vec<Block>,
    pub completed: bool,
}

pub struct History {
    pub condition: char,
    protected: Vec<Value>,
    tools: Vec<Value>,
    pub turns: Vec<Turn>,
    pub index: Vec<Reference>,
    cache: Vec<Block>,
    levels: Vec<(usize, Reference, usize)>,
    pub events: Vec<Value>,
    pub logical_lookups: usize,
    pub cache_hits: usize,
    pub server_reloads: usize,
    next_turn: usize,
}
impl History {
    pub fn new(
        condition: char,
        brief: &str,
        glossary: Value,
        tools: Vec<Value>,
    ) -> Result<Self, HistoryError> {
        if !matches!(condition, 'A' | 'B' | 'C') {
            return Err(HistoryError::Invalid("condition must be A, B or C"));
        }
        Ok(Self {
            condition,
            protected: vec![
                json!({"role":"system","content":brief}),
                json!({"role":"user","content":metadata_text(&json!({"identifier_glossary":glossary}))}),
            ],
            tools,
            turns: vec![],
            index: vec![],
            cache: vec![],
            levels: vec![],
            events: vec![],
            logical_lookups: 0,
            cache_hits: 0,
            server_reloads: 0,
            next_turn: 1,
        })
    }
    pub fn current(&self) -> Result<&Turn, HistoryError> {
        self.turns
            .last()
            .filter(|turn| !turn.completed)
            .ok_or(HistoryError::Invalid("no current turn"))
    }
    fn current_mut(&mut self) -> Result<&mut Turn, HistoryError> {
        self.turns
            .last_mut()
            .filter(|turn| !turn.completed)
            .ok_or(HistoryError::Invalid("no current turn"))
    }
    pub fn begin_turn(&mut self, content: &str) -> Result<usize, HistoryError> {
        if self.turns.last().is_some_and(|turn| !turn.completed) {
            return Err(HistoryError::Invalid("finish the current turn first"));
        }
        let number = self.next_turn;
        self.next_turn += 1;
        self.turns.push(Turn {
            number,
            blocks: vec![Block {
                messages: vec![json!({"role":"user","content":content})],
                reference: None,
                packet: false,
            }],
            completed: false,
        });
        if self.condition != 'A' {
            for old in &mut self.turns {
                if number - old.number < 4 {
                    continue;
                }
                for block in &mut old.blocks {
                    if !block.packet {
                        continue;
                    }
                    let reference = block.reference.as_ref().expect("packet reference");
                    block.messages = vec![
                        json!({"role":"user","content":metadata_text(&json!({"host_reference":reference}))}),
                    ];
                    block.packet = false;
                    self.events.push(json!({"type":"packet_eviction","at_turn":number,"exposure_turn":old.number,"reference":reference}));
                }
            }
        }
        Ok(number)
    }
    pub fn add_packet(
        &mut self,
        reference: &Reference,
        messages: Vec<Value>,
        limits: (usize, usize),
    ) -> Result<(), HistoryError> {
        self.current()?;
        if matches!(self.condition, 'A' | 'B') && limits != DEFAULT_LIMITS {
            return Err(HistoryError::Invalid(
                "A/B can expose only the default packet budget",
            ));
        }
        let mut calls = vec![];
        let mut results = vec![];
        for message in &messages {
            if let Some(items) = message.get("tool_calls") {
                for call in items
                    .as_array()
                    .ok_or(HistoryError::Invalid("tool_calls must be an array"))?
                {
                    calls.push(
                        call["id"]
                            .as_str()
                            .ok_or(HistoryError::Invalid("tool call ID must be a string"))?
                            .to_owned(),
                    );
                }
            }
            if message["role"] == "tool" {
                results.push(
                    message["tool_call_id"]
                        .as_str()
                        .ok_or(HistoryError::Invalid("tool result ID must be a string"))?
                        .to_owned(),
                );
            }
        }
        if calls.is_empty()
            || calls != results
            || calls
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != calls.len()
        {
            return Err(HistoryError::Invalid(
                "packet requires a complete ordered tool-call/result exchange",
            ));
        }
        if self
            .index
            .iter()
            .any(|item| item.same_key(reference) && item != reference)
        {
            return Err(HistoryError::Invalid(
                "an assessment reference cannot change identity or original time",
            ));
        }
        let block = Block {
            messages,
            reference: Some(reference.clone()),
            packet: true,
        };
        self.current_mut()?.blocks.push(block.clone());
        if !self.index.iter().any(|item| item.same_key(reference)) {
            self.index.push(reference.clone());
        }
        if self.condition == 'B'
            && !self.cache.iter().any(|b| {
                b.reference
                    .as_ref()
                    .is_some_and(|item| item.same_key(reference))
            })
        {
            self.cache.push(block);
        }
        Ok(())
    }
    pub fn add_messages(&mut self, messages: Vec<Value>) -> Result<(), HistoryError> {
        self.current_mut()?.blocks.push(Block {
            messages,
            reference: None,
            packet: false,
        });
        Ok(())
    }
    pub fn finish_turn(&mut self, answer: &str) -> Result<(), HistoryError> {
        self.add_messages(vec![json!({"role":"assistant","content":answer})])?;
        self.current_mut()?.completed = true;
        Ok(())
    }
    pub fn request(&self) -> Value {
        let mut messages = self.protected.clone();
        messages.push(
            json!({"role":"user","content":metadata_text(&json!({"reference_index":self.index}))}),
        );
        messages.extend(
            self.turns
                .iter()
                .flat_map(|turn| &turn.blocks)
                .flat_map(|block| block.messages.clone()),
        );
        json!({"messages":messages,"tools":self.tools})
    }
    pub fn prepare(
        &mut self,
        counter: impl Fn(&Value) -> Result<usize, HistoryError>,
        cap: usize,
    ) -> Result<Value, HistoryError> {
        if cap == 0 {
            return Err(HistoryError::Invalid("cap must be a positive integer"));
        }
        self.current()?;
        loop {
            let request = self.request();
            let count = counter(&request)?;
            if count <= cap {
                return Ok(request);
            }
            let number = self.current()?.number;
            let Some(position) = self.turns.iter().position(|turn| turn.completed) else {
                self.events.push(
                    json!({"type":"context_overflow","at_turn":number,"count":count,"cap":cap}),
                );
                return Err(HistoryError::ContextOverflow);
            };
            let removed = self.turns.remove(position);
            self.events.push(json!({"type":"emergency_turn_eviction","at_turn":number,"removed_turn":removed.number,"count_before":count,"cap":cap}));
        }
    }
    pub fn cache_lookup(&mut self, reference: &Reference) -> Result<Vec<Value>, HistoryError> {
        if self.condition != 'B' {
            return Err(HistoryError::Permission);
        }
        let number = self.current()?.number;
        self.logical_lookups += 1;
        let block = self
            .cache
            .iter()
            .find(|block| block.reference.as_ref() == Some(reference));
        let kind = if block.is_some() {
            "cache_hit"
        } else {
            "cache_miss"
        };
        self.events
            .push(json!({"type":kind,"at_turn":number,"reference":reference}));
        let block = block.ok_or(HistoryError::Missing)?;
        self.cache_hits += 1;
        Ok(block.messages.clone())
    }
    pub fn reload<T>(
        &mut self,
        reference: &Reference,
        retrieve: impl FnOnce(&Reference, (usize, usize)) -> Result<T, String>,
    ) -> Result<T, HistoryError> {
        if self.condition != 'C' {
            return Err(HistoryError::Permission);
        }
        if !self.index.iter().any(|item| item == reference) {
            return Err(HistoryError::Missing);
        }
        let number = self.current()?.number;
        let position = match self
            .levels
            .iter()
            .position(|(n, item, _)| *n == number && item.same_key(reference))
        {
            Some(position) => position,
            None => {
                self.levels.push((number, reference.clone(), 0));
                self.levels.len() - 1
            }
        };
        let level = &mut self.levels[position].2;
        let limits = *RELOAD_LADDER
            .get(*level)
            .ok_or(HistoryError::RetrievalExhausted)?;
        *level += 1;
        self.logical_lookups += 1;
        self.server_reloads += 1;
        self.events.push(json!({"type":"service_reload","at_turn":number,"reference":reference,"limits":[limits.0,limits.1]}));
        retrieve(reference, limits).map_err(HistoryError::Retrieval)
    }
}

pub fn exchange(call_id: &str, content: &str) -> Vec<Value> {
    vec![
        json!({"role":"assistant","tool_calls":[{"id":call_id,"type":"function","function":{"name":"eal_assess_known","arguments":"{\"entry_id\":\"entry\",\"claim\":\"ready\"}"}}]}),
        json!({"role":"tool","tool_call_id":call_id,"content":content}),
    ]
}

#[cfg(test)]
mod tests;
