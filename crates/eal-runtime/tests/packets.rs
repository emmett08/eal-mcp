use eal_runtime::packets::{AssessmentPacketBuilder, PacketLimits};
use serde_json::{Value, json};

fn assessment() -> Value {
    json!({"valid":true,"complete":true,"assessment_id":"stored","claims":{"pressure":{"status":"contested","statement":"The measured pressure meets the declared threshold.","supporting_arguments":[],"contested_arguments":["experiment"]}},
        "arguments":{"experiment":{"status":"contested","conclusion":"pressure","reasoning_result":{"status":"supported","method":"structured/1","details":{"authored":true}},"dependencies":{"evidence":["trial"],"assumptions":[],"premises":[]}}},
        "evidence":{"trial":{"status":"available","availability_issues":[]}}})
}

#[test]
fn ignored_raw_payloads_cannot_evade_depth_or_finite_admission() {
    let mut assessment = json!({"valid": true, "assessment_id": "stored", "claims": {"c": {"status": "unsupported", "statement": "Bounded authored proposition."}}});
    let mut deep = Value::Null;
    for _ in 0..130 {
        deep = json!([deep]);
    }
    assessment["unprojected"] = deep;
    assert!(
        AssessmentPacketBuilder::default()
            .build(&assessment, None, None)
            .unwrap_err()
            .contains("resource limit")
    );
    assessment["unprojected"] = serde_json::from_str("1e9999").unwrap();
    assert!(
        AssessmentPacketBuilder::default()
            .build(&assessment, None, None)
            .unwrap_err()
            .contains("finite")
    );
}

#[test]
fn packet_limits_validate_even_when_constructed_without_deserialization() {
    assert!(
        AssessmentPacketBuilder::new(
            PacketLimits {
                max_outputs: 0,
                ..Default::default()
            },
            None
        )
        .is_err()
    );
    assert!(
        AssessmentPacketBuilder::new(
            PacketLimits {
                max_bytes: 4096,
                max_statement_bytes: 4096,
                ..Default::default()
            },
            None
        )
        .is_ok()
    );
}

#[test]
fn extension_outputs_require_installed_contract_and_dynamic_maps_stay_private() {
    let mut assessment = assessment();
    assessment["arguments"]["experiment"]["reasoning_result"]["method"] =
        json!("engineering/rms/1");
    assessment["arguments"]["experiment"]["reasoning_result"]["details"] =
        json!({"rms": 0, "private": "secret", "sample_count": 5});
    let claims = vec!["pressure".to_owned()];
    let no_registry = AssessmentPacketBuilder::default()
        .build(&assessment, Some(&claims), None)
        .unwrap();
    assert!(
        no_registry["arguments"]["experiment"]["method_result"]
            .get("outputs")
            .is_none()
    );
    let registry = eal_core::methods::engineering_registry();
    let with_registry = AssessmentPacketBuilder::new(PacketLimits::default(), Some(registry))
        .unwrap()
        .build(&assessment, Some(&claims), None)
        .unwrap();
    assert_eq!(
        with_registry["arguments"]["experiment"]["method_result"]["outputs"]["rms"],
        0
    );
    assert!(!with_registry.to_string().contains("secret"));
}
