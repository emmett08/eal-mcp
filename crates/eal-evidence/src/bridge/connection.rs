//! One private persistent owner per operator-configured evidence connection.
use super::*;
impl EvidenceBridge {
    pub(super) fn connection(&self, id: &str) -> Result<&ConnectionConfig, BridgeError> {
        self.inner
            .config
            .connections
            .iter()
            .find(|connection| connection.id == id)
            .ok_or_else(|| {
                error(
                    BridgeErrorKind::Configuration,
                    "MCP capability requires an operator-configured connection",
                )
            })
    }
    pub(super) fn environment(
        &self,
        connection: &ConnectionConfig,
    ) -> Result<BTreeMap<String, String>, BridgeError> {
        connection
            .environment_names()
            .into_iter()
            .map(|name| {
                let value = self
                    .inner
                    .environment
                    .get(name)
                    .filter(|value| !value.is_empty() && value.len() <= 16_384)
                    .ok_or_else(|| {
                        error(
                            BridgeErrorKind::Configuration,
                            "A required evidence connection environment reference is unavailable",
                        )
                    })?;
                Ok((name.to_owned(), value.clone()))
            })
            .collect()
    }
    pub(super) fn session(
        &self,
        connection: &ConnectionConfig,
    ) -> Result<Arc<PersistentMcp>, BridgeError> {
        let cell = self.inner.sessions.get(&connection.id).ok_or_else(|| {
            error(
                BridgeErrorKind::Configuration,
                "Unknown evidence connection",
            )
        })?;
        let mut selected = lock(cell)?;
        if let Some(session) = selected.as_ref() {
            return Ok(session.clone());
        }
        let environment = self.environment(connection)?;
        let target = match &connection.transport {
            TransportConfig::Stdio {
                command,
                arguments,
                cwd,
                ..
            } => ClientTarget::Stdio(StdioTarget {
                command: command.clone(),
                arguments: arguments.clone(),
                environment,
                cwd: cwd.clone(),
            }),
            TransportConfig::Http { url, token_env } => {
                let client = reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .timeout(Duration::from_secs(connection.timeout_seconds))
                    .build()
                    .map_err(|_| {
                        error(
                            BridgeErrorKind::Configuration,
                            "Cannot configure evidence HTTP client",
                        )
                    })?;
                eal_mcp::client::create_http_target_with_client(
                    url,
                    token_env.as_deref(),
                    &environment,
                    client,
                )
                .map_err(|_| {
                    error(
                        BridgeErrorKind::Configuration,
                        "Evidence HTTP connection lacks valid transport authentication",
                    )
                })?
            }
        };
        let session = Arc::new(
            PersistentMcp::new(target, Duration::from_secs(connection.timeout_seconds)).map_err(
                |_| {
                    error(
                        BridgeErrorKind::Configuration,
                        "Invalid MCP evidence deadline",
                    )
                },
            )?,
        );
        *selected = Some(session.clone());
        Ok(session)
    }
    pub(super) fn list_limits(&self) -> ToolListLimits {
        ToolListLimits {
            max_tools: self.inner.config.discovery.candidates,
            max_bytes: self.inner.config.discovery.bytes,
            max_pages: self
                .inner
                .config
                .discovery
                .pages_per_connection
                .min(self.inner.config.discovery.requests.saturating_sub(1)),
        }
    }
    pub(super) fn list(
        &self,
        connection: &ConnectionConfig,
        limits: ToolListLimits,
        deadline: Instant,
    ) -> Result<(ToolDiscovery, CallMetrics), BridgeError> {
        let start = Instant::now();
        let session = self.session(connection)?;
        let found = session
            .discover_bounded(limits, deadline)
            .map_err(protocol_error)?;
        let metrics = CallMetrics {
            requests: found.requests.saturating_add(found.initialization_requests),
            request_bytes: 0,
            response_bytes: found.response_bytes,
            elapsed_ms: elapsed(start),
            server_starts: found.initialization_requests,
        };
        Ok((found, metrics))
    }
}
