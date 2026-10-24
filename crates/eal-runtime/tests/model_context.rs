use eal_runtime::model_context::ModelContextBuilder;
use serde_json::{Value, json};

fn registered() -> Value {
    json!({"assessed_at":"2026-09-28T10:00:00Z","assessment_id":"assessment-123","claim":"pressure","status":"contested","packet":{
        "assessment_id":"assessment-123","claims":{"pressure":{"status":"contested","statement":"The measured pressure meets the declared threshold.","statement_truncated":false,"prose_verified":false,"typed_result":{"path":"pressure","operator":"==","expected":1,"unit":null}}},
        "arguments":{"experiment":{"status":"contested","conclusion":"pressure","premises":[],"method_result":{"status":"supported","method":"structured/1","binding":{"status":"bound","evidence_id":"trial","actual":1}}}},
        "assumptions":{"calibration":{"status":"supported","validation":"trial"}},"evidence":{"trial":{"status":"unavailable","availability_issues":["predicate_not_met"]}},"summary_complete":true,"omitted":{}}})
}
#[test]
fn native_context_preserves_selected_claim_status_and_qualification() {
    let assessment = registered();
    let original = assessment.clone();
    let context = ModelContextBuilder::new().build(&assessment).unwrap();
    assert_eq!(context["claim_id"], "pressure");
    assert_eq!(context["claim_status"], "contested");
    assert_eq!(context["claims"]["pressure"]["prose_verified"], false);
    assert_eq!(context["evidence"]["trial"]["requirements"], "not_met");
    assert_eq!(assessment, original);
}

#[test]
fn forged_raw_fields_and_mismatched_metadata_are_rejected() {
    let mut registered = registered();
    registered["packet"]["evidence"]["trial"]["value"] = json!("raw-secret");
    assert!(ModelContextBuilder::new().build(&registered).is_err());
    registered["packet"]["evidence"]["trial"]
        .as_object_mut()
        .unwrap()
        .remove("value");
    registered["status"] = json!("supported");
    assert!(
        ModelContextBuilder::new()
            .build(&registered)
            .unwrap_err()
            .contains("status differs")
    );
}

#[test]
fn nested_projection_fields_cannot_carry_raw_payloads_or_false_qualification() {
    let registered = registered();
    for pointer in [
        "/packet/arguments/experiment/method_result/binding/actual",
        "/packet/claims/pressure/typed_result/expected",
        "/packet/assumptions/calibration/validation",
        "/packet/evidence/trial/availability_issues",
        "/packet/omitted",
    ] {
        let mut forged = registered.clone();
        *forged.pointer_mut(pointer).unwrap() = json!({"raw": "PRIVATE_PAYLOAD"});
        assert!(
            ModelContextBuilder::new().build(&forged).is_err(),
            "{pointer}"
        );
    }
    let mut forged = registered;
    forged["packet"]["claims"]["pressure"]["statement_truncated"] = json!(true);
    forged["packet"]["claims"]["pressure"]["prose_verified"] = json!(true);
    assert!(
        ModelContextBuilder::new()
            .build(&forged)
            .unwrap_err()
            .contains("complete statement")
    );
}

#[test]
fn negative_predicate_is_distinct_from_missing_or_stale_measurement() {
    let registered = json!({"assessment_id":"stored", "assessed_at":"2026-09-28T10:00:00Z", "claim":"c", "status":"unsupported", "packet": {
        "claims": {"c": {"status": "unsupported"}}, "evidence": {
            "pass": {"status":"available", "availability_issues":[]},
            "negative": {"status":"unavailable", "availability_issues":["predicate_not_met"]},
            "stale_negative": {"status":"unavailable", "availability_issues":["predicate_not_met","stale_observation"]},
            "missing": {"status":"unavailable", "availability_issues":["missing_observation"]}}, "summary_complete": false, "omitted": {"details":"packet_byte_limit"}}});
    let context = ModelContextBuilder::new().build(&registered).unwrap();
    assert_eq!(context["evidence"]["pass"]["requirements"], "met");
    assert_eq!(context["evidence"]["negative"]["requirements"], "not_met");
    assert_eq!(
        context["evidence"]["stale_negative"]["requirements"],
        "unresolved"
    );
    assert_eq!(context["evidence"]["missing"]["requirements"], "unresolved");
}
