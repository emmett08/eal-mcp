//! Bounded native measurements resolved from immutable operator configuration.
use super::{CommandProtocol, NativeProviderConfig, NativeTransport};
use crate::{
    execution::{
        ResolvedAcquisition, SourceError, SourceErrorKind, SourceExecutor, SourceFuture,
        SourceMetrics, SourceObservation, measured_coverage,
    },
    normalization,
    plan::{AcquisitionStep, SourceIdentity},
};
use eal_collector_contracts::{
    AcquisitionRequest, JsonLimits,
    capability::{canonical_identity, scope_from_input, validate_historical_input},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, VecDeque},
    fs::{File, OpenOptions},
    io::Read,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct NativeSources {
    providers: Arc<BTreeMap<String, NativeProviderConfig>>,
    environment: Arc<BTreeMap<String, String>>,
    budgets: Arc<BTreeMap<String, NativeBudget>>,
}
struct NativeBudget {
    slots: tokio::sync::Semaphore,
    requests: tokio::sync::Mutex<VecDeque<Instant>>,
}
fn failure(kind: SourceErrorKind, message: impl Into<String>) -> SourceError {
    SourceError {
        kind,
        message: message.into(),
    }
}
fn configuration(message: impl Into<String>) -> SourceError {
    failure(SourceErrorKind::Configuration, message)
}
fn protocol(message: impl Into<String>) -> SourceError {
    failure(SourceErrorKind::Protocol, message)
}

fn open_regular(path: &Path) -> Result<File, SourceError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(
            (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32,
        );
    }
    let file = options
        .open(path)
        .map_err(|_| configuration("Native source file is unavailable or is a symbolic link"))?;
    if !file
        .metadata()
        .map_err(|_| configuration("Cannot inspect native source file"))?
        .is_file()
    {
        return Err(configuration("Native source path must be a regular file"));
    }
    Ok(file)
}
fn file_digest(path: &Path, maximum: u64) -> Result<String, SourceError> {
    let mut file = open_regular(path)?;
    if file
        .metadata()
        .map_err(|_| configuration("Cannot inspect native source file"))?
        .len()
        > maximum
    {
        return Err(configuration(
            "Native source file exceeds fingerprint allowance",
        ));
    }
    let mut hash = Sha256::new();
    let mut block = [0u8; 65536];
    let mut total = 0u64;
    loop {
        let count = file
            .read(&mut block)
            .map_err(|_| configuration("Cannot fingerprint native source file"))?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        if total > maximum {
            return Err(configuration(
                "Native source file exceeds fingerprint allowance",
            ));
        }
        hash.update(&block[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

impl NativeSources {
    pub fn new(
        configurations: Vec<NativeProviderConfig>,
        environment: BTreeMap<String, String>,
    ) -> Result<Self, String> {
        if configurations.len() > 128 {
            return Err("Too many configured native providers".into());
        }
        let mut providers = BTreeMap::new();
        for configuration in configurations {
            configuration.validate()?;
            if providers
                .insert(configuration.capability.id.clone(), configuration)
                .is_some()
            {
                return Err("Native provider repeats a capability identity".into());
            }
        }
        Ok(Self {
            budgets: Arc::new(
                providers
                    .iter()
                    .map(|(id, config)| {
                        (
                            id.clone(),
                            NativeBudget {
                                slots: tokio::sync::Semaphore::new(
                                    config.capability.limits.concurrency as usize,
                                ),
                                requests: tokio::sync::Mutex::new(VecDeque::new()),
                            },
                        )
                    })
                    .collect(),
            ),
            providers: Arc::new(providers),
            environment: Arc::new(environment),
        })
    }
    pub fn contains(&self, id: &str) -> bool {
        self.providers.contains_key(id)
    }
    pub fn configurations(&self) -> impl Iterator<Item = &NativeProviderConfig> {
        self.providers.values()
    }
    fn selected(&self, id: &str) -> Result<&NativeProviderConfig, SourceError> {
        self.providers
            .get(id)
            .ok_or_else(|| configuration("Native capability has no configured execution authority"))
    }
    fn environment(
        &self,
        config: &NativeProviderConfig,
    ) -> Result<BTreeMap<String, String>, SourceError> {
        config
            .environment_names()
            .into_iter()
            .map(|name| {
                let value = self
                    .environment
                    .get(name)
                    .filter(|value| {
                        !value.is_empty() && value.len() <= 65536 && !value.contains('\0')
                    })
                    .ok_or_else(|| {
                        configuration(
                            "Native provider requires an unavailable environment reference",
                        )
                    })?;
                Ok((name.to_owned(), value.clone()))
            })
            .collect()
    }
    /// Validate the configured implementation without acquiring evidence.
    pub fn admit(&self, id: &str) -> Result<(), SourceError> {
        let config = self.selected(id)?;
        config
            .capability
            .measurement_identity()
            .map_err(configuration)?;
        self.fingerprints(config)?;
        Ok(())
    }
    fn fingerprints(&self, config: &NativeProviderConfig) -> Result<(String, String), SourceError> {
        let mut files = Vec::new();
        if let NativeTransport::Command {
            program,
            supervisor,
            ..
        } = &config.transport
        {
            files.push((program.clone(), file_digest(program, 256 * 1024 * 1024)?));
            files.push((
                supervisor.clone(),
                file_digest(supervisor, 256 * 1024 * 1024)?,
            ));
        }
        if let NativeTransport::RetainedArtifact { path, sha256 } = &config.transport {
            let actual = file_digest(path, config.capability.limits.output_bytes)?;
            if &actual != sha256 {
                return Err(failure(
                    SourceErrorKind::SchemaDrift,
                    "Retained artifact content differs from its approved hash",
                ));
            }
            files.push((path.clone(), actual));
        }
        for pin in &config.pinned_files {
            let actual = file_digest(&pin.path, 256 * 1024 * 1024)?;
            if actual != pin.sha256 {
                return Err(failure(
                    SourceErrorKind::SchemaDrift,
                    "Native source package/configuration file changed after approval",
                ));
            }
            files.push((pin.path.clone(), actual));
        }
        let configuration_sha256 = canonical_identity(&(
            config.configuration_identity().map_err(configuration)?,
            files,
        ))
        .map_err(configuration)?;
        let authentication_scope_sha256 = canonical_identity(&(
            &config.capability.provider.authentication_scope,
            self.environment(config)?,
        ))
        .map_err(configuration)?;
        Ok((configuration_sha256, authentication_scope_sha256))
    }
    fn bind_sync(
        &self,
        step: &AcquisitionStep,
        arguments: Value,
    ) -> Result<ResolvedAcquisition, SourceError> {
        let config = self.selected(&step.source.capability_id)?;
        config.validate().map_err(configuration)?;
        let descriptor = &config.capability;
        let qualification = descriptor.measurement_identity().map_err(configuration)?;
        step.source
            .validate_capability(descriptor)
            .map_err(|message| failure(SourceErrorKind::SchemaDrift, message))?;
        let required_scope = step
            .source
            .scope
            .as_object()
            .ok_or_else(|| configuration("Native source scope requires an object"))?;
        let scope = scope_from_input(descriptor, &arguments).map_err(configuration)?;
        if required_scope
            .iter()
            .any(|(dimension, value)| scope.get(dimension) != Some(value))
        {
            return Err(configuration(
                "Resolved native input scope differs from required subject scope",
            ));
        }
        if let Some(interval) = &step.source.historical_interval {
            validate_historical_input(descriptor, &arguments, &interval.start, &interval.end)
                .map_err(configuration)?;
        }
        let (configuration_sha256, authentication_scope_sha256) = self.fingerprints(config)?;
        Ok(ResolvedAcquisition {
            source: SourceIdentity {
                provider_id: descriptor.provider.id.clone(),
                capability_id: descriptor.id.clone(),
                capability_version: descriptor.version.clone(),
                input_schema_sha256: descriptor.input_schema_sha256.clone(),
                output_schema_sha256: descriptor.output_schema_sha256.clone(),
                qualified_contract_sha256: qualification,
                configuration_sha256,
                authentication_scope_sha256,
                scope: json!(scope),
                historical_interval: step.source.historical_interval.clone(),
            },
            arguments,
            admission_metrics: SourceMetrics::default(),
        })
    }
    async fn acquire(
        &self,
        request: &ResolvedAcquisition,
        deadline: Instant,
    ) -> Result<SourceObservation, SourceError> {
        let config = self.selected(&request.source.capability_id)?.clone();
        let budget = self
            .budgets
            .get(&request.source.capability_id)
            .ok_or_else(|| configuration("Native provider budget is unavailable"))?;
        let _permit = tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            budget.slots.acquire(),
        )
        .await
        .map_err(|_| {
            failure(
                SourceErrorKind::Budget,
                "Native source concurrency deadline exhausted",
            )
        })?
        .map_err(|_| configuration("Native provider scheduler is closed"))?;
        if let Some(maximum) = config.capability.limits.requests_per_minute {
            loop {
                let mut requests = budget.requests.lock().await;
                let now = Instant::now();
                while requests
                    .front()
                    .is_some_and(|at| now.duration_since(*at) >= Duration::from_secs(60))
                {
                    requests.pop_front();
                }
                if (requests.len() as u64) < maximum {
                    requests.push_back(now);
                    break;
                }
                let ready = *requests
                    .front()
                    .ok_or_else(|| configuration("Native rate-limit state is unavailable"))?
                    + Duration::from_secs(60);
                drop(requests);
                tokio::time::timeout_at(
                    tokio::time::Instant::from_std(deadline),
                    tokio::time::sleep_until(tokio::time::Instant::from_std(ready)),
                )
                .await
                .map_err(|_| {
                    failure(
                        SourceErrorKind::Budget,
                        "Native provider request rate would exceed the execution deadline",
                    )
                })?;
            }
        }
        let current = self.bind_sync(
            &AcquisitionStep {
                id: "native_rebind".into(),
                source: request.source.clone(),
                arguments: request.arguments.clone(),
                bindings: vec![],
                for_each: None,
                availability: Default::default(),
                reuse: Default::default(),
                max_age_seconds: None,
                coverage_proof: None,
            },
            request.arguments.clone(),
        )?;
        if current.source != request.source {
            return Err(failure(
                SourceErrorKind::SchemaDrift,
                "Native configuration or authentication changed after source binding",
            ));
        }
        let remaining =
            deadline
                .saturating_duration_since(Instant::now())
                .min(Duration::from_secs(
                    config.capability.limits.timeout_seconds,
                ));
        if remaining.is_zero() {
            return Err(failure(
                SourceErrorKind::Budget,
                "Native source deadline exhausted",
            ));
        }
        let environment = self.environment(&config)?;
        let secret_names =
            config
                .credential_env
                .iter()
                .map(String::as_str)
                .chain(match &config.transport {
                    NativeTransport::HttpGet { token_env, .. } => token_env.as_deref(),
                    _ => None,
                });
        let guard = eal_adapters::http::CredentialGuard::captured(
            secret_names.filter_map(|name| environment.get(name).cloned()),
        );
        let mut expected_request = None;
        let (bytes, request_bytes, transport_requests) = match &config.transport {
            NativeTransport::Command {
                program,
                supervisor,
                arguments,
                cwd,
                protocol,
                ..
            } => {
                let input = match protocol {
                    CommandProtocol::RawJson => request.arguments.clone(),
                    CommandProtocol::EalAcquisition {
                        environment,
                        tool,
                        tool_version,
                        context,
                    } => {
                        let envelope = AcquisitionRequest {
                            evidence_id: request.source_id().map_err(configuration)?,
                            environment: environment.clone(),
                            tool: tool.clone(),
                            tool_version: tool_version.clone(),
                            input: request.arguments.clone(),
                            context: context.clone(),
                        };
                        expected_request = Some(envelope.acquisition());
                        envelope.to_value()
                    }
                };
                let mut input = serde_json::to_vec(&input).map_err(|_| protocol_error())?;
                input.push(b'\n');
                if input.len() > 16 * 1024 * 1024 {
                    return Err(failure(
                        SourceErrorKind::Budget,
                        "Native command input exceeds byte allowance",
                    ));
                }
                let request_bytes = input.len() as u64;
                let mut argv = vec![
                    program
                        .to_str()
                        .ok_or_else(|| configuration("Native executable path is not UTF-8"))?
                        .to_owned(),
                ];
                argv.extend(arguments.clone());
                let outcome = eal_adapters::command::run_with(
                    eal_process::Supervisor::new(supervisor.clone()),
                    eal_process::CommandRequest {
                        argv,
                        environment,
                        cwd: cwd.clone(),
                        input,
                        timeout: remaining,
                        max_output_bytes: config.capability.limits.output_bytes as usize,
                    },
                )
                .await
                .map_err(|_| {
                    SourceError::acquisition("Native supervised command could not complete")
                })?;
                if outcome.returncode != 0 || outcome.execution_error.is_some() || outcome.timed_out
                {
                    return Err(SourceError::acquisition(
                        "Native supervised command failed or exceeded its limits",
                    ));
                }
                (outcome.stdout, request_bytes, 1)
            }
            NativeTransport::HttpGet {
                url,
                query_bindings,
                token_env,
            } => {
                let mut url = reqwest::Url::parse(url)
                    .map_err(|_| configuration("Native HTTP endpoint is invalid"))?;
                for (name, pointer) in query_bindings {
                    let value = request
                        .arguments
                        .pointer(pointer)
                        .ok_or_else(|| configuration("Native HTTP query argument is unresolved"))?;
                    let value = match value {
                        Value::String(value) => value.clone(),
                        Value::Number(_) | Value::Bool(_) => value.to_string(),
                        _ => return Err(configuration("Native HTTP query values must be scalar")),
                    };
                    url.query_pairs_mut().append_pair(name, &value);
                }
                let client = reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .no_proxy()
                    .timeout(remaining)
                    .build()
                    .map_err(|_| configuration("Cannot configure native HTTP client"))?;
                let mut builder = client
                    .get(url.clone())
                    .header(reqwest::header::ACCEPT, "application/json");
                if let Some(name) = token_env {
                    builder =
                        builder.bearer_auth(environment.get(name).ok_or_else(|| {
                            configuration("Native HTTP credential is unavailable")
                        })?);
                }
                let mut response = builder.send().await.map_err(|_| {
                    SourceError::acquisition("Native HTTP request failed or reached its deadline")
                })?;
                if !response.status().is_success() {
                    return Err(SourceError::acquisition(
                        "Native HTTP endpoint returned a failed status",
                    ));
                }
                let maximum = config.capability.limits.output_bytes as usize;
                if response
                    .content_length()
                    .is_some_and(|size| size > maximum as u64)
                {
                    return Err(failure(
                        SourceErrorKind::Budget,
                        "Native HTTP result exceeds output allowance",
                    ));
                }
                let mut bytes = Vec::new();
                while let Some(chunk) = response
                    .chunk()
                    .await
                    .map_err(|_| SourceError::acquisition("Native HTTP transfer failed"))?
                {
                    if chunk.len() > maximum.saturating_sub(bytes.len()) {
                        return Err(failure(
                            SourceErrorKind::Budget,
                            "Native HTTP result exceeds output allowance",
                        ));
                    }
                    bytes.extend_from_slice(&chunk);
                }
                (bytes, url.as_str().len() as u64, 1)
            }
            NativeTransport::RetainedArtifact { path, sha256 } => {
                let mut bytes = Vec::new();
                open_regular(path)?
                    .take(config.capability.limits.output_bytes + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|_| SourceError::acquisition("Retained artifact could not be read"))?;
                if bytes.len() > config.capability.limits.output_bytes as usize {
                    return Err(failure(
                        SourceErrorKind::Budget,
                        "Retained artifact exceeds output allowance",
                    ));
                }
                if format!("{:x}", Sha256::digest(&bytes)) != *sha256 {
                    return Err(failure(
                        SourceErrorKind::SchemaDrift,
                        "Retained artifact changed after binding",
                    ));
                }
                (bytes, 0, 0)
            }
        };
        if Instant::now() >= deadline {
            return Err(failure(
                SourceErrorKind::Budget,
                "Native source deadline exhausted",
            ));
        }
        let decoded = eal_collector_contracts::strict_json_exact(
            &bytes,
            JsonLimits {
                max_bytes: config.capability.limits.output_bytes as usize,
                ..Default::default()
            },
        )
        .map_err(|_| protocol("Native source did not return exact bounded JSON"))?;
        guard
            .inspect(&decoded)
            .map_err(|_| protocol("Native source output includes a selected credential"))?;
        if expected_request
            .as_ref()
            .is_some_and(|expected| decoded.get("request") != Some(expected))
        {
            return Err(protocol(
                "Native EAL response does not correspond to its exact acquisition request",
            ));
        }
        let normalised = normalization::normalise_decoded(
            &decoded,
            &config.mapping,
            &config.capability,
            &request.source.scope,
            bytes.len() as u64,
        )
        .map_err(|error| protocol(error.message))?;
        let coverage = measured_coverage(&normalised.value, &normalised.coverage);
        let mut provenance = config
            .capability
            .qualified_provenance()
            .map_err(configuration)?;
        provenance.as_object_mut().expect("provenance object").extend(json!({"configuration_sha256":request.source.configuration_sha256,"authentication_scope_sha256":request.source.authentication_scope_sha256,"transport":"native","metric_scope":"native_payload_bytes"}).as_object().expect("transport provenance object").clone());
        Ok(SourceObservation {
            value: normalised.value,
            observed_at: normalised.observed_at,
            coverage,
            provenance,
            metrics: SourceMetrics {
                transport_requests: Some(transport_requests),
                request_bytes: Some(request_bytes),
                response_bytes: Some(bytes.len() as u64),
            },
        })
    }
}
fn protocol_error() -> SourceError {
    protocol("Cannot encode native acquisition input")
}
impl SourceExecutor for NativeSources {
    fn bind<'a>(
        &'a self,
        step: &'a AcquisitionStep,
        arguments: Value,
    ) -> SourceFuture<'a, ResolvedAcquisition> {
        let executor = self.clone();
        let step = step.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || executor.bind_sync(&step, arguments))
                .await
                .map_err(|_| configuration("Native source binding worker failed"))?
        })
    }
    fn execute<'a>(
        &'a self,
        request: &'a ResolvedAcquisition,
        deadline: Instant,
    ) -> SourceFuture<'a, SourceObservation> {
        Box::pin(self.acquire(request, deadline))
    }
}
