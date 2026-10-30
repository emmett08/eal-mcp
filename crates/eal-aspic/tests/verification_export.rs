//! Verification provenance survives export independently of graph construction.
#[path = "../../../tests/native_case.rs"]
mod native_case;
use eal_aspic::{CompileOptions, compile_eal_aspic, export_aspic_view};
use eal_core::ExecutionLimits;
use serde_json::{Value, json};

fn unavailable_snapshot() -> Value {
    let case = native_case::case();
    compile_eal_aspic(
        native_case::SOURCE,
        &json!({}),
        CompileOptions::new("working", native_case::NOW, &case["context"]),
    )
    .unwrap()
    .to_dict()
    .unwrap()
}

fn supplied_report() -> Value {
    json!({
        "schema":"eal.verification-report/1","target_id":"working",
        "mode":"empirical","status":"unknown",
        "obligations":[{
            "id":"calibration","role":"applicability","status":"unknown",
            "reasons":["No calibration record supplied."],"details":{"observation_id":"probe"}
        }],
        "atoms":[{"symbol":"p","observation_id":"probe","status":"unknown",
            "truth":null,"reasons":["No observation supplied."],"details":{}}],
        "formulas":[]
    })
}

#[test]
fn supplied_unresolved_report_remains_visible_without_a_constructed_argument() {
    let mut input = unavailable_snapshot();
    let report = supplied_report();
    input["source_map"]["arguments"]["support"]["computation"] = json!({
        "method":"grounded/1","status":"supported","reasons":[],
        "details":{"verified":false,"verification":report}
    });
    let view = export_aspic_view(&input, &ExecutionLimits::default()).unwrap();
    assert!(
        view["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .all(|a| a["origin"]["name"] != "support")
    );
    assert_eq!(
        view["verification_reports"],
        json!([{"argument_name":"support","source":"native_method","method_status":"supported","reasons":[],"report":report}])
    );
    assert_eq!(view["validation"]["provenance"], "supplied");
}

#[test]
fn old_snapshots_do_not_gain_an_implied_verification_result() {
    let view = export_aspic_view(&unavailable_snapshot(), &ExecutionLimits::default()).unwrap();
    assert!(view.get("verification_reports").is_none());
}

#[test]
fn skipped_grounding_stays_visible_without_fabricated_obligation_results() {
    let mut input = unavailable_snapshot();
    input["source_map"]["arguments"]["support"]["computation"] = json!({
        "method":"grounded/1","status":"not_run","reasons":["Source unavailable"],"details":{}
    });
    let view = export_aspic_view(&input, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        view["verification_reports"],
        json!([{
            "argument_name":"support","source":"native_method","method_status":"not_run","reasons":["Source unavailable"],"report":null
        }])
    );
}

#[test]
fn malformed_recorded_verification_is_rejected_at_the_export_boundary() {
    for (field, value) in [
        ("status", json!("verified")),
        ("status", json!("not_required")),
        ("mode", json!("universal_truth")),
        ("schema", json!("eal.verification-report/2")),
        ("authenticated", json!(true)),
    ] {
        let mut input = unavailable_snapshot();
        let mut report = supplied_report();
        report[field] = value;
        input["source_map"]["arguments"]["support"]["computation"] = json!({
            "method":"grounded/1","status":"supported","reasons":[],
            "details":{"verified":false,"verification":report}
        });
        assert!(
            export_aspic_view(&input, &ExecutionLimits::default()).is_err(),
            "{field}"
        );
    }
}

#[test]
fn native_undefined_symbol_report_survives_omission_of_its_formal_route() {
    let query = json!({
        "coverage":{"target_id":"grounded_claim","mode":"empirical",
            "formulas":[{"id":"premise","formula":"p"}],"required_obligations":[]},
        "contract":{"schema":"eal.verification-contract/1","target_id":"grounded_claim",
            "mode":"empirical","atoms":[],"obligations":[],"applicability":[]}
    });
    let source = format!(
        r#"language "EAL/4"
environment lab {{
 require true
}}
tool fixture_reader {{
 version "1"
}}
evidence verification_input {{
 tool fixture_reader
 kind verification_case
 environment lab
 max_age 60
 require schema == "EAL/typed-input/1"
}}
reasoning verify_grounding {{
 method "grounded/1"
 rationale "Check the declared synthetic predicate grounding."
}}
claim grounded_claim {{
 statement "The supplied predicate p is grounded."
 environment lab
 proposition {{
  subject "synthetic verification"
  quantity "proposition"
  unit "1"
  scope "synthetic case"
  valid_from "2026-10-24T11:00:00Z"
  valid_until "2026-10-24T13:00:00Z"
  query {query}
  result verified == true
 }}
}}
argument checked_route = [evidence verification_input] via verify_grounding => grounded_claim binding verification_input
"#
    );
    let context = json!({});
    let value = json!({
        "schema":"EAL/typed-input/1","method":"grounded/1","subject":"synthetic verification",
        "quantity":"proposition","unit":"1","scope":"synthetic case",
        "valid_from":"2026-10-24T11:00:00Z","valid_until":"2026-10-24T13:00:00Z",
        "payload":{"coverage":query["coverage"],"contract":query["contract"],"observations":{},"assessed_at":native_case::NOW}
    });
    let record = native_case::observation(&source, "verification_input", &context, value);
    let compiled = compile_eal_aspic(
        &source,
        &json!({"verification_input":record}),
        CompileOptions::new("grounded_claim", native_case::NOW, &context),
    )
    .unwrap()
    .to_dict()
    .unwrap();
    let view = export_aspic_view(&compiled, &ExecutionLimits::default()).unwrap();
    assert!(
        view["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .all(|argument| argument["origin"]["name"] != "checked_route")
    );
    let record = &view["verification_reports"][0];
    assert_eq!(record["source"], "native_method");
    assert_eq!(record["method_status"], "supported");
    assert_eq!(record["report"]["status"], "unknown");
    assert_eq!(record["report"]["atoms"][0]["observation_id"], Value::Null);
    assert_eq!(record["report"]["atoms"][0]["truth"], Value::Null);
}
