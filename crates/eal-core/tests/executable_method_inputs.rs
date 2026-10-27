//! Authored formulae are checked program data, never manufactured observations.
use eal_core::{ExecutionLimits, methods};
use serde_json::json;

#[test]
fn authored_entailment_has_no_observation_identity() {
    let result = methods::assess_input_with_limits(
        "deductive/1",
        &json!({"premises":["a","b"],"conclusion":{"and":["a","b"]}}),
        methods::default_registry(),
        &ExecutionLimits::default(),
    );
    assert_eq!(result["status"], "supported");
    assert_eq!(result["details"]["entailed"], true);
    assert_eq!(result["input_origin"], "authored_program");
    assert!(result.get("evidence_id").is_none());
    assert!(result.get("run_id").is_none());
}

#[test]
fn query_validation_rejects_semantic_errors_but_accepts_negative_findings() {
    let registry = methods::default_registry();
    assert!(
        methods::validate_query("inductive/1", &json!({"confidence":0.95}), registry).is_empty()
    );
    assert!(
        methods::validate_query("causal/1", &json!({"assignment":"randomised"}), registry)
            .is_empty()
    );
    assert!(
        methods::validate_query(
            "deductive/1",
            &json!({"premises":["a"],"conclusion":"b"}),
            registry
        )
        .is_empty()
    );
    assert!(
        !methods::validate_query(
            "deductive/1",
            &json!({"premises":[true],"conclusion":"b"}),
            registry
        )
        .is_empty()
    );
    assert!(!methods::validate_query("abductive/1", &json!({"observed":"x","candidates":[{"name":"a","prior":0.6,"likelihood":0.5},{"name":"b","prior":0.6,"likelihood":0.5}]}), registry).is_empty());
    assert!(!methods::validate_query("counterfactual/1", &json!({"variables":{"a":{"intercept":0,"noise":0,"coefficients":{"a":0}}},"intervention":{"variable":"a","value":1},"outcome":"a"}), registry).is_empty());
}

#[test]
fn imported_method_inputs_preserve_standalone_eal4_meaning() {
    use eal_core::{canonical, composition, evaluation, validation};
    const SOURCE: &str = include_str!("support/method-input.eal");
    let resolver = |_: &str| Ok(SOURCE.to_owned());
    let standalone =
        composition::compile(SOURCE, None, "sample.eal", ExecutionLimits::default()).unwrap();
    let imported = composition::compile(
        "language \"EAL/4\"\nimport \"sample.eal\" as library\n",
        Some(&resolver),
        "root.eal",
        ExecutionLimits::default(),
    )
    .unwrap();
    for (program, prefix) in [(standalone, ""), (imported, "library.")] {
        assert!(validation::validate_default(&program).is_empty());
        let evidence_id = format!("{prefix}sample");
        let evidence = &program.evidence[&evidence_id];
        let tool = &program.tools[evidence.tool.as_ref().unwrap()];
        let context = json!({});
        let value = json!({"successes":90,"trials":100});
        let acquisition = json!({"tool":tool.name,"tool_version":tool.version,"input":evidence.input,"context":context});
        let request = json!({"evidence_id":evidence_id,"environment":evidence.environment,"tool":tool.name,"tool_version":tool.version,"input":evidence.input,"context":context});
        let digest = |value: &serde_json::Value| {
            canonical::canonical_digest(value, &program.limits).unwrap()
        };
        let record = json!({"evidence_id":evidence_id,"source_digest":program.source_digest,
            "tool":tool.name,"tool_version":tool.version,"tool_binding_digest":"0".repeat(64),
            "evidence_kind":evidence.kind,"environment":evidence.environment,
            "environment_fingerprint":evaluation::environment_fingerprint(&evidence.environment,&context,&program.limits).unwrap(),
            "input_digest":digest(&evidence.input),"input":evidence.input,"context":context,
            "acquisition_request_digest":digest(&acquisition),"acquisition_request":acquisition,
            "request_digest":digest(&request),"collected_at":"2026-10-27T12:00:00Z","run_id":"independent-method-input-case",
            "status":"ok","data_digest":digest(&value),"value":value});
        let mut records = json!({});
        records[&evidence_id] = record;
        let result =
            evaluation::evaluate_default(&program, &records, "2026-10-27T12:00:01Z", &context)
                .unwrap();
        assert_eq!(
            result["claims"][format!("{prefix}result")]["status"],
            "supported",
            "{result}"
        );
        let lower =
            result["arguments"][format!("{prefix}proof")]["reasoning_result"]["details"]["lower"]
                .as_f64()
                .unwrap();
        assert!((lower - 0.8256343384950865).abs() < 1e-8);
    }
}
