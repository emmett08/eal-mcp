//! Current language syntax and operation guidance with installed method identities.
use eal_core::{
    ExecutionLimits,
    methods::{self, MethodRegistry},
    propositions,
};
use serde_json::{Value, json};
use std::sync::OnceLock;

fn guide() -> &'static Value {
    static GUIDE: OnceLock<Value> = OnceLock::new();
    GUIDE.get_or_init(|| {
        serde_json::from_str(include_str!("discovery/guide.json")).expect("current language guide")
    })
}

/// Describe the language and installed contracts. Transport admission is separate.
/// Listed MCP operations describe the interface available through configured servers.
pub fn describe_language(registry: &MethodRegistry) -> Result<Value, String> {
    let mut guide = guide().clone();
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
