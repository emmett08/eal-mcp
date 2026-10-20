use eal_core::{ExecutionLimits, composition, methods, validation};
use eal_runtime::discovery;
use serde_json::{Value, json};

#[test]
fn pinned_language_guide_retains_executable_example_and_native_registry_identity() {
    let mut guide = discovery::describe_default().unwrap();
    let source = guide["example"].as_str().unwrap();
    let program = composition::compile(source, None, "", ExecutionLimits::default()).unwrap();
    assert!(validation::validate(&program, methods::default_registry()).is_empty());
    assert_eq!(
        guide["method_registry_fingerprint"],
        methods::default_registry().fingerprint()
    );
    assert_eq!(guide["languages"], json!(["EAL/3"]));
    assert!(
        guide["native_capabilities"]["mcp_transport_admission"]
            .as_str()
            .unwrap()
            .contains("operator configuration")
    );
    for key in [
        "implementation_version",
        "method_registry_fingerprint",
        "typed_bindings",
        "native_capabilities",
    ] {
        guide.as_object_mut().unwrap().remove(key);
    }
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/applications/discovery-reference.json"
    ))
    .unwrap();
    assert_eq!(guide, fixture["language"]);
    let optional = discovery::describe_language(&methods::engineering_registry()).unwrap();
    assert!(
        optional["typed_bindings"]["methods"]
            .get("engineering/rms/1")
            .is_some()
    );
}
