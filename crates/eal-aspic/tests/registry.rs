//! Current method contract validation and explicit bounded installation of native ASPIC.
use eal_aspic::registry::{
    ASPIC_METHOD, aspic_contract, aspic_registry, module_identity, output_schema, theory_schema,
    with_aspic,
};
use eal_core::{
    ExecutionLimits,
    methods::{assess_mode, assess_mode_with_limits, default_registry, engineering_registry},
};
use serde_json::{Value, json};

#[test]
fn current_aspic_contract_describes_native_identity_and_typed_query() {
    let actual = aspic_contract().describe();
    assert_eq!(actual["identifier"], ASPIC_METHOD);
    assert_eq!(actual["evidence_kind"], "aspic_theory");
    assert_eq!(
        theory_schema(),
        &actual["input_schema"]["properties"]["theory"]
    );
    assert_eq!(output_schema(), &actual["output_schema"]);
    assert_eq!(aspic_contract().query_fields, ["theory"]);
    assert_eq!(
        aspic_contract().output_type("query_accepted"),
        Some("boolean")
    );
    let identity = module_identity();
    assert_eq!(identity.len(), 64);
    assert!(
        identity
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    );
    assert_eq!(actual["implementation_artifact_digest"], identity);
    assert!(
        actual["implementation_version"]
            .as_str()
            .unwrap()
            .starts_with("eal-rust-aspic-finite-")
    );
    assert!(actual["implementation_source_digest"].is_null());
    assert!(actual["implementation_code_digest"].is_null());
    assert_eq!(actual["execution"], "trusted_native_callback");
}

#[test]
fn installation_is_explicit_immutable_and_combines_engineering_methods() {
    assert!(default_registry().get(ASPIC_METHOD).is_none());
    let registry = aspic_registry();
    assert_eq!(registry.describe().as_object().unwrap().len(), 9);
    let engineering = engineering_registry();
    let combined = with_aspic(&engineering).unwrap();
    assert_eq!(combined.describe().as_object().unwrap().len(), 13);
    assert!(engineering.get(ASPIC_METHOD).is_none());
    assert!(combined.get("engineering/rms/1").is_some());
    assert!(with_aspic(&combined).is_err());
    assert_ne!(registry.fingerprint(), default_registry().fingerprint());
}

fn payload(goal: &str) -> Value {
    json!({"theory":{"premises":[{"atom":"p","kind":"axiom"}],"rules":[],"contraries":[],"goal":goal}})
}
fn evidence(payload: Value) -> [Value; 1] {
    [json!({"id":"theory","kind":"aspic_theory","value":payload})]
}

#[test]
fn registered_solver_keeps_negative_results_and_explicit_budget_exhaustion() {
    let registry = aspic_registry();
    let positive = assess_mode(ASPIC_METHOD, &evidence(payload("p")), &[], &registry);
    assert_eq!(positive["status"], "supported", "{positive}");
    assert_eq!(positive["details"]["query_accepted"], true);
    let negative = assess_mode(ASPIC_METHOD, &evidence(payload("q")), &[], &registry);
    assert_eq!(negative["status"], "supported", "{negative}");
    assert_eq!(negative["details"]["query_status"], "unconstructed");
    assert_eq!(negative["details"]["query_accepted"], false);
    let mut input = payload("p");
    input["theory"]["premises"]
        .as_array_mut()
        .unwrap()
        .push(json!({"atom":"q","kind":"axiom"}));
    let limits = ExecutionLimits {
        formal_arguments: 1,
        ..Default::default()
    };
    let incomplete =
        assess_mode_with_limits(ASPIC_METHOD, &evidence(input), &[], &registry, &limits);
    assert_eq!(incomplete["status"], "incomplete", "{incomplete}");
    assert_eq!(incomplete["details"], json!({}));
    assert_eq!(incomplete["method_contract"]["identifier"], ASPIC_METHOD);
    assert_eq!(incomplete["evidence_id"], "theory");
    let invalid = assess_mode(ASPIC_METHOD, &evidence(payload("bad atom")), &[], &registry);
    assert_eq!(invalid["status"], "unsupported", "{invalid}");
    assert_eq!(invalid["details"], json!({}));
}
