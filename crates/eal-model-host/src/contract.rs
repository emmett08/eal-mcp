//! Boundaries shared by inference, policy and native execution.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

pub type Catalogue = BTreeMap<String, Value>;
pub type HostState = Map<String, Value>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Generation {
    pub text: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    #[serde(default)]
    pub metadata: Value,
}
impl Generation {
    pub fn scripted(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            input_tokens: 0,
            output_tokens: 0,
            metadata: json!({"backend":"scripted"}),
        }
    }
    pub fn trace_metadata(&self) -> Value {
        let mut value = self.metadata.as_object().cloned().unwrap_or_default();
        value.insert("input_tokens".into(), self.input_tokens.into());
        value.insert("output_tokens".into(), self.output_tokens.into());
        Value::Object(value)
    }
}

pub trait Generator {
    fn generate(&mut self, messages: &[Value], max_new_tokens: usize) -> Result<Generation>;
}

/// Replays authored actions, permitting contract verification without model inference.
pub struct ScriptGenerator {
    actions: std::collections::VecDeque<String>,
}
impl ScriptGenerator {
    pub fn new(actions: impl IntoIterator<Item = String>) -> Self {
        Self {
            actions: actions.into_iter().collect(),
        }
    }
}
impl Generator for ScriptGenerator {
    fn generate(&mut self, _messages: &[Value], _max_new_tokens: usize) -> Result<Generation> {
        self.actions
            .pop_front()
            .map(Generation::scripted)
            .ok_or_else(|| anyhow::anyhow!("scripted generation exhausted"))
    }
}

#[derive(Debug)]
pub enum SessionError {
    /// Arguments were rejected before sending, so another proposal is safe.
    Admission(String),
    /// Execution may have happened. Stop without replaying this request.
    Transport(String),
}
impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admission(v) | Self::Transport(v) => f.write_str(v),
        }
    }
}
impl std::error::Error for SessionError {}

pub trait Session {
    fn tools(&self) -> &Catalogue;
    /// Requests admitted for SDK dispatch, including uncertain transport outcomes.
    fn dispatched_tool_calls(&self) -> Option<usize> {
        None
    }
    fn call(
        &mut self,
        name: &str,
        arguments: &Value,
    ) -> impl std::future::Future<Output = std::result::Result<Value, SessionError>>;
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PromptVariant {
    #[default]
    Baseline,
    ExplicitLifecycle,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompletionGuard {
    #[default]
    Baseline,
    /// Applied only when the operator task declares `requires_assessment: true`.
    AssessmentRequested,
}

#[derive(Clone, Debug)]
pub struct RunOptions {
    pub max_steps: usize,
    pub max_new_tokens: usize,
    pub prompt: PromptVariant,
    pub completion_guard: CompletionGuard,
    /// Canonicalise object keys when detecting completed-call replays.
    pub canonical_call_identity: bool,
    /// Opt-in observational catalogue scope, including a collection prerequisite for reason.
    pub evidence_guided: bool,
    pub state: Option<HostState>,
}
impl Default for RunOptions {
    fn default() -> Self {
        Self {
            max_steps: 6,
            max_new_tokens: 160,
            prompt: PromptVariant::Baseline,
            completion_guard: CompletionGuard::Baseline,
            canonical_call_identity: false,
            evidence_guided: false,
            state: None,
        }
    }
}

/// All executable, workspace and registry choices belong to the operator.
#[derive(Clone, Debug)]
pub struct SessionConfig {
    pub executable: PathBuf,
    pub workspace: PathBuf,
    pub registry: Option<PathBuf>,
    pub known_entries: Vec<String>,
    pub timeout: Duration,
    pub environment: BTreeMap<String, String>,
    pub max_result_bytes: usize,
}
impl SessionConfig {
    pub fn new(executable: PathBuf, workspace: PathBuf) -> Self {
        Self {
            executable,
            workspace,
            registry: None,
            known_entries: Vec::new(),
            timeout: Duration::from_secs(30),
            environment: BTreeMap::new(),
            max_result_bytes: 4 * 1024 * 1024,
        }
    }
}
