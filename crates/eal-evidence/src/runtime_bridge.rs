//! MCP capability binding for the source-independent shared execution runtime.
use crate::{
    bridge::{BridgeBinding, BridgeError, BridgeErrorKind, EvidenceBridge},
    execution::{
        ResolvedAcquisition, SourceError, SourceErrorKind, SourceExecutor, SourceFuture,
        SourceMetrics, SourceObservation, measured_coverage,
    },
    plan::{AcquisitionStep, SourceIdentity},
};
use serde_json::Value;
use std::time::Instant;
fn failure(error: BridgeError) -> SourceError {
    SourceError {
        kind: match error.kind {
            BridgeErrorKind::Configuration => SourceErrorKind::Configuration,
            BridgeErrorKind::SchemaDrift => SourceErrorKind::SchemaDrift,
            BridgeErrorKind::Acquisition => SourceErrorKind::Acquisition,
            BridgeErrorKind::Budget => SourceErrorKind::Budget,
            BridgeErrorKind::Protocol => SourceErrorKind::Protocol,
        },
        message: error.message,
    }
}
fn source(
    binding: &BridgeBinding,
    requested: &SourceIdentity,
) -> Result<SourceIdentity, SourceError> {
    let identity = &binding.identity;
    if let Some(interval) = &requested.historical_interval {
        eal_collector_contracts::capability::validate_historical_input(
            &binding.descriptor,
            &binding.arguments,
            &interval.start,
            &interval.end,
        )
        .map_err(|_| SourceError {
            kind: SourceErrorKind::HistoricalUnavailable,
            message: "historical interval is unbound or differs from actual source arguments"
                .into(),
        })?;
    }
    requested
        .validate_capability(&binding.descriptor)
        .map_err(|message| SourceError {
            kind: SourceErrorKind::SchemaDrift,
            message,
        })?;
    Ok(SourceIdentity {
        provider_id: identity.provider_id.clone(),
        capability_id: identity.capability_id.clone(),
        capability_version: identity.capability_version.clone(),
        input_schema_sha256: identity.input_schema_sha256.clone(),
        output_schema_sha256: identity.output_schema_sha256.clone(),
        qualified_contract_sha256: identity.qualified_contract_sha256.clone(),
        configuration_sha256: identity.configuration_sha256.clone(),
        authentication_scope_sha256: identity.authentication_scope_sha256.clone(),
        scope: identity.scope.clone(),
        historical_interval: requested.historical_interval.clone(),
    })
}
impl SourceExecutor for EvidenceBridge {
    fn bind<'a>(
        &'a self,
        step: &'a AcquisitionStep,
        arguments: Value,
    ) -> SourceFuture<'a, ResolvedAcquisition> {
        self.bind_until(
            step,
            arguments,
            Instant::now()
                + std::time::Duration::from_secs(self.config().discovery.timeout_seconds),
        )
    }
    fn bind_until<'a>(
        &'a self,
        step: &'a AcquisitionStep,
        arguments: Value,
        deadline: Instant,
    ) -> SourceFuture<'a, ResolvedAcquisition> {
        let bridge = self.clone();
        let requested = step.source.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let binding = bridge
                    .bind_acquisition_derived_scope_until(
                        &requested.capability_id,
                        arguments,
                        requested.scope.clone(),
                        deadline,
                    )
                    .map_err(failure)?;
                Ok(ResolvedAcquisition {
                    source: source(&binding, &requested)?,
                    arguments: binding.arguments,
                    admission_metrics: SourceMetrics {
                        transport_requests: Some(binding.admission_metrics.requests),
                        request_bytes: Some(binding.admission_metrics.request_bytes),
                        response_bytes: Some(binding.admission_metrics.response_bytes),
                    },
                })
            })
            .await
            .map_err(|_| SourceError::acquisition("MCP source binding worker failed"))?
        })
    }
    fn execute<'a>(
        &'a self,
        request: &'a ResolvedAcquisition,
        deadline: Instant,
    ) -> SourceFuture<'a, SourceObservation> {
        let bridge = self.clone();
        let request = request.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let binding = bridge
                    .bind_acquisition_until(
                        &request.source.capability_id,
                        request.arguments.clone(),
                        request.source.scope.clone(),
                        deadline,
                    )
                    .map_err(failure)?;
                if source(&binding, &request.source)? != request.source {
                    return Err(SourceError {
                        kind: SourceErrorKind::SchemaDrift,
                        message: "source configuration or authentication changed after binding"
                            .into(),
                    });
                }
                let observation = bridge.execute_bound(&binding, deadline).map_err(failure)?;
                // Completeness is accepted only when normalization validated the
                // qualified measured completeness indicator, never from prose.
                let coverage = measured_coverage(&observation.value, &observation.coverage);
                Ok(SourceObservation {
                    value: observation.value,
                    observed_at: observation.observed_at,
                    provenance: observation.provenance,
                    coverage,
                    metrics: SourceMetrics {
                        transport_requests: Some(observation.metrics.requests),
                        request_bytes: Some(observation.metrics.request_bytes),
                        response_bytes: Some(observation.metrics.response_bytes),
                    },
                })
            })
            .await
            .map_err(|_| SourceError::acquisition("MCP source acquisition worker failed"))?
        })
    }
}
