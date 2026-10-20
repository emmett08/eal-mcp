//! The pinned language guide, with the selected native implementation identity.
use eal_core::{
    ExecutionLimits,
    methods::{self, MethodRegistry},
    propositions,
};
use serde_json::{Value, json};
use std::sync::OnceLock;

pub const REFERENCE_REVISION: &str = "e2d6f346a37f92483b9f9a13923b98feb053447b";

fn reference() -> &'static Value {
    static REFERENCE: OnceLock<Value> = OnceLock::new();
    REFERENCE.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/applications/discovery-reference.json"
        )))
        .expect("frozen language reference")
    })
}

/// Describe the language and installed contracts. Transport admission is separate.
/// The source reference names MCP operations without asserting a running server.
pub fn describe_language(registry: &MethodRegistry) -> Result<Value, String> {
    let mut guide = reference()["language"].clone();
    guide["implementation_version"] = json!(env!("CARGO_PKG_VERSION"));
    guide["method_registry_fingerprint"] = json!(registry.fingerprint());
    guide["typed_bindings"] = propositions::describe_bindings(registry);
    guide["native_capabilities"] = json!({"language_reference":true,"authored_assessment":true,"registered_assessment":true,
        "mcp_transport_admission":"selected by the installed server and its operator configuration"});
    eal_core::canonical::canonical_json_bytes(&guide, &ExecutionLimits::default())?;
    Ok(guide)
}
pub fn describe_default() -> Result<Value, String> {
    describe_language(methods::default_registry())
}
