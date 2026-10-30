//! Explicit host installation of the finite Rust ASPIC method.
mod core_sources;
use eal_core::methods::{
    MethodContract, MethodError, MethodRegistry, NativeMethodIdentity, default_registry,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

/// The frozen selector identifies the query contract, independently of its backend.
pub const ASPIC_METHOD: &str = "argumentation/aspic/2";

fn descriptor() -> &'static Value {
    static DESCRIPTOR: OnceLock<Value> = OnceLock::new();
    DESCRIPTOR.get_or_init(|| {
        serde_json::from_str(include_str!("aspic-contract.json")).expect("frozen ASPIC contract")
    })
}

pub fn theory_schema() -> &'static Value {
    &descriptor()["input_schema"]["properties"]["theory"]
}
pub fn output_schema() -> &'static Value {
    &descriptor()["output_schema"]
}

fn callback(payload: &Value, limits: &eal_core::ExecutionLimits) -> Result<Value, MethodError> {
    let computed = crate::solver::solve_aspic(payload, limits).and_then(|formal| {
        check_output_bytes(&formal, limits)?;
        Ok(formal)
    });
    computed.map_err(|error| match error {
        crate::AspicError::Invalid(message) => MethodError::Invalid(message),
        crate::AspicError::Incomplete(message) => MethodError::Incomplete(message),
    })
}

/// Apply the registered output cap to the complete successful response envelope.
/// This is a pure byte check; process ownership and transport remain separate.
pub(crate) fn check_output_bytes(
    formal: &Value,
    limits: &eal_core::ExecutionLimits,
) -> Result<(), crate::AspicError> {
    let envelope = json!({"ok":true,"output":formal});
    if eal_core::canonical::canonical_json_bytes(&envelope, limits)?.len()
        > aspic_contract().max_output_bytes
    {
        return Err(crate::AspicError::Invalid(
            "Method output exceeds byte limit".into(),
        ));
    }
    Ok(())
}

/// SHA-256 of the retained Rust implementation bundle, not a binary attestation.
pub fn module_identity() -> String {
    static IDENTITY: OnceLock<String> = OnceLock::new();
    IDENTITY
        .get_or_init(|| {
            let mut digest = Sha256::new();
            for (path, source) in [
                ("registry.rs", include_str!("registry.rs")),
                ("aspic-contract.json", include_str!("aspic-contract.json")),
                ("solver.rs", include_str!("solver.rs")),
                ("solver/validation.rs", include_str!("solver/validation.rs")),
                ("solver/model.rs", include_str!("solver/model.rs")),
                (
                    "solver/construction.rs",
                    include_str!("solver/construction.rs"),
                ),
                (
                    "solver/preferences.rs",
                    include_str!("solver/preferences.rs"),
                ),
                ("solver/extensions.rs", include_str!("solver/extensions.rs")),
                ("solver/defeats.rs", include_str!("solver/defeats.rs")),
                ("lib.rs", include_str!("lib.rs")),
                ("error.rs", include_str!("error.rs")),
                ("Cargo.toml", include_str!("../Cargo.toml")),
                ("package-version", env!("CARGO_PKG_VERSION")),
            ]
            .into_iter()
            .chain(core_sources::SOURCES.iter().copied())
            {
                digest.update((path.len() as u64).to_be_bytes());
                digest.update(path.as_bytes());
                digest.update((source.len() as u64).to_be_bytes());
                digest.update(source.as_bytes());
            }
            format!("{:x}", digest.finalize())
        })
        .clone()
}

/// A checked pure callback contract. Declared timeout/memory metadata is not process supervision.
pub fn aspic_contract() -> &'static MethodContract {
    static CONTRACT: OnceLock<MethodContract> = OnceLock::new();
    CONTRACT.get_or_init(|| {
        let descriptor = descriptor();
        let mut contract = MethodContract::native(
            ASPIC_METHOD,
            descriptor["evidence_kind"]
                .as_str()
                .expect("ASPIC evidence kind"),
            descriptor["input_schema"].clone(),
            descriptor["query_schema"].clone(),
            descriptor["output_schema"].clone(),
            NativeMethodIdentity {
                version: format!("eal-rust-aspic-finite-{}", env!("CARGO_PKG_VERSION")),
                // The declared artefact is this source bundle; linked machine code is separate.
                artifact_sha256: module_identity(),
            },
            callback,
        );
        contract.outputs = descriptor["outputs"]
            .as_object()
            .expect("ASPIC output map")
            .iter()
            .map(|(path, kind)| {
                (
                    path.clone(),
                    kind.as_str().expect("output interpretation").to_owned(),
                )
            })
            .collect();
        contract.quantities = descriptor["quantities"]
            .as_array()
            .expect("ASPIC quantities")
            .iter()
            .map(|q| q.as_str().expect("quantity").to_owned())
            .collect();
        contract.exact_unit = descriptor["exact_unit"]
            .as_bool()
            .expect("ASPIC exact unit");
        contract.timeout_seconds = descriptor["limits"]["timeout_seconds"]
            .as_f64()
            .expect("timeout");
        contract.max_input_bytes = descriptor["limits"]["max_input_bytes"]
            .as_u64()
            .expect("input limit") as usize;
        contract.max_output_bytes = descriptor["limits"]["max_output_bytes"]
            .as_u64()
            .expect("output limit") as usize;
        contract.max_memory_bytes = descriptor["limits"]["max_memory_bytes"]
            .as_u64()
            .expect("memory limit") as usize;
        contract
    })
}

/// Add ASPIC to an immutable registry, retaining every previously selected method.
pub fn with_aspic(registry: &MethodRegistry) -> Result<MethodRegistry, String> {
    registry.with_method(aspic_contract().clone())
}

/// Default ten methods plus the explicitly installed finite ASPIC method.
pub fn aspic_registry() -> MethodRegistry {
    with_aspic(default_registry()).expect("frozen ASPIC native contract")
}

#[cfg(test)]
mod tests {
    use super::*;
    use eal_core::{ExecutionLimits, canonical::canonical_json_bytes};

    #[test]
    fn complete_output_envelope_accepts_exact_limit_and_rejects_one_extra_byte() {
        let limits = ExecutionLimits::default();
        let maximum = aspic_contract().max_output_bytes;
        let empty = json!({"padding":""});
        let fixed = canonical_json_bytes(&json!({"ok":true,"output":&empty}), &limits)
            .unwrap()
            .len();
        let exact = json!({"padding":"x".repeat(maximum-fixed)});
        assert_eq!(
            canonical_json_bytes(&json!({"ok":true,"output":&exact}), &limits)
                .unwrap()
                .len(),
            maximum
        );
        assert_eq!(check_output_bytes(&exact, &limits), Ok(()));

        let one_over = json!({"padding":"x".repeat(maximum-fixed+1)});
        assert_eq!(
            canonical_json_bytes(&json!({"ok":true,"output":&one_over}), &limits)
                .unwrap()
                .len(),
            maximum + 1
        );
        assert!(
            canonical_json_bytes(&one_over, &limits).unwrap().len() <= maximum,
            "details alone would admit this oversized response"
        );
        assert_eq!(
            check_output_bytes(&one_over, &limits),
            Err(crate::AspicError::Invalid(
                "Method output exceeds byte limit".into()
            ))
        );
    }
}
