use eal_core::{ExecutionLimits, composition, methods, validation};
use eal_runtime::discovery;
use serde_json::json;

#[test]
fn language_guide_retains_executable_example_and_native_registry_identity() {
    let guide = discovery::describe_default().unwrap();
    let source = guide["example"].as_str().unwrap();
    let program = composition::compile(source, None, "", ExecutionLimits::default()).unwrap();
    assert!(validation::validate(&program, methods::default_registry()).is_empty());
    assert_eq!(
        guide["method_registry_fingerprint"],
        methods::default_registry().fingerprint()
    );
    assert_eq!(guide["languages"], json!(["EAL/3"]));
    assert_eq!(guide["implementation_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        guide["registered_workflow"],
        json!([
            "eal_sources",
            "eal_find_claims",
            "eal_assess_known",
            "eal_get_known_packet"
        ])
    );
    assert!(
        guide["native_capabilities"]["mcp_transport_admission"]
            .as_str()
            .unwrap()
            .contains("operator configuration")
    );
    let optional = discovery::describe_language(&methods::engineering_registry()).unwrap();
    assert!(
        optional["typed_bindings"]["methods"]
            .get("engineering/rms/1")
            .is_some()
    );
}
