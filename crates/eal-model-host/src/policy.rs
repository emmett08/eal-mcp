//! Task-bound model proposals and bounded views of native results.
use crate::contract::{Catalogue, HostState, PromptVariant};
use anyhow::{Result, anyhow, bail};
use eal_collector_contracts::{JsonLimits, strict_json_exact};
use serde_json::{Value, json};

pub const SYSTEM: &str = r#"Use the discovered EAL/4 MCP tools to fulfil the request.
Return one JSON object, without prose or markdown.
Call format: {"tool":"eal_NAME","arguments":{...}}.
Use {"$state":"KEY"} as an argument value to copy a listed host-state value.
The host stores source, context, time and returned IDs. Reuse those references.
Return {"finish":true} when the requested operation has completed, including
valid unsupported findings. Return {"stop":"unavailable"} if no offered tool
can fulfil the request. After a failed call, correct the cause or stop.
Use only offered tools. Preserve source, context, time and claim identity.
Prefer execute for fresh acquisition plus assessment, reason for an existing
collection, packet for compact stored results, and explain for full details.
An acquisition result supplies observations. If the request asks to collect and
then reason, call eal_reason with the returned collection_id before finishing.
Source validation uses eal_validate. Registered-only tools assess registered
claims and provide no operation for validating arbitrary supplied source.
Historical retrieval preserves the original assessment time."#;

pub const LIFECYCLE_RULES: &str = r#"
A collection_id names observations. An assessment_id names an assessment.
For a request that asks for collection followed by reasoning, collection is an
intermediate result. Use {"$state":"collection_id"} for the returned collection
in eal_reason. Finish only after the requested assessment is available.
Use an existing assessment_id for historical retrieval. Reuse evidence without
collecting again unless the request explicitly asks for new observations.
A registered claim assessment cannot validate arbitrary supplied source. If the
discovered exposure offers no operation for the requested input, stop unavailable.
Every argument reference must name a currently listed host-state key."#;

#[derive(Clone, Debug)]
pub enum Action {
    Finish {
        proposal: Value,
    },
    Stop {
        proposal: Value,
        reason: String,
    },
    Call {
        proposal: Value,
        tool: String,
        arguments: Value,
    },
}

pub fn strict_json(text: &str) -> Result<Value> {
    let value = strict_json_exact(
        text.as_bytes(),
        JsonLimits {
            max_bytes: 65_536,
            max_depth: 64,
            max_nodes: 32_768,
        },
    )
    .map_err(|e| anyhow!(e))?;
    if !value.is_object() {
        bail!("policy action must be one JSON object");
    }
    Ok(value)
}

pub fn resolve(value: &Value, state: &HostState) -> Result<Value> {
    fn inner(value: &Value, state: &HostState, depth: usize) -> Result<Value> {
        if depth > 32 {
            bail!("state reference depth exceeds 32");
        }
        match value {
            Value::Object(object) => {
                if let Some(reference) = object.get("$state") {
                    if object.len() != 1 || !reference.is_string() {
                        bail!("state reference must contain exactly one string key");
                    }
                    let key = reference.as_str().unwrap();
                    // Clone once. An opaque copied value can legitimately contain `$state`.
                    state
                        .get(key)
                        .cloned()
                        .ok_or_else(|| anyhow!("state reference is unavailable: {key}"))
                } else {
                    Ok(Value::Object(
                        object
                            .iter()
                            .map(|(k, v)| Ok((k.clone(), inner(v, state, depth + 1)?)))
                            .collect::<Result<_>>()?,
                    ))
                }
            }
            Value::Array(array) => Ok(Value::Array(
                array
                    .iter()
                    .map(|v| inner(v, state, depth + 1))
                    .collect::<Result<_>>()?,
            )),
            Value::Number(number) if number.as_str().contains(['.', 'e', 'E']) => {
                let finite = number.as_str().parse::<f64>().is_ok_and(f64::is_finite);
                if !finite {
                    bail!("nonfinite argument");
                }
                Ok(value.clone())
            }
            _ => Ok(value.clone()),
        }
    }
    inner(value, state, 0)
}

pub fn decode_action(text: &str, tools: &Catalogue, state: &HostState) -> Result<Action> {
    let proposal = strict_json(text)?;
    let object = proposal.as_object().unwrap();
    if object.len() == 1 && proposal.get("finish") == Some(&Value::Bool(true)) {
        return Ok(Action::Finish { proposal });
    }
    if object.len() == 1
        && let Some(reason @ ("unavailable" | "failed")) =
            proposal.get("stop").and_then(Value::as_str)
    {
        return Ok(Action::Stop {
            reason: reason.to_owned(),
            proposal,
        });
    }
    if object.len() != 2
        || !object.contains_key("tool")
        || !proposal.get("arguments").is_some_and(Value::is_object)
    {
        bail!("expected tool/arguments, finish:true or stop");
    }
    let tool = proposal["tool"]
        .as_str()
        .filter(|name| tools.contains_key(*name))
        .ok_or_else(|| anyhow!("tool is unavailable in the discovered exposure"))?
        .to_owned();
    let arguments = resolve(&proposal["arguments"], state)?;
    for (key, value) in arguments.as_object().unwrap() {
        if matches!(key.as_str(), "assessment_id" | "collection_id")
            && !state.contains_key(key)
            && !value.is_null()
        {
            bail!("authoritative {key} is unavailable");
        }
        if let Some(original) = state.get(key)
            && original != value
        {
            bail!("task-bound {key} changed");
        }
    }
    Ok(Action::Call {
        proposal,
        tool,
        arguments,
    })
}

pub fn result_value(reply: &Value) -> Option<&Value> {
    if reply
        .get("isError")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }
    reply.get("structuredContent").filter(|v| !v.is_null())
}

pub fn result_ids(value: Option<&Value>) -> HostState {
    let mut result = HostState::new();
    let Some(value) = value.filter(|v| v.is_object()) else {
        return result;
    };
    for boundary in std::iter::once(value).chain(
        ["assessment", "collection", "packet"]
            .iter()
            .filter_map(|key| value.get(*key))
            .filter(|v| v.is_object()),
    ) {
        for key in ["assessment_id", "collection_id"] {
            if let Some(id) = boundary
                .get(key)
                .and_then(Value::as_str)
                .filter(|v| !v.is_empty())
            {
                result.insert(key.to_owned(), Value::String(id.to_owned()));
            }
        }
    }
    result
}

pub fn summary(reply: &Value, tool: &str) -> Value {
    let mut result = json!({"tool":tool,"is_error":reply.get("isError").and_then(Value::as_bool).unwrap_or(false)});
    let object = result.as_object_mut().unwrap();
    match result_value(reply) {
        None => {
            let texts = reply
                .get("content")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|v| v.get("type").and_then(Value::as_str) == Some("text"))
                .map(|v| v.get("text").and_then(Value::as_str).unwrap_or(""))
                .collect::<Vec<_>>()
                .join("\n");
            object.insert(
                "error".into(),
                texts.chars().take(1500).collect::<String>().into(),
            );
        }
        Some(Value::Object(value)) => {
            object.extend(result_ids(Some(&Value::Object(value.clone()))));
            for key in ["valid", "complete", "status", "diagnostics", "labels"] {
                if let Some(v) = value.get(key) {
                    object.insert(key.into(), v.clone());
                }
            }
            let assessment = value
                .get("assessment")
                .and_then(Value::as_object)
                .unwrap_or(value);
            for key in ["valid", "complete", "claims"] {
                if let Some(v) = assessment.get(key) {
                    object.insert(key.into(), v.clone());
                }
            }
            if serde_json::to_vec(&result).unwrap_or_default().len() > 3000 {
                let object = result.as_object_mut().unwrap();
                for key in ["claims", "diagnostics", "labels"] {
                    object.remove(key);
                }
                object.insert("details_omitted".into(), true.into());
            }
            result
                .as_object_mut()
                .unwrap()
                .insert("result_available".into(), true.into());
        }
        Some(_) => {
            object.insert("result_available".into(), true.into());
        }
    }
    result
}

pub fn catalogue_view(tools: &Catalogue) -> String {
    tools
        .iter()
        .map(|(name, tool)| {
            let schema = &tool["inputSchema"];
            let required = schema
                .get("required")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let mut fields: Vec<String> = required
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect();
            if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
                fields.extend(
                    properties
                        .keys()
                        .filter(|k| !required.iter().any(|r| r.as_str() == Some(k)))
                        .map(|key| format!("{key}?")),
                );
            }
            format!(
                "{name}({}): {}",
                fields.join(","),
                tool.get("description")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn messages_for(
    task: &Value,
    tools: &Catalogue,
    state: &HostState,
    history: &[Value],
) -> Result<Vec<Value>> {
    messages_for_variant(task, tools, state, history, PromptVariant::Baseline)
}

pub fn messages_for_variant(
    task: &Value,
    tools: &Catalogue,
    state: &HostState,
    history: &[Value],
    variant: PromptVariant,
) -> Result<Vec<Value>> {
    let request = task
        .get("request")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("task requires a request string"))?;
    let visible: HostState = state
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                if matches!(key.as_str(), "source" | "context") {
                    json!({"stored":true})
                } else {
                    value.clone()
                },
            )
        })
        .collect();
    let mut prompt = format!(
        "Request: {request}\nHost state: {}\nAvailable tools:\n{}",
        serde_json::to_string(&visible)?,
        catalogue_view(tools)
    );
    if !history.is_empty() {
        prompt.push_str("\nRecent calls: ");
        prompt.push_str(&serde_json::to_string(
            &history[history.len().saturating_sub(3)..],
        )?);
    }
    let system = match variant {
        PromptVariant::Baseline => SYSTEM.to_owned(),
        PromptVariant::ExplicitLifecycle => format!("{SYSTEM}{LIFECYCLE_RULES}"),
    };
    Ok(vec![
        json!({"role":"system","content":system}),
        json!({"role":"user","content":prompt}),
    ])
}
