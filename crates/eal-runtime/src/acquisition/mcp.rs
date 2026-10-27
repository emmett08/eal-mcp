//! Direct named MCP calls through the shared official-SDK client.
use super::{AcquisitionError, AcquisitionResult, ToolBinding, ToolKind, normalization};
use eal_collector_contracts::ResponseFormat;
use eal_core::ExecutionLimits;
use eal_mcp_client::{
    client::{self, ClientTarget, StdioTarget},
    persistent_mcp::{PersistentMcp, ToolListLimits, ToolSchemaPins},
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) fn acquire(
    binding: &ToolBinding,
    request: &Value,
    workspace: &Path,
    secret: &[u8],
    limits: &ExecutionLimits,
) -> Result<AcquisitionResult, AcquisitionError> {
    let arguments = normalization::input(binding, request, workspace)?;
    let settings = binding.direct.as_ref().expect("validated direct settings");
    let duration = Duration::from_secs_f64(binding.timeout_seconds);
    let deadline = Instant::now() + duration;
    let mut metadata = json!({"transport":binding.kind.as_str(),"operation":settings.tool});
    let target = match binding.kind {
        ToolKind::McpStdio => {
            let argv = binding.resolved_argv(workspace)?;
            let environment = binding.effective_environment()?;
            metadata["process_environment_digest"] =
                json!(binding.process_environment_digest_with_limits(
                    secret,
                    Some(&environment),
                    limits
                )?);
            ClientTarget::Stdio(StdioTarget {
                command: PathBuf::from(&argv[0]),
                arguments: argv[1..].to_vec(),
                environment,
                cwd: Some(binding.package_root(workspace).to_owned()),
            })
        }
        ToolKind::McpHttp => {
            let environment = binding.effective_environment()?;
            metadata["process_environment_digest"] =
                json!(binding.process_environment_digest_with_limits(
                    secret,
                    Some(&environment),
                    limits
                )?);
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .no_proxy()
                .timeout(duration)
                .build()
                .map_err(|_| AcquisitionError::invalid("Cannot configure HTTP MCP client"))?;
            client::create_http_target_with_client(
                settings.url.as_deref().unwrap(),
                settings.token_env.as_deref(),
                &environment,
                client,
            )
            .map_err(AcquisitionError::invalid)?
        }
        _ => return Err("MCP adapter requires an MCP binding".into()),
    };
    let session = PersistentMcp::new(target, duration).map_err(AcquisitionError::invalid)?;
    let discovery_limits = ToolListLimits {
        max_tools: 4096,
        max_pages: 128,
        max_bytes: binding.max_output_bytes,
    };
    let tool = settings.tool.as_deref().unwrap();
    let result = if let Some(input) = &settings.input_schema_sha256 {
        session
            .call_pinned(
                tool,
                &arguments,
                &ToolSchemaPins {
                    input_sha256: input.clone(),
                    output_sha256: settings.output_schema_sha256.clone(),
                },
                discovery_limits,
                deadline,
            )
            .map(|r| r.result)
    } else {
        session.call_bounded(
            tool,
            &arguments,
            discovery_limits,
            binding.max_output_bytes,
            deadline,
        )
    }
    .map_err(|error| AcquisitionError::invalid(error.to_string()))?;
    session.close();
    if result.is_error.unwrap_or(false) {
        return Err("MCP tool reported acquisition failure".into());
    }
    let bytes = serde_json::to_vec(&result)
        .map_err(|_| AcquisitionError::invalid("Cannot encode MCP result"))?;
    if bytes.len() > binding.max_output_bytes {
        return Err("MCP result exceeds output allowance".into());
    }
    let decoded = match settings.response_format {
        ResponseFormat::Structured => result
            .structured_content
            .ok_or("MCP tool omitted structured content")?,
        ResponseFormat::JsonText => {
            let blocks = serde_json::to_value(result.content)
                .map_err(|_| AcquisitionError::invalid("Cannot inspect MCP content"))?;
            let text = blocks
                .as_array()
                .and_then(|a| a.get(settings.text_block))
                .filter(|b| b["type"] == "text")
                .and_then(|b| b["text"].as_str())
                .ok_or("MCP tool omitted configured JSON text block")?;
            eal_collector_contracts::strict_json_exact(
                text.as_bytes(),
                eal_collector_contracts::JsonLimits {
                    max_bytes: binding.max_output_bytes,
                    ..Default::default()
                },
            )
            .map_err(AcquisitionError::invalid)?
        }
    };
    normalization::finish(binding, &decoded, workspace, limits, metadata)
}
