//! Bounded dependency execution retaining shared source identities and original age.
use crate::{acquisition::*, evaluation, plan::*, storage::SourceStore};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{sync::Semaphore, task::JoinSet};

pub type SourceFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, SourceError>> + Send + 'a>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedAcquisition {
    pub source: SourceIdentity,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub arguments: Value,
    #[serde(default)]
    pub admission_metrics: SourceMetrics,
}
impl ResolvedAcquisition {
    pub fn source_id(&self) -> Result<String, String> {
        canonical_digest(&(&self.source, &self.arguments))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceMetrics {
    pub transport_requests: Option<u64>,
    pub request_bytes: Option<u64>,
    pub response_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CoverageStatus {
    #[default]
    Unspecified,
    Witness,
    CompleteEnumeration,
    CompleteEnumerationWithStatistics {
        sample_size: u64,
    },
    Statistical {
        sample_size: u64,
    },
}

impl CoverageStatus {
    pub fn is_complete(&self) -> bool {
        matches!(
            self,
            Self::CompleteEnumeration | Self::CompleteEnumerationWithStatistics { .. }
        )
    }
    pub fn sample_size(&self) -> Option<u64> {
        match self {
            Self::Statistical { sample_size }
            | Self::CompleteEnumerationWithStatistics { sample_size }
                if *sample_size > 0 =>
            {
                Some(*sample_size)
            }
            _ => None,
        }
    }
    pub fn supplies_witness_route(&self) -> bool {
        self.is_complete() || matches!(self, Self::Witness)
    }
}
/// Retain independently qualified population completeness and measured sample size.
pub fn measured_coverage(
    value: &Value,
    guarantees: &[eal_collector_contracts::capability::CoverageGuarantee],
) -> CoverageStatus {
    use eal_collector_contracts::capability::LogicalStrength;
    let complete = guarantees.iter().any(|guarantee| {
        guarantee.strength == LogicalStrength::CompleteEnumeration
            && guarantee
                .completeness_pointer
                .as_ref()
                .is_some_and(|pointer| value.pointer(pointer) == Some(&Value::Bool(true)))
    });
    let sample_size = guarantees
        .iter()
        .filter(|guarantee| guarantee.strength == LogicalStrength::StatisticalEstimate)
        .filter_map(|guarantee| guarantee.sample_size_pointer.as_ref())
        .filter_map(|pointer| value.pointer(pointer))
        .filter_map(Value::as_u64)
        .filter(|count| *count > 0)
        .min();
    match (complete, sample_size) {
        (true, Some(sample_size)) => {
            CoverageStatus::CompleteEnumerationWithStatistics { sample_size }
        }
        (true, None) => CoverageStatus::CompleteEnumeration,
        (false, Some(sample_size)) => CoverageStatus::Statistical { sample_size },
        _ if guarantees
            .iter()
            .any(|guarantee| guarantee.strength == LogicalStrength::ExistentialWitness) =>
        {
            CoverageStatus::Witness
        }
        _ => CoverageStatus::Unspecified,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceObservation {
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub value: Value,
    pub observed_at: String,
    /// Executor-owned acquisition provenance. Reviewer-record consumption also
    /// requires actual descriptor-derived `qualified_support_types` and
    /// `qualified_logical_strengths` arrays containing `judgement`; source
    /// response content must never supply those qualification assertions.
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub provenance: Value,
    #[serde(default)]
    pub coverage: CoverageStatus,
    #[serde(default)]
    pub metrics: SourceMetrics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceErrorKind {
    Configuration,
    SchemaDrift,
    Acquisition,
    Budget,
    Protocol,
    HistoricalUnavailable,
    Retention,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceError {
    pub kind: SourceErrorKind,
    pub message: String,
}
impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for SourceError {}
impl SourceError {
    pub fn acquisition(message: impl Into<String>) -> Self {
        Self {
            kind: SourceErrorKind::Acquisition,
            message: message.into(),
        }
    }
}

/// Resolve against the current operator-owned provider before cache lookup.
/// Implementations must reject requested schema/version drift and use actual
/// configuration/authentication fingerprints, never model-provided authority.
pub trait SourceExecutor: Send + Sync {
    fn bind<'a>(
        &'a self,
        step: &'a AcquisitionStep,
        arguments: Value,
    ) -> SourceFuture<'a, ResolvedAcquisition>;
    /// Deadline-aware admission; existing in-process executors can keep bind().
    fn bind_until<'a>(
        &'a self,
        step: &'a AcquisitionStep,
        arguments: Value,
        _deadline: Instant,
    ) -> SourceFuture<'a, ResolvedAcquisition> {
        self.bind(step, arguments)
    }
    fn execute<'a>(
        &'a self,
        request: &'a ResolvedAcquisition,
        deadline: Instant,
    ) -> SourceFuture<'a, SourceObservation>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRecord {
    pub schema: String,
    pub source_id: String,
    /// Immutable measurement identity, distinct from the physical reuse index.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub observation_id: String,
    pub source: SourceIdentity,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub arguments: Value,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub value: Value,
    pub value_sha256: String,
    pub observed_at: String,
    #[serde(deserialize_with = "eal_collector_contracts::deserialize_value_exact")]
    pub provenance: Value,
    pub coverage: CoverageStatus,
    pub metrics: SourceMetrics,
    #[serde(default)]
    pub integrity: String,
}

impl SourceRecord {
    pub fn observation_identity(&self) -> Result<String, String> {
        canonical_digest(&(
            "eal.source-observation/1",
            &self.source_id,
            &self.value_sha256,
            &self.observed_at,
            &self.provenance,
            &self.coverage,
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Measured,
    Derived,
    Unavailable,
    Failed,
    Incomplete,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepResult {
    pub id: String,
    pub status: StepStatus,
    pub value: Option<Value>,
    pub source_ids: Vec<String>,
    #[serde(default)]
    pub observation_ids: Vec<String>,
    pub observed_at: Option<String>,
    pub coverage: CoverageStatus,
    pub truncated: bool,
    pub reused: bool,
    pub diagnostic: Option<String>,
}
impl StepResult {
    pub(crate) fn unavailable(id: &str, status: StepStatus, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status,
            value: None,
            source_ids: vec![],
            observation_ids: vec![],
            observed_at: None,
            coverage: CoverageStatus::Unspecified,
            truncated: false,
            reused: false,
            diagnostic: Some(message.into()),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationStatus {
    Satisfied,
    NotSatisfied,
    Pending,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationResult {
    pub id: String,
    pub requirement_id: String,
    pub status: EvaluationStatus,
    pub source_ids: Vec<String>,
    #[serde(default)]
    pub observation_ids: Vec<String>,
    pub observed_at: Option<String>,
    pub value: Option<Value>,
    pub diagnostic: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionMetrics {
    pub source_requests: usize,
    pub reused_sources: usize,
    pub normalised_response_bytes: usize,
    pub generated_code_count: usize,
    pub elapsed_ms: u64,
    pub reported_provider_requests: u64,
    pub reported_request_bytes: u64,
    pub reported_response_bytes: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionReport {
    pub schema: String,
    pub plan_sha256: String,
    pub evaluated_at: String,
    pub complete: bool,
    pub steps: BTreeMap<String, StepResult>,
    pub sources: BTreeMap<String, SourceRecord>,
    /// Every exact observation used by this execution, including older records
    /// of a physical source that was subsequently refreshed.
    #[serde(default)]
    pub observations: BTreeMap<String, SourceRecord>,
    pub evaluations: Vec<EvaluationResult>,
    pub metrics: ExecutionMetrics,
    pub metric_scope: String,
}

#[derive(Clone, Debug)]
pub struct ExecutionLimits {
    pub timeout: Duration,
    pub maximum_source_requests: usize,
    pub maximum_source_bytes: usize,
}
impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300),
            maximum_source_requests: 4096,
            maximum_source_bytes: 1_048_576,
        }
    }
}

pub struct EvidenceExecutor<E: SourceExecutor + 'static> {
    executor: Arc<E>,
    store: SourceStore,
    limits: ExecutionLimits,
}
impl<E: SourceExecutor + 'static> EvidenceExecutor<E> {
    pub fn new(
        executor: Arc<E>,
        store: SourceStore,
        limits: ExecutionLimits,
    ) -> Result<Self, String> {
        if limits.timeout.is_zero()
            || limits.timeout > Duration::from_secs(86_400)
            || limits.maximum_source_requests == 0
            || limits.maximum_source_requests > 65_536
            || !(1024..=16_777_216).contains(&limits.maximum_source_bytes)
        {
            return Err("evidence execution limits are outside supported bounds".into());
        }
        Ok(Self {
            executor,
            store,
            limits,
        })
    }

    /// Execute only this requirement's transitive acquisition/derivation closure.
    /// The report retains the validated full plan identity for collector binding.
    pub async fn execute_requirement(
        &self,
        plan: &EvidenceExecutionPlan,
        requirement_id: &str,
        assessment_time: Option<&str>,
    ) -> Result<ExecutionReport, String> {
        plan.validate()?;
        let selected: Vec<_> = plan
            .evaluations
            .iter()
            .filter(|evaluation| evaluation.requirement_id == requirement_id)
            .cloned()
            .collect();
        if selected.len() != 1 {
            return Err("execution requires one unambiguous requirement".into());
        }
        let mut retained: std::collections::BTreeSet<_> = selected[0]
            .inputs
            .iter()
            .chain(&selected[0].applicability)
            .chain(&selected[0].challenges)
            .map(|reference| reference.step_id.clone())
            .collect();
        loop {
            let before = retained.len();
            for step in &plan.derivations {
                if retained.contains(&step.id) {
                    retained.extend(
                        step.inputs
                            .iter()
                            .map(|reference| reference.step_id.clone()),
                    );
                }
            }
            for step in &plan.acquisitions {
                if retained.contains(&step.id) {
                    retained.extend(
                        references(step)
                            .iter()
                            .map(|reference| reference.step_id.clone()),
                    );
                }
            }
            if retained.len() == before {
                break;
            }
        }
        let projection = EvidenceExecutionPlan {
            schema: plan.schema.clone(),
            acquisitions: plan
                .acquisitions
                .iter()
                .filter(|step| retained.contains(&step.id))
                .cloned()
                .collect(),
            derivations: plan
                .derivations
                .iter()
                .filter(|step| retained.contains(&step.id))
                .cloned()
                .collect(),
            evaluations: selected,
            scheduling: plan.scheduling.clone(),
        };
        let mut report = self.execute(&projection, assessment_time).await?;
        report.plan_sha256 = canonical_digest(plan)?;
        Ok(report)
    }

    /// Explicit assessment time is a cut-off. Without one, evaluate after the
    /// acquisition phase so new observations do not appear to come from future.
    pub async fn execute(
        &self,
        plan: &EvidenceExecutionPlan,
        assessment_time: Option<&str>,
    ) -> Result<ExecutionReport, String> {
        plan.validate()?;
        if let Some(now) = assessment_time {
            parse_time(now)?;
        }
        let started = Instant::now();
        let deadline = started
            .checked_add(self.limits.timeout)
            .ok_or("execution timeout is not representable")?;
        let current = assessment_time.map(str::to_owned).unwrap_or_else(now);
        let policy = Arc::new(Concurrency {
            global: Arc::new(Semaphore::new(plan.scheduling.maximum_concurrent_sources)),
            providers: plan
                .acquisitions
                .iter()
                .map(|step| {
                    (
                        step.source.provider_id.clone(),
                        Arc::new(Semaphore::new(
                            *plan
                                .scheduling
                                .per_provider
                                .get(&step.source.provider_id)
                                .unwrap_or(&1),
                        )),
                    )
                })
                .collect(),
            requests: AtomicUsize::new(0),
            assessment_time: assessment_time.map(str::to_owned),
            acquired: std::sync::Mutex::new(std::collections::BTreeSet::new()),
            maximum_requests: self.limits.maximum_source_requests,
        });
        let mut acquisitions: BTreeMap<_, _> = plan
            .acquisitions
            .iter()
            .map(|step| (step.id.clone(), step.clone()))
            .collect();
        let mut derivations: BTreeMap<_, _> = plan
            .derivations
            .iter()
            .map(|step| (step.id.clone(), step.clone()))
            .collect();
        let mut steps = BTreeMap::new();
        let mut sources = BTreeMap::new();
        let mut observations = BTreeMap::new();
        let mut metrics = ExecutionMetrics::default();
        let mut owned_sources = BTreeMap::<String, Vec<String>>::new();
        let mut running = JoinSet::new();
        loop {
            let ready: Vec<_> = derivations
                .values()
                .filter(|step| {
                    step.inputs
                        .iter()
                        .all(|reference| steps.contains_key(&reference.step_id))
                })
                .map(|step| step.id.clone())
                .collect();
            let derived_any = !ready.is_empty();
            for id in ready {
                let step = derivations.remove(&id).expect("ready derivation");
                let parents: Vec<_> = step
                    .inputs
                    .iter()
                    .filter_map(|reference| steps.get(&reference.step_id))
                    .collect();
                let mut output = lineage(&id, &parents);
                match step
                    .inputs
                    .iter()
                    .map(|reference| evaluation::referenced(reference, &steps))
                    .collect::<Result<Vec<_>, _>>()
                    .and_then(|values| evaluation::derive(&step.operation, &values))
                {
                    Ok(value) => {
                        output.status = StepStatus::Derived;
                        if matches!(step.operation, DerivationOperation::AnyIdentityEqual) {
                            output.coverage = if value == Value::Bool(true) {
                                CoverageStatus::Witness
                            } else if parents
                                .get(1)
                                .is_some_and(|step| step.coverage.is_complete())
                                && !output.truncated
                            {
                                CoverageStatus::CompleteEnumeration
                            } else {
                                CoverageStatus::Unspecified
                            };
                        }
                        output.value = Some(value);
                    }
                    Err(message) => {
                        output.status = StepStatus::Unavailable;
                        output.diagnostic = Some(message);
                    }
                }
                steps.insert(id, output);
            }
            let ready: Vec<_> = acquisitions
                .values()
                .filter(|step| {
                    references(step)
                        .iter()
                        .all(|reference| steps.contains_key(&reference.step_id))
                })
                .map(|step| step.id.clone())
                .collect();
            for id in ready {
                let step = acquisitions.remove(&id).expect("ready acquisition");
                let prepared = prepare_arguments(&step, &steps);
                let parents: Vec<_> = references(&step)
                    .iter()
                    .filter_map(|reference| steps.get(&reference.step_id))
                    .cloned()
                    .collect();
                let upstream = lineage(&id, &parents.iter().collect::<Vec<_>>());
                let freshness = minimum_freshness(&id, plan);
                let (executor, store, policy, limits, current) = (
                    self.executor.clone(),
                    self.store.clone(),
                    policy.clone(),
                    self.limits.clone(),
                    current.clone(),
                );
                running.spawn(async move {
                    let result = match prepared {
                        Ok((arguments, truncated)) => {
                            let mut upstream = upstream;
                            upstream.truncated |= truncated;
                            acquire_step(
                                executor, store, &step, arguments, upstream, &policy, &limits,
                                deadline, &current, freshness,
                            )
                            .await
                        }
                        Err(message) => {
                            let mut output = upstream;
                            output.status = StepStatus::Unavailable;
                            output.diagnostic = Some(message);
                            (output, Vec::new(), ExecutionMetrics::default())
                        }
                    };
                    (id, result)
                });
            }
            if acquisitions.is_empty() && derivations.is_empty() && running.is_empty() {
                break;
            }
            if derived_any {
                continue;
            }
            if Instant::now() >= deadline {
                running.abort_all();
                while running.join_next().await.is_some() {}
                break;
            }
            match tokio::time::timeout_at(
                tokio::time::Instant::from_std(deadline),
                running.join_next(),
            )
            .await
            {
                Ok(Some(Ok((id, (result, records, counts))))) => {
                    metrics.reused_sources += counts.reused_sources;
                    metrics.reported_provider_requests += counts.reported_provider_requests;
                    metrics.reported_request_bytes += counts.reported_request_bytes;
                    metrics.reported_response_bytes += counts.reported_response_bytes;
                    metrics.normalised_response_bytes += counts.normalised_response_bytes;
                    owned_sources.insert(
                        id.clone(),
                        records
                            .iter()
                            .map(|record| record.observation_id.clone())
                            .collect(),
                    );
                    for record in records {
                        observations
                            .entry(record.observation_id.clone())
                            .or_insert_with(|| record.clone());
                        sources.insert(record.source_id.clone(), record);
                    }
                    steps.insert(id, result);
                }
                Ok(Some(Err(_))) => return Err("evidence acquisition worker failed".into()),
                Ok(None) => {
                    if !acquisitions.is_empty() || !derivations.is_empty() {
                        return Err("execution graph made no progress".into());
                    }
                }
                Err(_) => {
                    running.abort_all();
                    while running.join_next().await.is_some() {}
                    break;
                }
            }
        }
        for id in plan
            .acquisitions
            .iter()
            .map(|step| &step.id)
            .chain(plan.derivations.iter().map(|step| &step.id))
        {
            if !steps.contains_key(id) {
                steps.insert(
                    id.clone(),
                    StepResult::unavailable(
                        id,
                        StepStatus::Incomplete,
                        "execution deadline exhausted",
                    ),
                );
            }
        }
        let evaluated_at = assessment_time.map(str::to_owned).unwrap_or_else(now);
        crate::freshness::enforce(
            plan,
            &mut steps,
            &observations,
            &owned_sources,
            &evaluated_at,
        );
        let evaluations = plan
            .evaluations
            .iter()
            .map(|evaluation| {
                evaluation::evaluate(evaluation, &steps, &observations, &evaluated_at)
            })
            .collect::<Vec<_>>();
        metrics.source_requests = policy.requests.load(Ordering::Relaxed);
        metrics.elapsed_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
        let complete = steps
            .values()
            .all(|step| matches!(step.status, StepStatus::Measured | StepStatus::Derived))
            && evaluations
                .iter()
                .all(|result| result.status != EvaluationStatus::Pending);
        Ok(ExecutionReport{schema:"eal.evidence-execution/1".into(),plan_sha256:canonical_digest(plan)?,evaluated_at,complete,steps,sources,observations,evaluations,metrics,metric_scope:"source executor calls and normalised JSON response payloads; reported provider JSON protocol metrics include cache-admission checks; failed calls may lack byte counts; provider per-observation metrics remain on source records".into()})
    }
}
