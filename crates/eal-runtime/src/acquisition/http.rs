//! Read-only fixed-authority HTTP acquisition with bounded JSON and credentials.
use super::{AcquisitionError, AcquisitionResult, ToolBinding, normalization};
use eal_core::ExecutionLimits;
use serde_json::{Value, json};
use std::{
    path::Path,
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
    let configured = reqwest::Url::parse(settings.url.as_deref().unwrap())
        .map_err(|_| AcquisitionError::invalid("Invalid HTTP URL"))?;
    let origin = configured.origin().ascii_serialization();
    let mut client = eal_adapters::http::JsonClient::new(
        &origin,
        &settings.headers_env,
        settings.token_env.as_deref(),
        Instant::now(),
        Duration::from_secs_f64(binding.timeout_seconds),
        binding.max_output_bytes,
    )
    .map_err(|e| AcquisitionError::invalid(e.to_string()))?;
    let mut url = configured;
    {
        let mut query = url.query_pairs_mut();
        for (name, value) in arguments.as_object().unwrap() {
            let value = match value {
                Value::String(s) => s.clone(),
                Value::Bool(v) => v.to_string(),
                Value::Number(v) => v.to_string(),
                _ => unreachable!("normalization admitted scalar query values"),
            };
            query.append_pair(name, &value);
        }
    }
    // A dedicated owner permits callers both inside and outside a Tokio runtime.
    let decoded = std::thread::scope(|scope| {
        scope
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|_| AcquisitionError::invalid("Cannot initialise HTTP execution"))?;
                let value = runtime
                    .block_on(client.request(reqwest::Method::GET, url, None))
                    .map_err(|e| AcquisitionError::invalid(e.to_string()))?;
                Ok::<_, AcquisitionError>(value)
            })
            .join()
            .map_err(|_| AcquisitionError::invalid("HTTP owner failed"))?
    })?;
    normalization::finish(
        binding,
        &decoded,
        workspace,
        limits,
        json!({"transport":"http_get","process_environment_digest":binding.process_environment_digest_with_limits(secret, None, limits)?}),
    )
}
