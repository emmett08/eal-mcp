use eal_runtime::packets::{AssessmentPacketBuilder, PacketLimits};
use serde_json::{Value, json};

fn reference() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/applications/packets-reference.json"
    ))
    .unwrap()
}

#[test]
fn pinned_packet_projections_match_source_exactly() {
    let fixture = reference();
    let cases = fixture["cases"].as_array().unwrap();
    assert!(cases.len() >= 50);
    for case in cases {
        let limits = serde_json::from_value::<PacketLimits>(case["limits"].clone()).unwrap();
        let claims = case["claims"].as_array().map(|a| {
            a.iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
        });
        let collection = case.get("collection").filter(|v| !v.is_null());
        let result = AssessmentPacketBuilder::new(limits, None)
            .and_then(|builder| builder.build(&case["assessment"], claims.as_deref(), collection));
        match (result, case.get("error")) {
            (Ok(actual), None) => assert_eq!(actual, case["packet"], "{}", case["name"]),
            (Err(actual), Some(expected)) => {
                assert_eq!(actual, expected.as_str().unwrap(), "{}", case["name"])
            }
            (actual, expected) => panic!("{}: {actual:?} != {expected:?}", case["name"]),
        }
    }
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
    let mut assessment = reference()["cases"][0]["assessment"].clone();
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
