//! Acquisition policies: dependency bindings, bounded concurrency and atomic source reuse.
use crate::{
    evaluation,
    execution::*,
    plan::*,
    storage::{SourceStore, StoreIoError},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;
pub(crate) struct Concurrency {
    pub(crate) assessment_time: Option<String>,
    pub(crate) acquired: std::sync::Mutex<BTreeSet<String>>,
    pub(crate) global: Arc<Semaphore>,
    pub(crate) providers: BTreeMap<String, Arc<Semaphore>>,
    pub(crate) requests: AtomicUsize,
    pub(crate) maximum_requests: usize,
}
pub(crate) fn parse_time(value: &str) -> Result<chrono::DateTime<chrono::FixedOffset>, String> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map_err(|_| "original observation/assessment time requires RFC3339 with timezone".into())
}
pub(crate) fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}

pub(crate) fn references(step: &AcquisitionStep) -> Vec<&OutputReference> {
    step.bindings
        .iter()
        .map(|binding| &binding.output)
        .chain(step.for_each.iter().map(|binding| &binding.output))
        .chain(
            step.for_each
                .iter()
                .filter_map(|binding| binding.stop_on_witness.as_ref())
                .filter_map(|predicate| predicate.equals_output.as_ref()),
        )
        .collect()
}
pub(crate) struct PreparedAcquisition {
    pub arguments: Vec<Value>,
    pub truncated: bool,
    pub witness: Option<ResolvedWitness>,
}
pub(crate) struct ResolvedWitness {
    pointer: String,
    expected: Value,
}
pub(crate) fn prepare_arguments(
    step: &AcquisitionStep,
    steps: &BTreeMap<String, StepResult>,
) -> Result<PreparedAcquisition, String> {
    let witness = step
        .for_each
        .as_ref()
        .and_then(|binding| binding.stop_on_witness.as_ref())
        .map(|predicate| {
            let expected = if let Some(value) = &predicate.equals {
                value.clone()
            } else {
                let selected = evaluation::referenced(
                    predicate
                        .equals_output
                        .as_ref()
                        .ok_or("witness output is missing")?,
                    steps,
                )?;
                if selected.is_null() || selected.is_array() {
                    return Err(
                        "dynamic witness identity is missing or has a collection shape".to_owned(),
                    );
                }
                selected
            };
            Ok(ResolvedWitness {
                pointer: predicate.pointer.clone(),
                expected,
            })
        })
        .transpose()?;
    let mut arguments = step.arguments.clone();
    for binding in &step.bindings {
        *arguments
            .pointer_mut(&binding.argument_pointer)
            .ok_or("argument binding destination disappeared")? =
            evaluation::referenced(&binding.output, steps)?;
    }
    if let Some(binding) = &step.for_each {
        let selected = evaluation::referenced(&binding.output, steps)?;
        let values = selected
            .as_array()
            .ok_or("fan-out source requires an array of identifiers")?;
        let arguments = values
            .iter()
            .take(binding.maximum_items)
            .map(|value| {
                let mut arguments = arguments.clone();
                *arguments
                    .pointer_mut(&binding.argument_pointer)
                    .ok_or("fan-out argument destination disappeared")? = value.clone();
                Ok(arguments)
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(PreparedAcquisition {
            arguments,
            truncated: values.len() > binding.maximum_items,
            witness,
        })
    } else {
        Ok(PreparedAcquisition {
            arguments: vec![arguments],
            truncated: false,
            witness,
        })
    }
}
pub(crate) fn minimum_freshness(id: &str, plan: &EvidenceExecutionPlan) -> Option<u64> {
    let mut dependants = BTreeSet::from([id.to_owned()]);
    loop {
        let before = dependants.len();
        for step in &plan.derivations {
            if step
                .inputs
                .iter()
                .any(|reference| dependants.contains(&reference.step_id))
            {
                dependants.insert(step.id.clone());
            }
        }
        for step in &plan.acquisitions {
            if references(step)
                .iter()
                .any(|reference| dependants.contains(&reference.step_id))
            {
                dependants.insert(step.id.clone());
            }
        }
        if before == dependants.len() {
            break;
        }
    }
    plan.evaluations
        .iter()
        .filter(|evaluation| {
            evaluation
                .inputs
                .iter()
                .chain(&evaluation.applicability)
                .chain(&evaluation.challenges)
                .any(|reference| dependants.contains(&reference.step_id))
        })
        .filter_map(|evaluation| evaluation.max_age_seconds)
        .chain(
            plan.acquisitions
                .iter()
                .find(|step| step.id == id)
                .and_then(|step| step.max_age_seconds),
        )
        .min()
}
pub(crate) fn lineage(id: &str, parents: &[&StepResult]) -> StepResult {
    let source_ids = parents
        .iter()
        .flat_map(|step| step.source_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let observed_at = parents
        .iter()
        .filter_map(|step| step.observed_at.clone())
        .min_by_key(|time| parse_time(time).ok());
    let coverage = parents
        .iter()
        .fold(CoverageStatus::CompleteEnumeration, |current, step| {
            combine_coverage(&current, &step.coverage)
        });
    StepResult {
        id: id.into(),
        status: StepStatus::Unavailable,
        value: None,
        source_ids,
        observation_ids: parents
            .iter()
            .flat_map(|step| step.observation_ids.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        observed_at,
        coverage,
        truncated: parents.iter().any(|step| step.truncated),
        reused: parents.iter().all(|step| step.reused),
        diagnostic: None,
    }
}
fn combine_coverage(left: &CoverageStatus, right: &CoverageStatus) -> CoverageStatus {
    if matches!(left, CoverageStatus::Unspecified) || matches!(right, CoverageStatus::Unspecified) {
        return CoverageStatus::Unspecified;
    }
    if matches!(left, CoverageStatus::Witness) || matches!(right, CoverageStatus::Witness) {
        return CoverageStatus::Witness;
    }
    let sample_size = left
        .sample_size()
        .into_iter()
        .chain(right.sample_size())
        .min();
    match (left.is_complete() && right.is_complete(), sample_size) {
        (true, Some(sample_size)) => {
            CoverageStatus::CompleteEnumerationWithStatistics { sample_size }
        }
        (true, None) => CoverageStatus::CompleteEnumeration,
        (false, Some(sample_size)) => CoverageStatus::Statistical { sample_size },
        _ => CoverageStatus::Unspecified,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn acquire_step<E: SourceExecutor + 'static>(
    executor: Arc<E>,
    store: SourceStore,
    step: &AcquisitionStep,
    prepared: PreparedAcquisition,
    mut output: StepResult,
    policy: &Concurrency,
    limits: &ExecutionLimits,
    deadline: Instant,
    freshness: Option<u64>,
) -> (StepResult, Vec<SourceRecord>, ExecutionMetrics) {
    let availability = match &step.availability {
        SourceAvailability::Configured => None,
        SourceAvailability::Unavailable { reason } => {
            Some(format!("source operation is unavailable: {reason}"))
        }
        SourceAvailability::ConfigurationMissing { requirements } => Some(format!(
            "configuration requirements remain: {}",
            requirements.join(", ")
        )),
        SourceAvailability::HistoricalUnavailable { reason } => {
            Some(format!("historical source is unavailable: {reason}"))
        }
    };
    if let Some(message) = availability {
        output.status = StepStatus::Unavailable;
        output.diagnostic = Some(message);
        return (output, vec![], ExecutionMetrics::default());
    }
    let mut records = Vec::new();
    let mut values = Vec::new();
    let mut metrics = ExecutionMetrics::default();
    let mut all_reused = true;
    for arguments in prepared.arguments {
        let call = acquire_one(
            executor.as_ref(),
            &store,
            step,
            arguments,
            policy,
            limits,
            deadline,
            freshness,
        )
        .await;
        match call {
            Ok((record, reused, refresh_error, protocol_metrics)) => {
                metrics.reported_provider_requests +=
                    protocol_metrics.transport_requests.unwrap_or(0);
                metrics.reported_request_bytes += protocol_metrics.request_bytes.unwrap_or(0);
                metrics.reported_response_bytes += protocol_metrics.response_bytes.unwrap_or(0);
                all_reused &= reused;
                if reused {
                    metrics.reused_sources += 1;
                } else {
                    metrics.normalised_response_bytes +=
                        serde_json::to_vec(&record.value).map_or(0, |bytes| bytes.len());
                }
                output.source_ids.push(record.source_id.clone());
                output.observation_ids.push(record.observation_id.clone());
                output.observed_at = match output.observed_at {
                    Some(ref previous)
                        if parse_time(previous).ok() <= parse_time(&record.observed_at).ok() =>
                    {
                        Some(previous.clone())
                    }
                    _ => Some(record.observed_at.clone()),
                };
                output.coverage = combine_coverage(&output.coverage, &record.coverage);
                if let Some(error) = refresh_error {
                    output.diagnostic = Some(format!(
                        "fresh acquisition failed; retained measurement remains available subject to per-requirement age: {error}"
                    ));
                }
                let witness = prepared.witness.as_ref().is_some_and(|predicate| {
                    record.value.pointer(&predicate.pointer) == Some(&predicate.expected)
                });
                values.push(record.value.clone());
                records.push(record);
                if witness {
                    output.truncated = true;
                    output.coverage = combine_coverage(&output.coverage, &CoverageStatus::Witness);
                    break;
                }
            }
            Err(error) => {
                output.status = if matches!(error.kind, SourceErrorKind::Budget) {
                    StepStatus::Incomplete
                } else if matches!(
                    error.kind,
                    SourceErrorKind::Configuration
                        | SourceErrorKind::HistoricalUnavailable
                        | SourceErrorKind::Retention
                ) {
                    StepStatus::Unavailable
                } else {
                    StepStatus::Failed
                };
                output.diagnostic = Some(error.message);
                output.source_ids.sort();
                output.source_ids.dedup();
                output.observation_ids.sort();
                output.observation_ids.dedup();
                return (output, records, metrics);
            }
        }
    }
    output.source_ids.sort();
    output.source_ids.dedup();
    output.observation_ids.sort();
    output.observation_ids.dedup();
    output.status = StepStatus::Measured;
    output.reused = all_reused;
    output.value = Some(if step.for_each.is_some() {
        json!(values)
    } else {
        values.into_iter().next().unwrap_or(Value::Null)
    });
    let assessment = policy.assessment_time.clone().unwrap_or_else(now);
    if let Some(message) = crate::freshness::diagnostic(
        step.max_age_seconds,
        records.iter().map(|record| record.observed_at.as_str()),
        output.observed_at.as_deref(),
        &assessment,
    ) {
        output.status = StepStatus::Unavailable;
        output.value = None;
        output.diagnostic = Some(message);
    }
    (output, records, metrics)
}

#[allow(clippy::too_many_arguments)]
async fn acquire_one<E: SourceExecutor>(
    executor: &E,
    store: &SourceStore,
    step: &AcquisitionStep,
    arguments: Value,
    policy: &Concurrency,
    limits: &ExecutionLimits,
    deadline: Instant,
    freshness: Option<u64>,
) -> Result<(SourceRecord, bool, Option<String>, SourceMetrics), SourceError> {
    let request = tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), async {
        let provider = policy
            .providers
            .get(&step.source.provider_id)
            .ok_or_else(|| SourceError::acquisition("planned provider has no scheduler"))?;
        let _provider = provider
            .acquire()
            .await
            .map_err(|_| SourceError::acquisition("provider admission scheduler closed"))?;
        let _global = policy
            .global
            .acquire()
            .await
            .map_err(|_| SourceError::acquisition("global admission scheduler closed"))?;
        executor.bind_until(step, arguments, deadline).await
    })
    .await
    .map_err(|_| SourceError {
        kind: SourceErrorKind::Budget,
        message: "source binding deadline exhausted".into(),
    })??;
    request
        .source
        .validate()
        .map_err(SourceError::acquisition)?;
    if !request.arguments.is_object() {
        return Err(SourceError::acquisition(
            "resolved source arguments require an object",
        ));
    }
    let key = request.source_id().map_err(SourceError::acquisition)?;
    let lease = store.lock(&key, deadline).await.map_err(store_error)?;
    let cached = store
        .load_until(&key, &lease, deadline)
        .await
        .map_err(store_error)?;
    let current = policy.assessment_time.clone().unwrap_or_else(now);
    let fresh = cached.as_ref().is_some_and(|record| {
        parse_time(&current)
            .ok()
            .zip(parse_time(&record.observed_at).ok())
            .is_some_and(|(now, at)| {
                at <= now
                    && freshness.is_none_or(|maximum| {
                        now.signed_duration_since(at)
                            .to_std()
                            .is_ok_and(|age| age <= Duration::from_secs(maximum))
                    })
            })
    });
    let acquired_this_run = policy
        .acquired
        .lock()
        .map_err(|_| SourceError::acquisition("source sharing state is unavailable"))?
        .contains(&key);
    if ((fresh && (acquired_this_run || !matches!(step.reuse, ReusePolicy::Refresh)))
        || matches!(step.reuse, ReusePolicy::RetainedOnly))
        && let Some(record) = cached
    {
        return Ok((record, true, None, request.admission_metrics.clone()));
    }
    if matches!(step.reuse, ReusePolicy::RetainedOnly) {
        return Err(SourceError {
            kind: SourceErrorKind::HistoricalUnavailable,
            message: "no compatible retained source observation exists".into(),
        });
    }
    let mut received = false;
    let acquire = async {
        let provider = policy
            .providers
            .get(&request.source.provider_id)
            .ok_or_else(|| {
                SourceError::acquisition("bound provider differs from the planned scheduling scope")
            })?;
        let _provider = provider
            .acquire()
            .await
            .map_err(|_| SourceError::acquisition("source provider scheduler closed"))?;
        let _global = policy
            .global
            .acquire()
            .await
            .map_err(|_| SourceError::acquisition("global source scheduler closed"))?;
        ensure_deadline(deadline)?;
        let mut reservation = RequestReservation::new(policy)?;
        store
            .begin_until(&key, &lease, deadline)
            .await
            .map_err(store_error)?;
        ensure_deadline(deadline)?;
        reservation.commit();
        let observation = executor.execute(&request, deadline).await?;
        received = true;
        ensure_deadline(deadline)?;
        let observed = parse_time(&observation.observed_at).map_err(SourceError::acquisition)?;
        if let Some(interval) = &request.source.historical_interval
            && (observed < parse_time(&interval.start).map_err(SourceError::acquisition)?
                || observed > parse_time(&interval.end).map_err(SourceError::acquisition)?)
        {
            return Err(SourceError::acquisition(
                "historical observation time lies outside the requested interval",
            ));
        }
        let bytes = serde_json::to_vec(&observation.value)
            .map_err(|_| SourceError::acquisition("source observation could not be encoded"))?;
        if bytes.len() > limits.maximum_source_bytes {
            return Err(SourceError {
                kind: SourceErrorKind::Budget,
                message: "normalised source observation exceeds its byte limit".into(),
            });
        }
        let mut protocol_metrics = observation.metrics;
        for (total, admitted) in [
            (
                &mut protocol_metrics.transport_requests,
                request.admission_metrics.transport_requests,
            ),
            (
                &mut protocol_metrics.request_bytes,
                request.admission_metrics.request_bytes,
            ),
            (
                &mut protocol_metrics.response_bytes,
                request.admission_metrics.response_bytes,
            ),
        ] {
            if let Some(admitted) = admitted {
                *total = Some(total.unwrap_or(0).saturating_add(admitted));
            }
        }
        let record = SourceRecord {
            schema: "eal.shared-source/1".into(),
            source_id: key.clone(),
            observation_id: String::new(),
            source: request.source.clone(),
            arguments: request.arguments.clone(),
            value_sha256: canonical_digest(&observation.value).map_err(SourceError::acquisition)?,
            value: observation.value,
            observed_at: observation.observed_at,
            provenance: observation.provenance,
            coverage: observation.coverage,
            metrics: protocol_metrics.clone(),
            integrity: String::new(),
        };
        let record = store
            .publish_until(record, &lease, deadline)
            .await
            .map_err(|error| match error {
                StoreIoError::Deadline => SourceError {
                    kind: SourceErrorKind::Budget,
                    message: error.to_string(),
                },
                StoreIoError::Storage(message) => SourceError {
                    kind: SourceErrorKind::Retention,
                    message,
                },
            })?;
        ensure_deadline(deadline)?;
        policy
            .acquired
            .lock()
            .map_err(|_| SourceError::acquisition("source sharing state is unavailable"))?
            .insert(key.clone());
        Ok((record, false, None, protocol_metrics))
    };
    let outcome = tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), acquire)
        .await
        .unwrap_or_else(|_| {
            Err(SourceError {
                kind: SourceErrorKind::Budget,
                message: "source acquisition deadline exhausted".into(),
            })
        });
    match (outcome, cached) {
        (Ok(record), _) => Ok(record),
        (Err(error), Some(record))
            if !received
                && !matches!(
                    error.kind,
                    SourceErrorKind::SchemaDrift
                        | SourceErrorKind::Configuration
                        | SourceErrorKind::Retention
                ) =>
        {
            Ok((
                record,
                true,
                Some(error.message),
                request.admission_metrics.clone(),
            ))
        }
        (Err(error), _) => Err(error),
    }
}

fn store_error(error: StoreIoError) -> SourceError {
    match error {
        StoreIoError::Deadline => SourceError {
            kind: SourceErrorKind::Budget,
            message: error.to_string(),
        },
        StoreIoError::Storage(message) => SourceError::acquisition(message),
    }
}
fn ensure_deadline(deadline: Instant) -> Result<(), SourceError> {
    if Instant::now() >= deadline {
        Err(SourceError {
            kind: SourceErrorKind::Budget,
            message: "source acquisition deadline exhausted".into(),
        })
    } else {
        Ok(())
    }
}
struct RequestReservation<'a> {
    policy: &'a Concurrency,
    committed: bool,
}
impl<'a> RequestReservation<'a> {
    fn new(policy: &'a Concurrency) -> Result<Self, SourceError> {
        policy
            .requests
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |calls| {
                (calls < policy.maximum_requests).then_some(calls + 1)
            })
            .map_err(|_| SourceError {
                kind: SourceErrorKind::Budget,
                message: "source request budget exhausted".into(),
            })?;
        Ok(Self {
            policy,
            committed: false,
        })
    }
    fn commit(&mut self) {
        self.committed = true;
    }
}
impl Drop for RequestReservation<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.policy.requests.fetch_sub(1, Ordering::Relaxed);
        }
    }
}
