//! Explicitly installed engineering methods shipped with the native implementation.
mod api_load;
mod reachability;
mod rms;
mod sampled_negative;

use super::{MethodContract, MethodRegistry, default_registry};
use serde_json::Value;

pub const IDENTIFIERS: [&str; 4] = [
    "engineering/api-load-criteria/1",
    "engineering/reachability/1",
    "engineering/rms/1",
    "engineering/sampled-negative/1",
];

/// Select optional compiled methods by exact identifier; the default registry stays unchanged.
pub fn optional_registry(identifiers: &[&str]) -> Result<MethodRegistry, String> {
    let mut selected = Vec::new();
    for id in identifiers {
        if !IDENTIFIERS.contains(id) {
            return Err(format!("Unknown optional native method {id:?}"));
        }
        let descriptor = &super::contracts::optional_specs()[*id];
        selected.push(MethodContract::from_descriptor(id, descriptor));
    }
    default_registry().with_compiled_methods(selected)
}
pub fn engineering_registry() -> MethodRegistry {
    optional_registry(&IDENTIFIERS).expect("frozen optional native contracts")
}

pub(super) fn compute(mode: &str, payload: &Value) -> Result<Value, String> {
    match mode {
        "engineering/api-load-criteria" => api_load::assess(payload),
        "engineering/reachability" => reachability::assess(payload),
        "engineering/rms" => rms::assess(payload),
        "engineering/sampled-negative" => sampled_negative::assess(payload),
        _ => Err("Unknown optional native method".into()),
    }
}
