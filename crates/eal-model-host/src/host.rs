//! Persistent official-SDK MCP execution with task-bound proposal admission.
use crate::contract::{
    Catalogue, CompletionGuard, Generator, HostState, RunOptions, Session, SessionConfig,
    SessionError,
};
use crate::policy::{self, Action};
use anyhow::{Result, anyhow, bail};
use eal_mcp_client::{
    client::{self, ClientTarget, McpClient, StdioTarget},
    schema,
};
use rmcp::model::{CallToolRequestParams, CallToolResponse, PaginatedRequestParams};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};

pub struct NativeSession {
    client: McpClient,
    pub tools: Catalogue,
    validators: BTreeMap<String, jsonschema::Validator>,
    timeout: Duration,
    max_result_bytes: usize,
    terminal: bool,
    dispatched: usize,
    pub initialization: Value,
    pub discovery_pages: usize,
}
impl NativeSession {
    pub async fn connect(config: SessionConfig) -> Result<Self> {
        if config.timeout.is_zero() || config.timeout > Duration::from_secs(3600) {
            bail!("timeout must be positive and at most 3600 seconds");
        }
        if !(1..=16 * 1024 * 1024).contains(&config.max_result_bytes) {
            bail!("result byte bound must be between 1 and 16 MiB");
        }
        let executable = config.executable.canonicalize()?;
        let workspace = config.workspace.canonicalize()?;
        if !executable.is_file() || !workspace.is_dir() {
            bail!("executable must be a file and workspace a directory");
        }
        let mut arguments = vec![
            "--workspace".into(),
            workspace
                .to_str()
                .ok_or_else(|| anyhow!("workspace requires UTF-8"))?
                .into(),
            "--transport".into(),
            "stdio".into(),
        ];
        if let Some(registry) = config.registry {
            let path = registry.canonicalize()?;
            if !path.is_file() {
                bail!("registry must be an existing file");
            }
            arguments.extend([
                "--registry".into(),
                path.to_str()
                    .ok_or_else(|| anyhow!("registry requires UTF-8"))?
                    .into(),
            ]);
        }
        arguments.extend([
            "--exposure".into(),
            if config.known_entries.is_empty() {
                "operator"
            } else {
                "registered"
            }
            .into(),
        ]);
        let mut known = BTreeSet::new();
        for entry in config.known_entries {
            if entry.is_empty() || !known.insert(entry.clone()) {
                bail!("known entries must be distinct nonempty identifiers");
            }
            arguments.extend(["--known-entry".into(), entry]);
        }
        let mut environment: BTreeMap<String, String> = [
            "PATH",
            "HOME",
            "USER",
            "LOGNAME",
            "TMPDIR",
            "TMP",
            "TEMP",
            "LANG",
            "LC_ALL",
            "SYSTEMROOT",
            "WINDIR",
        ]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok().map(|value| (key.to_owned(), value)))
        .collect();
        environment.extend(
            config
                .environment
                .into_iter()
                .filter(|(key, _)| !key.starts_with("EAL_MCP_")),
        );
        let deadline = Instant::now()
            .checked_add(config.timeout)
            .ok_or_else(|| anyhow!("timeout exceeds clock bounds"))?;
        let client = client::connect(
            ClientTarget::Stdio(StdioTarget {
                command: executable,
                arguments,
                environment,
                cwd: Some(workspace),
            }),
            deadline,
        )
        .await
        .map_err(|error| anyhow!(error))?;
        let discovery = tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            Self::discover(&client),
        )
        .await;
        let (tools, validators, discovery_pages) = match discovery {
            Ok(Ok(value)) => value,
            result => {
                let mut client = client;
                let _ = client.close().await;
                return Err(anyhow!(match result {
                    Ok(Err(error)) => error.to_string(),
                    _ => "MCP discovery exceeded its deadline".into(),
                }));
            }
        };
        let initialization = client
            .peer_info()
            .map(serde_json::to_value)
            .transpose()?
            .unwrap_or(Value::Null);
        Ok(Self {
            client,
            tools,
            validators,
            timeout: config.timeout,
            max_result_bytes: config.max_result_bytes,
            terminal: false,
            dispatched: 0,
            initialization,
            discovery_pages,
        })
    }
    async fn discover(
        client: &McpClient,
    ) -> Result<(Catalogue, BTreeMap<String, jsonschema::Validator>, usize)> {
        let mut tools = Catalogue::new();
        let mut validators = BTreeMap::new();
        let mut cursor = None;
        let mut seen = BTreeSet::new();
        let mut bytes = 0usize;
        let mut pages = 0usize;
        loop {
            if pages >= 32 {
                bail!("unbounded MCP catalogue pagination");
            }
            let mut params = PaginatedRequestParams::default();
            params.cursor = cursor;
            let page = client.list_tools(Some(params)).await?;
            pages += 1;
            bytes = bytes.saturating_add(serde_json::to_vec(&page)?.len());
            if bytes > 4 * 1024 * 1024 || tools.len() + page.tools.len() > 4096 {
                bail!("MCP catalogue exceeds configured bounds");
            }
            for tool in page.tools {
                let name = tool.name.to_string();
                if tools.contains_key(&name) {
                    bail!("duplicate discovered MCP tool");
                }
                let value = serde_json::to_value(tool)?;
                let validator =
                    schema::compile_schema(&value["inputSchema"]).map_err(|e| anyhow!(e))?;
                validators.insert(name.clone(), validator);
                tools.insert(name, value);
            }
            match page.next_cursor {
                Some(next) => {
                    if !seen.insert(next.clone()) {
                        bail!("unbounded MCP catalogue pagination");
                    }
                    cursor = Some(next);
                }
                None => break,
            }
        }
        Ok((tools, validators, pages))
    }
    pub async fn close(&mut self) -> Result<()> {
        self.terminal = true;
        self.client.close().await.map_err(|error| anyhow!(error))
    }
    pub async fn call(
        &mut self,
        name: &str,
        arguments: &Value,
    ) -> std::result::Result<Value, SessionError> {
        <Self as Session>::call(self, name, arguments).await
    }
}

/// Preserve integer-token requirements beyond JSON Schema's mathematical integer type.
pub fn validate_integer_tokens(arguments: &Value) -> Result<()> {
    fn integer(value: &Value) -> bool {
        value.as_number().is_some_and(|n| {
            let text = n.as_str();
            !text.contains(['.', 'e', 'E'])
        })
    }
    for key in ["limit", "offset"] {
        if let Some(value) = arguments.get(key)
            && !integer(value)
        {
            bail!("{key} requires an integer token");
        }
    }
    if let Some(limits) = arguments.get("packet_limits").and_then(Value::as_object) {
        for (key, value) in limits {
            if !integer(value) {
                bail!("packet_limits.{key} requires an integer token");
            }
        }
    }
    Ok(())
}
impl Session for NativeSession {
    fn dispatched_tool_calls(&self) -> Option<usize> {
        Some(self.dispatched)
    }
    fn tools(&self) -> &Catalogue {
        &self.tools
    }
    async fn call(
        &mut self,
        name: &str,
        arguments: &Value,
    ) -> std::result::Result<Value, SessionError> {
        if self.terminal {
            return Err(SessionError::Transport(
                "MCP transport is closed; reopen explicitly".into(),
            ));
        }
        let validator = self.validators.get(name).ok_or_else(|| {
            SessionError::Admission(format!("tool is unavailable in this exposure: {name}"))
        })?;
        schema::validate(arguments, validator).map_err(SessionError::Admission)?;
        validate_integer_tokens(arguments)
            .map_err(|error| SessionError::Admission(error.to_string()))?;
        let arguments = arguments
            .as_object()
            .ok_or_else(|| SessionError::Admission("tool arguments must be an object".into()))?
            .clone();
        self.dispatched += 1;
        let operation = self
            .client
            .call_tool_once(CallToolRequestParams::new(name.to_owned()).with_arguments(arguments));
        let result = tokio::time::timeout(self.timeout, operation).await;
        let value = match result {
            Ok(Ok(CallToolResponse::Complete(result))) => serde_json::to_value(result)
                .map_err(|error| SessionError::Transport(error.to_string()))?,
            other => {
                self.terminal = true;
                let _ = self.client.close().await;
                let message = match other {
                    Ok(Err(error)) => error.to_string(),
                    Ok(Ok(_)) => "unsupported interactive or deferred MCP result".into(),
                    Err(_) => "MCP call exceeded its deadline".into(),
                };
                return Err(SessionError::Transport(message));
            }
        };
        if serde_json::to_vec(&value)
            .map_err(|error| SessionError::Transport(error.to_string()))?
            .len()
            > self.max_result_bytes
        {
            self.terminal = true;
            let _ = self.client.close().await;
            return Err(SessionError::Transport(
                "MCP result exceeds the configured retained-result bound".into(),
            ));
        }
        Ok(value)
    }
}

/// Explicit operator scope for observational workflows. Native EAL reasoning
/// without a collection remains legal and available when this mode is off.
pub fn evidence_guidance_rule() -> Value {
    json!({"schema":"eal.evidence-guided-catalogue/1", "scope":"observational workflows",
        "schema_required_retained_ids":["collection_id","assessment_id"],
        "additional_operator_prerequisite":{"eal_reason":"collection_id"},
        "retained_id_requirement":"nonempty authoritative host-state string",
        "recomputed":"before every model step", "arguments_rewritten":false,
        "native_schema_changed":false, "task_kind_used":false})
}

/// Apply only operator catalogue scope. The model still chooses an offered tool
/// and supplies its arguments through the unchanged task-bound admission path.
pub fn evidence_guided_catalogue(tools: &Catalogue, state: &HostState) -> Catalogue {
    let retained = |key: &str| {
        state
            .get(key)
            .and_then(Value::as_str)
            .is_some_and(|id| !id.is_empty())
    };
    tools
        .iter()
        .filter(|(name, tool)| {
            if name.as_str() == "eal_reason" && !retained("collection_id") {
                return false;
            }
            let required = tool
                .get("inputSchema")
                .and_then(|schema| schema.get("required"))
                .and_then(Value::as_array);
            ["collection_id", "assessment_id"].iter().all(|key| {
                !required
                    .is_some_and(|fields| fields.iter().any(|field| field.as_str() == Some(*key)))
                    || retained(key)
            })
        })
        .map(|(name, tool)| (name.clone(), tool.clone()))
        .collect()
}

pub async fn run_policy<S: Session, G: Generator>(
    session: &mut S,
    task: &Value,
    generator: &mut G,
    options: &RunOptions,
) -> Result<Value> {
    if !(1..=32).contains(&options.max_steps) {
        bail!("max_steps must be between 1 and 32");
    }
    if !(1..=4096).contains(&options.max_new_tokens) {
        bail!("max_new_tokens must be between 1 and 4096");
    }
    let mut state: HostState = match &options.state {
        Some(state) => state.clone(),
        None => task
            .get("state")
            .and_then(Value::as_object)
            .cloned()
            .ok_or_else(|| anyhow!("task requires host state object"))?,
    };
    let mut tools = session.tools().clone();
    if let Some(allowed) = task.get("allowed_tools") {
        let allowed = allowed
            .as_array()
            .ok_or_else(|| anyhow!("allowed_tools must list unique discovered tool names"))?;
        let mut selected = Catalogue::new();
        for name in allowed {
            let name = name
                .as_str()
                .ok_or_else(|| anyhow!("allowed_tools must list unique discovered tool names"))?;
            if selected.contains_key(name) {
                bail!("allowed_tools must list unique discovered tool names");
            }
            selected.insert(
                name.to_owned(),
                tools.get(name).cloned().ok_or_else(|| {
                    anyhow!("allowed_tools must list unique discovered tool names")
                })?,
            );
        }
        tools = selected;
    }
    let started = Instant::now();
    let dispatches_before = session.dispatched_tool_calls();
    let mut uncertain_dispatches = 0usize;
    let mut history = Vec::new();
    let mut attempts = Vec::new();
    let mut calls = Vec::new();
    let mut completed_calls = BTreeSet::new();
    let mut stop = "step_limit".to_owned();
    let mut fresh_assessment = false;
    let mut last: Option<Value> = None;
    let mut input_tokens = 0usize;
    let mut output_tokens = 0usize;
    for _ in 0..options.max_steps {
        let scoped = options
            .evidence_guided
            .then(|| evidence_guided_catalogue(&tools, &state));
        let visible_tools = scoped.as_ref().unwrap_or(&tools);
        let messages =
            policy::messages_for_variant(task, visible_tools, &state, &history, options.prompt)?;
        let generation = match generator.generate(&messages, options.max_new_tokens) {
            Ok(generation) => generation,
            Err(error) => {
                let mut attempt = json!({"error":error.to_string(),"error_type":"generation"});
                if options.evidence_guided {
                    attempt["visible_tools"] = json!(visible_tools.keys().collect::<Vec<_>>());
                }
                attempts.push(attempt);
                stop = "generation_error".into();
                break;
            }
        };
        input_tokens += generation.input_tokens;
        output_tokens += generation.output_tokens;
        let mut attempt = json!({"text":generation.text,"generation":generation.trace_metadata()});
        if options.evidence_guided {
            attempt["visible_tools"] = json!(visible_tools.keys().collect::<Vec<_>>());
        }
        let action = policy::decode_action(&generation.text, visible_tools, &state);
        let admission_error = match action {
            Err(error) => Some(error.to_string()),
            Ok(Action::Finish { proposal }) => {
                attempt["action"] = proposal;
                let succeeded = last.as_ref().is_some_and(|last| {
                    !last["response"]
                        .get("isError")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                });
                if !succeeded {
                    Some("finish requires a completed successful tool result".into())
                } else if options.completion_guard == CompletionGuard::AssessmentRequested
                    && task.get("requires_assessment").and_then(Value::as_bool) == Some(true)
                    && !fresh_assessment
                {
                    Some("the requested assessment is incomplete; use the retained collection_id for reasoning".into())
                } else {
                    stop = "finished".into();
                    attempts.push(attempt);
                    break;
                }
            }
            Ok(Action::Stop { proposal, reason }) => {
                attempt["action"] = proposal;
                stop = reason;
                attempts.push(attempt);
                break;
            }
            Ok(Action::Call {
                proposal,
                tool,
                arguments,
            }) => {
                attempt["action"] = proposal;
                let call_identity = json!({"tool":tool,"arguments":arguments});
                let identity = serde_json::to_string(&if options.canonical_call_identity {
                    canonical_value(&call_identity)
                } else {
                    call_identity
                })?;
                if completed_calls.contains(&identity) {
                    Some("identical call already completed; use its retained result".into())
                } else {
                    let tick = Instant::now();
                    match session.call(&tool, &arguments).await {
                        Ok(reply) => {
                            completed_calls.insert(identity);
                            let call = json!({"tool":tool,"arguments":arguments,"response":reply,"elapsed_seconds":tick.elapsed().as_secs_f64()});
                            let returned_ids = policy::result_ids(policy::result_value(&reply));
                            if returned_ids.contains_key("collection_id")
                                && matches!(
                                    tool.as_str(),
                                    "eal_collect" | "eal_collect_claim" | "eal_execute"
                                )
                            {
                                fresh_assessment = false;
                            }
                            if returned_ids.contains_key("assessment_id")
                                && matches!(
                                    tool.as_str(),
                                    "eal_reason" | "eal_execute" | "eal_assess_known"
                                )
                            {
                                fresh_assessment = true;
                            }
                            state.extend(returned_ids);
                            history.push(policy::summary(&reply, &tool));
                            calls.push(call.clone());
                            last = Some(call);
                            None
                        }
                        Err(SessionError::Admission(error)) => Some(error),
                        Err(SessionError::Transport(error)) => {
                            uncertain_dispatches += 1;
                            attempt["error"] = error.into();
                            stop = "transport_error".into();
                            attempts.push(attempt);
                            break;
                        }
                    }
                }
            }
        };
        if let Some(error) = admission_error {
            attempt["error"] = error.clone().into();
            history.push(json!({"admission_error":error}));
        }
        attempts.push(attempt);
    }
    let state_ids: HostState = state
        .into_iter()
        .filter(|(key, _)| key.ends_with("_id"))
        .collect();
    let mut report = json!({"schema":"eal.tool-use-run/1","task_id":task.get("id"),"allowed_tools":tools.keys().collect::<Vec<_>>(),
        "stop":stop,"attempts":attempts,"calls":calls,"state_ids":state_ids,
        "last_result":last.as_ref().map(|last| &last["response"]),"elapsed_seconds":started.elapsed().as_secs_f64(),
        "input_tokens":input_tokens,"output_tokens":output_tokens,
        "tool_dispatches":session.dispatched_tool_calls().zip(dispatches_before).map(|(a,b)|a.saturating_sub(b)),"uncertain_tool_call_attempts":uncertain_dispatches,
        "host_variant":{"prompt":options.prompt,"completion_guard":options.completion_guard,"canonical_call_identity":options.canonical_call_identity}});
    if options.evidence_guided {
        report["host_variant"]["evidence_guided"] = evidence_guidance_rule();
        report["discovered_catalogue"] = serde_json::to_value(session.tools())?;
    }
    Ok(report)
}

/// Canonicalisation is used for call identity only, never for source or argument transport.
fn canonical_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), canonical_value(value)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        Value::Array(array) => Value::Array(array.iter().map(canonical_value).collect()),
        _ => value.clone(),
    }
}
