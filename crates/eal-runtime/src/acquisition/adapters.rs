//! Select the concrete adapter; transport policies remain in cohesive modules.
use super::{AcquisitionError, AcquisitionResult, ToolBinding, ToolKind, registry};
use crate::{lease::AcquisitionLease, process::Supervisor};
use eal_core::ExecutionLimits;
use serde_json::Value;
use std::path::Path;

pub fn acquire(
    binding: &ToolBinding,
    request: &Value,
    workspace: &Path,
    secret: &[u8],
    supervisor: &Supervisor,
    lease: Option<&AcquisitionLease>,
) -> Result<AcquisitionResult, AcquisitionError> {
    acquire_with_limits(
        binding,
        request,
        workspace,
        secret,
        supervisor,
        lease,
        &ExecutionLimits::default(),
    )
}
#[allow(clippy::too_many_arguments)]
pub fn acquire_with_limits(
    binding: &ToolBinding,
    request: &Value,
    workspace: &Path,
    secret: &[u8],
    supervisor: &Supervisor,
    lease: Option<&AcquisitionLease>,
    limits: &ExecutionLimits,
) -> Result<AcquisitionResult, AcquisitionError> {
    limits.validate()?;
    registry::validate_binding(binding)?;
    super::encode_request(request, limits)?;
    match binding.kind {
        ToolKind::Command => super::command::acquire(
            binding, request, workspace, secret, supervisor, lease, limits,
        ),
        ToolKind::JsonFile => super::file::acquire(binding, workspace),
        ToolKind::McpStdio | ToolKind::McpHttp => super::mcp::acquire(
            binding, request, workspace, secret, supervisor, lease, limits,
        ),
        ToolKind::HttpGet => super::http::acquire(binding, request, workspace, secret, limits),
    }
}
