//! Native method metadata remains separate from authored prose and graph acceptance.
#[path = "../../../tests/native_case.rs"]
mod native_case;
use eal_aspic::{CompileOptions, compile_eal_aspic, export_aspic_view};
use eal_core::ExecutionLimits;
use serde_json::{Value, json};

fn snapshot() -> Value {
    let case = native_case::case();
    compile_eal_aspic(
        native_case::SOURCE,
        &case["records"],
        CompileOptions::new("working", native_case::NOW, &case["context"]),
    )
    .unwrap()
    .to_dict()
    .unwrap()
}

#[test]
fn native_method_result_survives_compilation_and_view_export() {
    let input = snapshot();
    let computation = &input["source_map"]["arguments"]["support"]["computation"];
    assert_eq!(computation["method"], "structured/1");
    assert_eq!(computation["status"], "supported");
    assert!(computation["reasons"].is_array());
    assert!(computation["details"].is_object());
    assert_eq!(computation.as_object().unwrap().len(), 4);
    let view = export_aspic_view(&input, &ExecutionLimits::default()).unwrap();
    let argument = view["arguments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|argument| argument["origin"]["name"] == "support")
        .unwrap();
    assert_eq!(&argument["origin"]["computation"], computation);
    assert_eq!(view["validation"]["provenance"], "supplied");
    assert!(
        argument["origin"]["computation"]
            .get("prose_verified")
            .is_none()
    );
}

#[test]
fn old_source_maps_remain_valid_without_implied_method_checks() {
    let mut input = snapshot();
    for entry in input["source_map"]["arguments"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        entry.as_object_mut().unwrap().remove("computation");
    }
    let view = export_aspic_view(&input, &ExecutionLimits::default()).unwrap();
    assert!(
        view["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .all(|argument| { argument["origin"].get("computation").is_none() })
    );
}

#[test]
fn closed_computation_shape_rejects_invented_status_and_oversized_fields() {
    let original = snapshot();
    let computation = &original["source_map"]["arguments"]["support"]["computation"];
    for invalid in [
        json!({"method":"deductive/1","status":"verified","reasons":[],"details":{}}),
        json!({"method":"deductive/1","status":"supported","reasons":[],"details":{},"authenticated":true}),
        json!({"method":"deductive/1","status":"supported","reasons":[],"details":[]}),
        json!({"method":"m".repeat(100_001),"status":"supported","reasons":[],"details":{}}),
        json!({"method":"deductive/1","status":"supported","reasons":vec!["reason";100_001],"details":{}}),
    ] {
        let mut input = original.clone();
        input["source_map"]["arguments"]["support"]["computation"] = invalid;
        assert!(export_aspic_view(&input, &ExecutionLimits::default()).is_err());
    }
    for status in ["supported", "unsupported", "not_run", "incomplete"] {
        let mut input = original.clone();
        input["source_map"]["arguments"]["support"]["computation"] = computation.clone();
        input["source_map"]["arguments"]["support"]["computation"]["status"] = json!(status);
        // Source-map metadata is supplied: accepting its shape adds no truth claim.
        let view = export_aspic_view(&input, &ExecutionLimits::default()).unwrap();
        assert_eq!(view["validation"]["provenance"], "supplied");
    }
}

#[test]
fn unavailable_sources_retain_not_run_in_source_map_without_creating_an_argument() {
    let case = native_case::case();
    let input = compile_eal_aspic(
        native_case::SOURCE,
        &json!({}),
        CompileOptions::new("working", native_case::NOW, &case["context"]),
    )
    .unwrap()
    .to_dict()
    .unwrap();
    assert_eq!(
        input["source_map"]["arguments"]["support"]["computation"]["status"],
        "not_run"
    );
    let view = export_aspic_view(&input, &ExecutionLimits::default()).unwrap();
    assert!(
        view["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .all(|argument| argument["origin"]["name"] != "support")
    );
    assert_eq!(
        view["unavailable_evidence"][0]["availability_issues"],
        json!(["missing_observation"])
    );
}

#[test]
fn a_checked_zero_premise_theorem_has_no_fabricated_observation() {
    let context = json!({});
    let source = r#"language "EAL/4"
environment formal_scope {
 require true
}
reasoning tautology {
 method "deductive/1"
 rationale "Finite truth-table enumeration checks this declared propositional tautology."
}
claim complete_split {
 statement "For the declared Boolean p, p or not p holds."
 environment formal_scope
 proposition {
  subject "Declared Boolean p"
  quantity "proposition"
  unit "1"
  scope "Classical finite propositional truth tables"
  valid_from "2026-01-01T00:00:00Z"
  valid_until "2030-01-01T00:00:00Z"
  query {"premises":[],"conclusion":{"or":["p",{"not":"p"}]}}
  result entailed == true
 }
}
argument proof = [] via tautology => complete_split
"#;
    let input = compile_eal_aspic(
        source,
        &json!({}),
        CompileOptions::new("complete_split", native_case::NOW, &context),
    )
    .unwrap()
    .to_dict()
    .unwrap();
    assert_eq!(input["authored_claim_status"], "supported");
    let view = export_aspic_view(&input, &ExecutionLimits::default()).unwrap();
    let arguments = view["arguments"].as_array().unwrap();
    assert_eq!(arguments.len(), 2);
    assert!(
        arguments
            .iter()
            .all(|argument| argument["origin"].get("observation").is_none())
    );
    let proof = arguments
        .iter()
        .find(|argument| argument["origin"]["name"] == "proof")
        .unwrap();
    assert_eq!(proof["origin"]["computation"]["method"], "deductive/1");
    assert_eq!(proof["origin"]["computation"]["details"]["entailed"], true);
    let carrier = arguments
        .iter()
        .find(|argument| argument["origin"]["kind"] == "structural basis")
        .unwrap();
    assert!(
        carrier["origin"]["meaning"]
            .as_str()
            .unwrap()
            .contains("no observation")
    );
    assert_eq!(view["unavailable_evidence"], json!([]));
}
