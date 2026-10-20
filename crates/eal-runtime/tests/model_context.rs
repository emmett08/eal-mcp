use eal_runtime::model_context::{CONTEXT_INSTRUCTION, ModelContextBuilder};
use serde_json::{Value, json};

#[test]
fn pinned_contexts_preserve_decisions_and_qualifications() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/applications/packets-reference.json"
    ))
    .unwrap();
    assert_eq!(CONTEXT_INSTRUCTION, fixture["context_instruction"]);
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c.get("model_context").is_some())
    {
        let expected = &case["model_context"];
        let registered = json!({"packet": case["packet"], "assessed_at": expected["assessed_at"], "assessment_id": expected["assessment_id"], "claim": expected["claim_id"], "status": expected["claim_status"]});
        let original = registered.clone();
        let actual = ModelContextBuilder::new()
            .build(&registered)
            .unwrap_or_else(|e| panic!("{}: {e}", case["name"]));
        assert_eq!(actual, *expected, "{}", case["name"]);
        assert_eq!(registered, original);
    }
}

#[test]
fn forged_raw_fields_and_mismatched_metadata_are_rejected() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/applications/packets-reference.json"
    ))
    .unwrap();
    let mut registered = json!({"packet": fixture["cases"][0]["packet"], "assessed_at": "2026-09-28T10:00:00Z", "assessment_id": "assessment-123", "claim": "pressure", "status": "contested"});
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
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/applications/packets-reference.json"
    ))
    .unwrap();
    let registered = json!({"packet": fixture["cases"][0]["packet"], "assessed_at": "2026-09-28T10:00:00Z", "assessment_id": "assessment-123", "claim": "pressure", "status": "contested"});
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
