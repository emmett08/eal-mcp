//! Direct named MCP calls through the shared official-SDK client.
use super::{AcquisitionError, AcquisitionResult, ToolBinding, ToolKind, normalization};
use eal_collector_contracts::ResponseFormat;
use eal_core::ExecutionLimits;
use eal_mcp_client::{
    client::{self, StdioTarget},
    persistent_mcp::{PersistentMcp, ToolInterfaceIdentity, ToolListLimits, ToolSchemaPins},
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
    supervisor: &crate::process::Supervisor,
    lease: Option<&crate::lease::AcquisitionLease>,
    limits: &ExecutionLimits,
) -> Result<AcquisitionResult, AcquisitionError> {
    let arguments = normalization::input(binding, request, workspace)?;
    let settings = binding.direct.as_ref().expect("validated direct settings");
    let (session, mut metadata, deadline) =
        session(binding, workspace, secret, supervisor, lease, limits)?;
    let discovery_limits = ToolListLimits {
        max_tools: 4096,
        max_pages: 128,
        max_bytes: binding.max_output_bytes,
    };
    let tool = settings.tool.as_deref().unwrap();
    let observed = session
        .call_observed(
            tool,
            &arguments,
            settings
                .input_schema_sha256
                .as_ref()
                .map(|input| ToolSchemaPins {
                    input_sha256: input.clone(),
                    output_sha256: settings.output_schema_sha256.clone(),
                }),
            settings.server_identity.clone(),
            discovery_limits,
            deadline,
        )
        .map_err(|error| AcquisitionError::invalid(error.to_string()))?;
    metadata["mcp_interface"] = observed.interface.to_json();
    metadata["discovery_requests"] = json!(observed.discovery_requests);
    metadata["discovery_bytes"] = json!(observed.discovery_bytes);
    session.close();
    let admitted = (|| {
        if let Some(error) = observed.validation_error {
            return Err(AcquisitionError::invalid(error));
        }
        let result = observed.result;
        if result.is_error.unwrap_or(false) {
            return Err(AcquisitionError::invalid(
                "MCP tool reported acquisition failure",
            ));
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
        normalization::finish(binding, &decoded, workspace, limits, metadata.clone())
    })();
    match admitted {
        Ok(result) => Ok(result),
        Err(error) => Ok(AcquisitionResult {
            metadata,
            error: Some(error),
            ..Default::default()
        }),
    }
}

/// Inspect only the named interface. This never invokes the external operation.
pub(crate) fn inspect(
    binding: &ToolBinding,
    workspace: &Path,
    secret: &[u8],
    supervisor: &crate::process::Supervisor,
    lease: Option<&crate::lease::AcquisitionLease>,
    limits: &ExecutionLimits,
) -> Result<Value, AcquisitionError> {
    let settings = binding
        .direct
        .as_ref()
        .ok_or("MCP binding has no direct settings")?;
    let (session, metadata, deadline) =
        session(binding, workspace, secret, supervisor, lease, limits)?;
    let discovery = session
        .discover_bounded(
            ToolListLimits {
                max_tools: 4096,
                max_pages: 128,
                max_bytes: binding.max_output_bytes,
            },
            deadline,
        )
        .map_err(|error| AcquisitionError::invalid(error.to_string()))?;
    let identity = discovery
        .interface(settings.tool.as_deref().ok_or("MCP binding has no tool")?)
        .map_err(|error| AcquisitionError::invalid(error.to_string()))?;
    check_prepared_identity(settings, &identity)?;
    session.close();
    Ok(
        json!({"interface":identity.to_json(), "discovery_requests":discovery.requests,
        "initialization_requests":discovery.initialization_requests,
        "discovery_bytes":discovery.response_bytes,
        "process_environment_digest":metadata["process_environment_digest"], "operation_calls":0}),
    )
}

fn check_prepared_identity(
    settings: &eal_collector_contracts::DirectBinding,
    identity: &ToolInterfaceIdentity,
) -> Result<(), AcquisitionError> {
    let pins = settings
        .input_schema_sha256
        .as_ref()
        .map(|input| ToolSchemaPins {
            input_sha256: input.clone(),
            output_sha256: settings.output_schema_sha256.clone(),
        });
    identity
        .check(pins.as_ref(), settings.server_identity.as_ref())
        .map_err(|error| AcquisitionError::invalid(error.to_string()))
}

fn session(
    binding: &ToolBinding,
    workspace: &Path,
    secret: &[u8],
    supervisor: &crate::process::Supervisor,
    lease: Option<&crate::lease::AcquisitionLease>,
    limits: &ExecutionLimits,
) -> Result<(PersistentMcp, Value, Instant), AcquisitionError> {
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
            client::supervised_stdio(
                StdioTarget {
                    command: PathBuf::from(&argv[0]),
                    arguments: argv[1..].to_vec(),
                    environment,
                    cwd: Some(binding.package_root(workspace).to_owned()),
                },
                supervisor,
                lease.map(|lease| lease as &dyn eal_process::Retention),
                super::MAX_REQUEST_BYTES * 2 + 65_536,
                binding
                    .max_output_bytes
                    .saturating_mul(2)
                    .saturating_add(65_536),
                duration,
            )
            .map_err(AcquisitionError::invalid)?
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
    Ok((session, metadata, deadline))
}
