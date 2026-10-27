//! Pin actual source identity before reuse and execute exact admitted requests.
use super::*;
impl EvidenceBridge {
    fn validate_arguments(
        &self,
        descriptor: &CapabilityDescriptor,
        arguments: &Value,
        scope: &Value,
    ) -> Result<(), BridgeError> {
        if !arguments.is_object() || !scope.is_object() {
            return Err(error(
                BridgeErrorKind::Configuration,
                "Evidence arguments and scope must be objects",
            ));
        }
        eal_mcp_client::schema::validate_schema(arguments, &descriptor.input_schema).map_err(
            |_| {
                error(
                    BridgeErrorKind::Configuration,
                    "Evidence arguments differ from the selected capability schema",
                )
            },
        )?;
        eal_collector_contracts::capability::validate_scope_input(
            descriptor,
            &scope
                .as_object()
                .unwrap()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            arguments,
        )
        .map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Evidence source scope differs from the actual arguments",
            )
        })?;
        Ok(())
    }
    /// Bind from actual operator configuration and fresh authenticated discovery before reuse.
    pub fn bind_acquisition(
        &self,
        capability_id: &str,
        arguments: Value,
        scope: Value,
    ) -> Result<BridgeBinding, BridgeError> {
        self.bind_acquisition_until(
            capability_id,
            arguments,
            scope,
            Instant::now() + Duration::from_secs(self.inner.config.discovery.timeout_seconds),
        )
    }
    /// Resolve dynamic selectors from qualified request bindings while retaining every
    /// caller-required selector. This permits an inventory result to supply a later ID.
    pub fn bind_acquisition_derived_scope_until(
        &self,
        capability_id: &str,
        arguments: Value,
        required_scope: Value,
        deadline: Instant,
    ) -> Result<BridgeBinding, BridgeError> {
        let required = required_scope.as_object().ok_or_else(|| {
            error(
                BridgeErrorKind::Configuration,
                "Required evidence scope must be an object",
            )
        })?;
        let descriptor = self.selected_descriptor(capability_id, deadline)?;
        let actual = eal_collector_contracts::capability::scope_from_input(&descriptor, &arguments)
            .map_err(|_| {
                error(
                    BridgeErrorKind::Configuration,
                    "Evidence scope cannot be derived from the resolved arguments",
                )
            })?;
        if required
            .iter()
            .any(|(key, value)| value.is_null() || actual.get(key) != Some(value))
        {
            return Err(error(
                BridgeErrorKind::Configuration,
                "Resolved evidence source differs from a required scope selector",
            ));
        }
        self.bind_acquisition_until(capability_id, arguments, json!(actual), deadline)
    }
    pub fn bind_acquisition_until(
        &self,
        capability_id: &str,
        arguments: Value,
        scope: Value,
        deadline: Instant,
    ) -> Result<BridgeBinding, BridgeError> {
        let descriptor = self.selected_descriptor(capability_id, deadline)?;
        self.validate_arguments(&descriptor, &arguments, &scope)?;
        let connection =
            self.connection(descriptor.provider.connection.as_deref().ok_or_else(|| {
                error(
                    BridgeErrorKind::Configuration,
                    "MCP capability omits its connection",
                )
            })?)?;
        let (found, admission_metrics) = self.list(connection, self.list_limits(), deadline)?;
        self.admit(&descriptor, connection, &found)?;
        let configuration_sha256 = connection.configuration_identity().map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Cannot identify source configuration",
            )
        })?;
        let qualified_contract_sha256 = descriptor.measurement_identity().map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Cannot identify capability qualification",
            )
        })?;
        let authentication_scope_sha256 = canonical_identity(&(
            &connection.authentication_scope,
            self.environment(connection)?,
        ))
        .map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Cannot identify authenticated source scope",
            )
        })?;
        let identity = BridgeIdentity {
            provider_id: descriptor.provider.id.clone(),
            capability_id: descriptor.id.clone(),
            capability_version: descriptor.version.clone(),
            input_schema_sha256: descriptor.input_schema_sha256.clone(),
            output_schema_sha256: descriptor.output_schema_sha256.clone(),
            configuration_sha256,
            authentication_scope_sha256,
            qualified_contract_sha256,
            scope,
        };
        Ok(BridgeBinding {
            identity,
            arguments,
            descriptor,
            admission_metrics,
        })
    }
    pub fn execute_bound(
        &self,
        binding: &BridgeBinding,
        deadline: Instant,
    ) -> Result<BridgeObservation, BridgeError> {
        let start = Instant::now();
        let descriptor = self.selected_descriptor(&binding.identity.capability_id, deadline)?;
        if canonical_identity(&descriptor).ok() != canonical_identity(&binding.descriptor).ok()
            || binding.identity.provider_id != descriptor.provider.id
            || binding.identity.capability_version != descriptor.version
            || binding.identity.input_schema_sha256 != descriptor.input_schema_sha256
            || binding.identity.output_schema_sha256 != descriptor.output_schema_sha256
        {
            return Err(error(
                BridgeErrorKind::Configuration,
                "Evidence binding differs from operator-qualified capability",
            ));
        }
        self.validate_arguments(&descriptor, &binding.arguments, &binding.identity.scope)?;
        let connection =
            self.connection(descriptor.provider.connection.as_deref().ok_or_else(|| {
                error(
                    BridgeErrorKind::Configuration,
                    "MCP capability omits its connection",
                )
            })?)?;
        // Public binding values are untrusted: recompute private source fingerprints.
        let config = connection.configuration_identity().map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Cannot identify source configuration",
            )
        })?;
        let qualification = descriptor.measurement_identity().map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Cannot identify capability qualification",
            )
        })?;
        let auth = canonical_identity(&(
            &connection.authentication_scope,
            self.environment(connection)?,
        ))
        .map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "Cannot identify authenticated source scope",
            )
        })?;
        if config != binding.identity.configuration_sha256
            || auth != binding.identity.authentication_scope_sha256
            || qualification != binding.identity.qualified_contract_sha256
        {
            return Err(error(
                BridgeErrorKind::Configuration,
                "Evidence binding configuration or authentication changed",
            ));
        }
        let mapping = connection
            .tools
            .iter()
            .find(|mapping| {
                Some(mapping.tool.as_str()) == binding.descriptor.provider.tool.as_deref()
            })
            .ok_or_else(|| {
                error(
                    BridgeErrorKind::Configuration,
                    "MCP tool normalisation is unavailable",
                )
            })?;
        let session = self.session(connection)?;
        let deadline = deadline
            .min(Instant::now() + Duration::from_secs(binding.descriptor.limits.timeout_seconds));
        let _permit = self.admit_execution(&descriptor, deadline)?;
        let result = session
            .call_pinned(
                &mapping.tool,
                &binding.arguments,
                &ToolSchemaPins {
                    input_sha256: binding.descriptor.input_schema_sha256.clone(),
                    output_sha256: mapping.raw_output_schema_sha256.clone(),
                },
                self.list_limits(),
                deadline,
            )
            .map_err(protocol_error)?;
        if serde_json::to_vec(&result.result)
            .map_err(|_| error(BridgeErrorKind::Protocol, "Cannot encode MCP result"))?
            .len()
            > connection.output_bytes
        {
            return Err(error(
                BridgeErrorKind::Budget,
                "MCP result exceeded the connection output budget",
            ));
        }
        let normal = normalization::normalise(
            &result.result,
            mapping,
            &binding.descriptor,
            &binding.identity.scope,
        )?;
        let request_bytes = serde_json::to_vec(&binding.arguments)
            .map_err(|_| error(BridgeErrorKind::Protocol, "Cannot encode MCP arguments"))?
            .len() as u64;
        let mut metrics = binding.admission_metrics.clone();
        metrics.add(&CallMetrics {
            requests: result.discovery_requests.saturating_add(1),
            request_bytes,
            response_bytes: result.discovery_bytes.saturating_add(normal.bytes),
            elapsed_ms: elapsed(start),
            server_starts: 0,
        });
        let source_identity = canonical_identity(&(&binding.identity, &binding.arguments))
            .map_err(|_| {
                error(
                    BridgeErrorKind::Protocol,
                    "Cannot identify MCP source observation",
                )
            })?;
        let mut provenance = binding.descriptor.qualified_provenance().map_err(|_| {
            error(
                BridgeErrorKind::Configuration,
                "cannot identify qualified source provenance",
            )
        })?;
        provenance.as_object_mut().expect("provenance object").extend(json!({"configuration_sha256":config,"authentication_scope_sha256":auth,"metric_scope":"mcp_logical_json_payloads"}).as_object().expect("transport provenance object").clone());
        Ok(BridgeObservation {
            value: normal.value,
            observed_at: normal.observed_at,
            source_identity,
            coverage: normal.coverage,
            metrics,
            provenance,
        })
    }
}
