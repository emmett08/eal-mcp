//! Finite correspondence checks over operator-registered EAL arguments.
mod answer;
mod check;
mod requirements;

use crate::{
    contracts::{ServerSpec, TASK_SCHEMA, TaskSpec},
    host::HostPaths,
};
use eal_core::{ExecutionLimits, canonical};
use eal_runtime::{
    bootstrap::{self, RuntimeOptions},
    knowledge::EALKnowledgeBase,
    store::utc_now,
};
use serde_json::{Value, json};
use std::sync::Arc;

pub use answer::validate_answer;
pub use check::{check_task, check_task_with_host};
pub use requirements::validate_requirements;

pub const PROFILE_NAMES: [&str; 2] = ["registered_claims/1", "api_load/1"];
pub const API_METHOD: &str = "engineering/api-load-criteria/1";
pub(super) const WORKLOAD_FIELDS: [&str; 5] = [
    "service",
    "build_id",
    "run_id",
    "concurrent_clients",
    "dataset",
];
pub(super) const CRITERIA: [(&str, i64); 3] = [
    ("minimum_requests", 100),
    ("maximum_sample_p95_ms", 200),
    ("maximum_error_rate_percent", 1),
];

pub fn knowledge(server: &ServerSpec) -> Result<EALKnowledgeBase, String> {
    knowledge_with_host(server, &HostPaths::current()?)
}

/// Rebuild host configuration on every check instead of trusting a cached service.
pub fn knowledge_with_host(
    server: &ServerSpec,
    host: &HostPaths,
) -> Result<EALKnowledgeBase, String> {
    server.validate()?;
    host.validate()?;
    let mut options = RuntimeOptions::new(server.workspace.clone());
    options.registry = server.registry.clone();
    options.database = server.database.clone();
    options.limits = server.limits.clone();
    options.methods = server.methods.clone().unwrap_or_else(|| "builtin".into());
    options.supervisor = Some(host.supervisor.clone());
    options.method_worker = Some(host.method_worker.clone());
    EALKnowledgeBase::new(Arc::new(bootstrap::build_service(options)?))
}

pub fn pin_task(server: &ServerSpec, task: &TaskSpec) -> Result<Value, String> {
    pin_task_with_host(server, task, &HostPaths::current()?)
}

pub fn pin_task_with_host(
    server: &ServerSpec,
    task: &TaskSpec,
    host: &HostPaths,
) -> Result<Value, String> {
    task.validate()?;
    let knowledge = knowledge_with_host(server, host)?;
    let entry = knowledge.catalogue.get(&task.entry_id, true)?;
    let mut anchor = json!({"schema":TASK_SCHEMA,"task":task.to_value()?,
        "source_digest":entry["source_digest"],
        "method_registry_fingerprint":entry["method_registry_fingerprint"],
        "registered_context":entry["context"],"pinned_at":utc_now()});
    anchor["task_digest"] = json!(digest(&anchor)?);
    Ok(anchor)
}

pub(super) fn digest(value: &Value) -> Result<String, String> {
    canonical::canonical_digest(
        value,
        &ExecutionLimits {
            json_depth: 128,
            ..Default::default()
        },
    )
}

pub(super) fn same_json(left: &Value, right: &Value) -> Result<bool, String> {
    Ok(digest(left)? == digest(right)?)
}

pub(super) fn exact_keys(value: &Value, keys: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key))
    })
}

pub(super) fn integer(value: &Value) -> bool {
    value.as_number().is_some_and(|number| {
        !number
            .to_string()
            .bytes()
            .any(|byte| matches!(byte, b'.' | b'e' | b'E'))
    })
}
