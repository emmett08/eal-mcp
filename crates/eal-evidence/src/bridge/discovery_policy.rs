//! Bounded source-first discovery and operator qualification admission.
use super::*;
impl EvidenceBridge {
    pub(super) fn loaded(
        &self,
        deadline: Instant,
        prior: &DiscoveryBudgetUsage,
    ) -> Result<LoadedCatalogues, BridgeError> {
        let mut loaded = lock(&self.inner.catalogues)?;
        if let Some(loaded) = loaded.as_ref() {
            let mut cached = loaded.clone();
            cached.metrics = CallMetrics::default();
            cached.budget_usage = DiscoveryBudgetUsage::default();
            return Ok(cached);
        }
        let start = Instant::now();
        let mut catalogues = self.inner.config.local_catalogues.clone();
        let mut gaps = Vec::new();
        let mut metrics = CallMetrics::default();
        let mut budget_usage = DiscoveryBudgetUsage::default();
        let mut incomplete = false;
        for selected in &self.inner.config.approved_catalogues {
            if prior
                .requests_upper_bound
                .saturating_add(budget_usage.requests_upper_bound)
                >= self.inner.config.discovery.requests as u64
                || prior
                    .bytes_upper_bound
                    .saturating_add(budget_usage.bytes_upper_bound)
                    >= self.inner.config.discovery.bytes as u64
                || Instant::now() >= deadline
            {
                incomplete = true;
                gaps.push(DiscoveryGap {
                    capability_id: None,
                    connection_id: None,
                    kind: "discovery_budget".into(),
                    message: "Approved catalogue discovery budget exhausted".into(),
                });
                break;
            }
            metrics.requests += 1;
            budget_usage.requests_upper_bound += 1;
            let reserved_bytes = self.inner.config.discovery.bytes.saturating_sub(
                prior
                    .bytes_upper_bound
                    .saturating_add(budget_usage.bytes_upper_bound) as usize,
            );
            match discovery::fetch_catalogue(&selected.url, reserved_bytes, deadline) {
                Ok((catalogue, bytes)) => {
                    metrics.response_bytes = metrics.response_bytes.saturating_add(bytes);
                    budget_usage.bytes_upper_bound =
                        budget_usage.bytes_upper_bound.saturating_add(bytes);
                    catalogues.push(catalogue);
                }
                Err(_) => {
                    budget_usage.bytes_upper_bound = budget_usage
                        .bytes_upper_bound
                        .saturating_add(reserved_bytes as u64);
                    incomplete = true;
                    gaps.push(DiscoveryGap {
                        capability_id: None,
                        connection_id: None,
                        kind: "catalogue_unavailable".into(),
                        message: "An approved catalogue could not be admitted within its bounds"
                            .into(),
                    });
                }
            }
        }
        let descriptors = discovery::combine(catalogues).map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Conflicting or invalid selected evidence catalogues",
            )
        })?;
        if descriptors.len() > self.inner.config.discovery.candidates {
            return Err(error(
                BridgeErrorKind::Budget,
                "Selected capability catalogue exceeds the candidate budget",
            ));
        }
        metrics.elapsed_ms = elapsed(start);
        let value = LoadedCatalogues {
            descriptors,
            gaps,
            metrics,
            budget_usage,
            incomplete,
        };
        *loaded = Some(value.clone());
        Ok(value)
    }
    pub(super) fn admit<'a>(
        &self,
        descriptor: &CapabilityDescriptor,
        connection: &'a ConnectionConfig,
        discovered: &ToolDiscovery,
    ) -> Result<&'a crate::config::ToolMapping, BridgeError> {
        descriptor.validate().map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Invalid qualified evidence capability",
            )
        })?;
        if descriptor.semantics.is_none() {
            return Err(error(
                BridgeErrorKind::Configuration,
                "MCP interface lacks qualified measurement semantics",
            ));
        }
        let transport = match connection.transport {
            TransportConfig::Stdio { .. } => TransportKind::McpStdio,
            TransportConfig::Http { .. } => TransportKind::McpHttp,
        };
        let configuration = connection.configuration_identity().map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Invalid evidence connection identity",
            )
        })?;
        if descriptor.provider.transport != transport
            || descriptor.provider.connection.as_deref() != Some(&connection.id)
            || descriptor.provider.authentication_scope != connection.authentication_scope
            || descriptor.provider.configuration_sha256 != configuration
            || descriptor.provider.version != discovered.server_version
        {
            return Err(error(
                BridgeErrorKind::Configuration,
                "Qualified MCP provider identity differs from the configured connection or initialized server version",
            ));
        }
        let name = descriptor.provider.tool.as_deref().ok_or_else(|| {
            error(
                BridgeErrorKind::Configuration,
                "MCP capability omits its selected tool",
            )
        })?;
        let mapping = connection
            .tools
            .iter()
            .find(|mapping| mapping.tool == name)
            .ok_or_else(|| {
                error(
                    BridgeErrorKind::Configuration,
                    "MCP tool lacks operator-owned normalisation configuration",
                )
            })?;
        let tool = discovered
            .tools
            .iter()
            .find(|tool| tool.name == name)
            .ok_or_else(|| {
                error(
                    BridgeErrorKind::Configuration,
                    "Configured MCP tool is unavailable to this connection",
                )
            })?;
        let input = schema_identity(&Value::Object((*tool.input_schema).clone()))
            .map_err(|_| error(BridgeErrorKind::SchemaDrift, "MCP input schema is invalid"))?;
        let output = tool
            .output_schema
            .as_ref()
            .map(|schema| schema_identity(&Value::Object((**schema).clone())))
            .transpose()
            .map_err(|_| error(BridgeErrorKind::SchemaDrift, "MCP output schema is invalid"))?;
        if input != descriptor.input_schema_sha256 || output != mapping.raw_output_schema_sha256 {
            return Err(error(
                BridgeErrorKind::SchemaDrift,
                "MCP tool schemas differ from their qualified identities",
            ));
        }
        let semantics = descriptor.semantics.as_ref().unwrap();
        if semantics
            .identity_dimensions
            .iter()
            .any(|dimension| !mapping.scope_pointers.contains_key(dimension))
            || matches!(mapping.observation_time, ObservationTime::CollectionTime)
                && (semantics.time.original_observation_time || semantics.time.historical_intervals)
        {
            return Err(error(
                BridgeErrorKind::Configuration,
                "Qualified measurement identity or original-time extraction is unconfigured",
            ));
        }
        Ok(mapping)
    }
    /// Discover configured interfaces before fetching only explicitly approved catalogues.
    pub fn discover(&self) -> Result<DiscoveryReport, BridgeError> {
        let start = Instant::now();
        let deadline = start + Duration::from_secs(self.inner.config.discovery.timeout_seconds);
        let mut metrics = CallMetrics::default();
        let mut budget_usage = DiscoveryBudgetUsage::default();
        let mut found_connections = BTreeMap::new();
        let mut gaps = Vec::new();
        let mut incomplete = false;
        for connection in &self.inner.config.connections {
            let requests = self
                .inner
                .config
                .discovery
                .requests
                .saturating_sub(budget_usage.requests_upper_bound as usize);
            let bytes = self
                .inner
                .config
                .discovery
                .bytes
                .saturating_sub(budget_usage.bytes_upper_bound as usize);
            // Reserve one request for initialization when no session exists yet.
            let starts = self
                .inner
                .sessions
                .get(&connection.id)
                .and_then(|cell| cell.try_lock().ok())
                .is_none_or(|session| {
                    session
                        .as_ref()
                        .is_none_or(|session| session.initialization_may_be_needed())
                });
            let pages = requests
                .saturating_sub(usize::from(starts))
                .min(self.inner.config.discovery.pages_per_connection);
            if pages == 0 || bytes == 0 || Instant::now() >= deadline {
                incomplete = true;
                gaps.push(DiscoveryGap {
                    capability_id: None,
                    connection_id: Some(connection.id.clone()),
                    kind: "discovery_budget".into(),
                    message: "Configured capability discovery budget exhausted".into(),
                });
                break;
            }
            let tools = found_connections
                .values()
                .map(|found: &ToolDiscovery| found.tools.len())
                .sum::<usize>();
            let remaining_tools = self.inner.config.discovery.candidates.saturating_sub(tools);
            if remaining_tools == 0 {
                incomplete = true;
                break;
            }
            let limits = ToolListLimits {
                max_tools: remaining_tools,
                max_bytes: bytes,
                max_pages: pages,
            };
            match self.list(connection, limits, deadline) {
                Ok((found, cost)) => {
                    budget_usage.requests_upper_bound = budget_usage
                        .requests_upper_bound
                        .saturating_add(cost.requests);
                    budget_usage.bytes_upper_bound = budget_usage
                        .bytes_upper_bound
                        .saturating_add(cost.response_bytes);
                    budget_usage
                        .pages_by_connection
                        .insert(connection.id.clone(), found.requests);
                    metrics.add(&cost);
                    found_connections.insert(connection.id.clone(), found);
                }
                Err(failure) => {
                    budget_usage.requests_upper_bound = budget_usage
                        .requests_upper_bound
                        .saturating_add(pages as u64 + u64::from(starts));
                    budget_usage.bytes_upper_bound =
                        budget_usage.bytes_upper_bound.saturating_add(bytes as u64);
                    budget_usage
                        .pages_by_connection
                        .insert(connection.id.clone(), pages as u64);
                    incomplete = true;
                    gaps.push(DiscoveryGap {
                        capability_id: None,
                        connection_id: Some(connection.id.clone()),
                        kind: "configuration".into(),
                        message: failure.message,
                    });
                }
            }
        }
        let loaded = self.loaded(deadline, &budget_usage)?;
        metrics.add(&loaded.metrics);
        budget_usage.add(&loaded.budget_usage);
        gaps.extend(loaded.gaps);
        incomplete |= loaded.incomplete;
        let mut interfaces = Vec::new();
        let mut admitted = Vec::new();
        for (connection_id, found) in &found_connections {
            let connection = self.connection(connection_id)?;
            for tool in &found.tools {
                if interfaces.len() >= self.inner.config.discovery.candidates {
                    incomplete = true;
                    break;
                }
                let input = schema_identity(&Value::Object((*tool.input_schema).clone())).map_err(
                    |_| {
                        error(
                            BridgeErrorKind::Protocol,
                            "Discovered MCP input schema is invalid",
                        )
                    },
                )?;
                let output = tool
                    .output_schema
                    .as_ref()
                    .map(|schema| schema_identity(&Value::Object((**schema).clone())))
                    .transpose()
                    .map_err(|_| {
                        error(
                            BridgeErrorKind::Protocol,
                            "Discovered MCP output schema is invalid",
                        )
                    })?;
                interfaces.push(InterfaceCandidate {
                    connection_id: connection.id.clone(),
                    tool: tool.name.to_string(),
                    input_schema_sha256: input,
                    output_schema_sha256: output,
                    qualified: false,
                });
            }
            for descriptor in loaded.descriptors.iter().filter(|descriptor| {
                descriptor.provider.connection.as_deref() == Some(connection_id)
            }) {
                match self.admit(descriptor, connection, found) {
                    Ok(_) => {
                        admitted.push(descriptor.id.clone());
                        if let Some(interface) = interfaces.iter_mut().find(|interface| {
                            interface.connection_id == *connection_id
                                && Some(interface.tool.as_str())
                                    == descriptor.provider.tool.as_deref()
                        }) {
                            interface.qualified = true;
                        }
                    }
                    Err(failure) => gaps.push(DiscoveryGap {
                        capability_id: Some(descriptor.id.clone()),
                        connection_id: Some(connection_id.clone()),
                        kind: format!("{:?}", failure.kind).to_ascii_lowercase(),
                        message: failure.message,
                    }),
                }
            }
        }
        for interface in interfaces.iter().filter(|interface| !interface.qualified) {
            if !loaded.descriptors.iter().any(|descriptor| {
                descriptor.provider.connection.as_deref() == Some(&interface.connection_id)
                    && descriptor.provider.tool.as_deref() == Some(&interface.tool)
            }) {
                gaps.push(DiscoveryGap{capability_id:None,connection_id:Some(interface.connection_id.clone()),kind:"qualification_missing".into(),message:"A configured MCP interface lacks an operator-qualified measurement contract".into()});
            }
        }
        for descriptor in &loaded.descriptors {
            if !self
                .inner
                .sessions
                .contains_key(descriptor.provider.connection.as_deref().unwrap_or(""))
            {
                gaps.push(DiscoveryGap {
                    capability_id: Some(descriptor.id.clone()),
                    connection_id: descriptor.provider.connection.clone(),
                    kind: "connection_missing".into(),
                    message: "The candidate requires a configured execution connection".into(),
                });
            }
        }
        interfaces.sort_by(|a, b| (&a.connection_id, &a.tool).cmp(&(&b.connection_id, &b.tool)));
        admitted.sort();
        admitted.dedup();
        metrics.elapsed_ms = elapsed(start);
        Ok(DiscoveryReport {
            schema: discovery::DISCOVERY_SCHEMA.into(),
            limits: self.inner.config.discovery.clone(),
            catalogue: CapabilityCatalogue {
                schema: CAPABILITY_CATALOGUE_SCHEMA.into(),
                capabilities: loaded.descriptors,
            },
            admitted_capabilities: admitted,
            interfaces,
            gaps,
            incomplete,
            metrics,
            budget_usage,
        })
    }
    pub(super) fn selected_descriptor(
        &self,
        id: &str,
        deadline: Instant,
    ) -> Result<CapabilityDescriptor, BridgeError> {
        // A local qualification needs no registry network request.
        if let Some(descriptor) = self
            .inner
            .config
            .local_catalogues
            .iter()
            .flat_map(|catalogue| &catalogue.capabilities)
            .find(|descriptor| descriptor.id == id)
        {
            return Ok(descriptor.clone());
        }
        self.loaded(deadline, &DiscoveryBudgetUsage::default())?
            .descriptors
            .into_iter()
            .find(|descriptor| descriptor.id == id)
            .ok_or_else(|| {
                error(
                    BridgeErrorKind::Configuration,
                    "Unknown qualified evidence capability",
                )
            })
    }
}
