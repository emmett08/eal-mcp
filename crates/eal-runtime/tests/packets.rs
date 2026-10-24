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

#[test]
fn exhausted_native_assessment_preserves_incomplete_through_packet_context_and_fallback() {
    use eal_core::{
        ExecutionLimits, canonical, composition, evaluation,
        methods::{MethodContract, MethodError, NativeMethodIdentity, default_registry},
    };
    use eal_runtime::model_context::ModelContextBuilder;
    const SOURCE: &str = "language \"EAL/3\"\nenvironment lab {\n require true\n}\ntool instrument {\n version \"1\"\n}\nevidence sample {\n tool instrument\n kind bounded\n environment lab\n max_age 60\n input {}\n require needed == 17\n}\nreasoning calculation {\n method \"example/bounded/1\"\n rationale \"The calculation supports this bounded result when completed.\"\n require completed == true\n}\nclaim bounded_result {\n statement \"The calculation establishes the requested result.\"\n environment lab\n}\nargument calculation_route = [evidence sample] via calculation => bounded_result\n";
    const NOW: &str = "2026-10-24T12:00:00Z";
    let limits = ExecutionLimits {
        formal_arguments: 16,
        ..Default::default()
    };
    let contract = MethodContract::native(
        "example/bounded/1",
        "bounded",
        json!({"type":"object","properties":{"needed":{"type":"integer"}},"required":["needed"],"additionalProperties":false}),
        json!({"type":"object","properties":{},"additionalProperties":false}),
        json!({"type":"object","properties":{"completed":{"type":"boolean"}},"required":["completed"],"additionalProperties":false}),
        NativeMethodIdentity {
            version: "bounded-test/1".into(),
            artifact_sha256: "d".repeat(64),
        },
        |input, limits| {
            if input["needed"].as_u64().unwrap() > limits.formal_arguments as u64 {
                Err(MethodError::Incomplete(
                    "enumeration budget exhausted".into(),
                ))
            } else {
                Ok(json!({"completed":true}))
            }
        },
    );
    let registry = default_registry().with_method(contract).unwrap();
    let program = composition::compile(SOURCE, None, "", limits).unwrap();
    let context = json!({});
    let input = json!({});
    let value = json!({"needed":17});
    let request = json!({"evidence_id":"sample","environment":"lab","tool":"instrument","tool_version":"1","input":input,"context":context});
    let acquisition =
        json!({"tool":"instrument","tool_version":"1","input":input,"context":context});
    let digest = |value: &Value| canonical::canonical_digest(value, &program.limits).unwrap();
    let record = json!({"evidence_id":"sample","environment":"lab","tool":"instrument","tool_version":"1","source_digest":program.source_digest,"tool_binding_digest":"0".repeat(64),"evidence_kind":"bounded","environment_fingerprint":evaluation::environment_fingerprint("lab",&context,&program.limits).unwrap(),"input_digest":digest(&input),"input":input,"context":context,"acquisition_request_digest":digest(&acquisition),"acquisition_request":acquisition,"request_digest":digest(&request),"collected_at":NOW,"run_id":"bounded-observation","status":"ok","data_digest":digest(&value),"value":value});
    let mut assessment = evaluation::evaluate(
        &program,
        &json!({"sample":record}),
        NOW,
        &context,
        &registry,
        None,
    )
    .unwrap();
    assert_eq!(assessment["valid"], true, "{assessment}");
    assert_eq!(assessment["complete"], false);
    assert_eq!(
        assessment["arguments"]["calculation_route"]["reasoning_result"]["status"],
        "incomplete"
    );
    assessment["assessment_id"] = json!("bounded-assessment");
    let builder =
        AssessmentPacketBuilder::new(PacketLimits::default(), Some(registry.clone())).unwrap();
    let packet = builder.build(&assessment, None, None).unwrap();
    assert_eq!(packet["schema"], "EAL/assessment-packet/3");
    assert_eq!(packet["complete"], false);
    assert_eq!(
        packet["claims"]["bounded_result"]["assessment_complete"],
        false
    );
    assert_eq!(
        packet["arguments"]["calculation_route"]["method_result"]["status"],
        "incomplete"
    );
    let wrap = |packet: Value| json!({"assessment_id":"bounded-assessment","assessed_at":NOW,"claim":"bounded_result","status":assessment["claims"]["bounded_result"]["status"],"packet":packet});
    let model = ModelContextBuilder::new()
        .build(&wrap(packet.clone()))
        .unwrap();
    assert_eq!(model["complete"], false);
    assert_eq!(
        model["arguments"]["calculation_route"]["method_result"]["status"],
        "incomplete"
    );
    let mut contradictory = assessment.clone();
    contradictory["complete"] = json!(true);
    assert!(builder.build(&contradictory, None, None).is_err());
    for corruption in ["schema", "complete", "marker", "state"] {
        let mut malformed = packet.clone();
        match corruption {
            "schema" => malformed["schema"] = json!("EAL/assessment-packet/99"),
            "complete" => malformed["complete"] = json!(true),
            "marker" => {
                malformed["claims"]["bounded_result"]
                    .as_object_mut()
                    .unwrap()
                    .remove("assessment_complete");
            }
            _ => {
                malformed["arguments"]["calculation_route"]["method_result"]["status"] =
                    json!("uncontrolled")
            }
        }
        assert!(
            ModelContextBuilder::new().build(&wrap(malformed)).is_err(),
            "{corruption}"
        );
    }
    // Independently supplied long authored statements exercise the projection's
    // fallback without changing the computation's incomplete outcome.
    let mut large = assessment.clone();
    for index in 0..8 {
        large["claims"][format!("additional_{index}")] =
            json!({"status":"unsupported","statement":"Long authored qualification. ".repeat(150)});
    }
    let small = AssessmentPacketBuilder::new(
        PacketLimits {
            max_bytes: 4096,
            ..Default::default()
        },
        Some(registry),
    )
    .unwrap()
    .build(&large, None, None)
    .unwrap();
    assert_eq!(small["omitted"]["details"], "packet_byte_limit");
    assert_eq!(small["complete"], false);
    assert_eq!(
        small["claims"]["bounded_result"]["assessment_complete"],
        false
    );
    assert_eq!(
        ModelContextBuilder::new().build(&wrap(small)).unwrap()["complete"],
        false
    );
}
