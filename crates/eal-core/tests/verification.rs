//! Independently specified observations distinguish denotation from entailment.
use eal_core::{
    ExecutionLimits, methods,
    verification::{
        self, Context, Contract, Coverage, Mode, ObservationStatus, Query, Role, Status,
    },
};
use serde_json::{Value, json};

fn query() -> Value {
    json!({
        "coverage":{"target_id":"output","mode":"empirical","formulas":[{"id":"measured","formula":"p"},{"id":"rule","formula":{"implies":["p","q"]}}],"required_obligations":[{"id":"source_check","role":"premise"}]},
        "contract":{"schema":"eal.verification-contract/1","target_id":"output","mode":"empirical",
            "atoms":[{"symbol":"p","observation_id":"meter","predicate":{"path":"measurement.count","operator":"==","expected":3,"expression":null}},{"symbol":"q","observation_id":"meter","predicate":{"path":"measurement.distinct","operator":"==","expected":true,"expression":null}}],
            "obligations":[{"id":"source_check","role":"premise","observation_id":"meter","predicate":{"path":"measurement.count","operator":"==","expected":3,"expression":null}}],
            "applicability":[{"observation_id":"meter","identity_pointer":"/device","identity":"serial-A","unit_pointer":"/unit","unit":"1","scope_pointer":"/scope","scope":{"configuration":"v1"},"valid_from_pointer":"/from","valid_until_pointer":"/until","valid_from":"2026-10-30T00:00:00Z","valid_until":"2026-10-31T00:00:00Z","max_age_seconds":60}]},
        "observations":{"meter":{"status":"available","value":{"measurement":{"count":3,"distinct":true},"device":"serial-A","unit":"1","scope":{"configuration":"v1"},"from":"2026-10-30T00:00:00Z","until":"2026-10-31T00:00:00Z","secret":"unrelated-credential"},"observed_at":"2026-10-30T12:00:00Z","source_digest":"a".repeat(64)}},"assessed_at":"2026-10-30T12:00:01Z"})
}
fn assess(value: &Value) -> verification::Report {
    verification::evaluate_query(value, &ExecutionLimits::default()).unwrap()
}

#[test]
fn actual_source_predicates_ground_formulas_and_keep_private_fields_out_of_report() {
    let report = assess(&query());
    assert_eq!(report.status, Status::Pass);
    assert_eq!(
        report
            .atoms
            .iter()
            .map(|atom| atom.truth)
            .collect::<Vec<_>>(),
        vec![Some(true), Some(true)]
    );
    assert!(
        report
            .formulas
            .iter()
            .all(|formula| formula.truth == Some(true))
    );
    let text = serde_json::to_string(&report).unwrap();
    assert!(!text.contains("unrelated-credential"));
    assert!(!text.contains("\"secret\""));
    assert_eq!(
        report.atoms[0].details["read_fields"]["measurement.count"],
        3
    );
}
#[test]
fn identical_formula_and_accepted_wrapper_do_not_replace_missing_atom_meaning() {
    let mut value = query();
    value["contract"]["atoms"] = json!([]);
    value["observations"]["meter"]["value"]["accepted"] = json!(true);
    let report = assess(&value);
    assert_eq!(report.status, Status::Unknown);
    assert!(report.atoms.iter().all(|atom| atom.truth.is_none()));
    assert!(
        report
            .formulas
            .iter()
            .all(|formula| formula.status == Status::Unknown)
    );
}
#[test]
fn measured_false_is_known_and_refutes_the_formula_instead_of_becoming_missing_data() {
    let mut value = query();
    value["observations"]["meter"]["value"]["measurement"]["distinct"] = json!(false);
    let report = assess(&value);
    assert_eq!(report.status, Status::Fail);
    assert_eq!(report.atoms[1].truth, Some(false));
    assert_eq!(report.atoms[1].status, Status::Pass);
    assert_eq!(report.formulas[1].truth, Some(false));
    assert_eq!(report.formulas[1].status, Status::Fail);
}
#[test]
fn omissions_bad_sources_and_missing_facets_remain_unknown() {
    for mutation in 0..7 {
        let mut value = query();
        match mutation {
            0 => {
                value["contract"]["obligations"] = json!([]);
            }
            1 => {
                value["contract"]["applicability"] = json!([]);
            }
            2 => {
                value["observations"]["meter"]["status"] = json!("failed");
            }
            3 => {
                value["observations"]["meter"]["value"]
                    .as_object_mut()
                    .unwrap()
                    .remove("unit");
            }
            4 => {
                value["observations"]["meter"]["observed_at"] = Value::Null;
            }
            5 => {
                value["contract"]["applicability"][0]["identity"] = Value::Null;
                value["observations"]["meter"]["value"]["device"] = Value::Null;
            }
            _ => {
                value["observations"]["meter"]["value"]["scope"] = json!({"configuration":null});
            }
        }
        assert_eq!(
            assess(&value).status,
            Status::Unknown,
            "mutation {mutation}"
        );
    }
}
#[test]
fn every_used_source_requires_its_own_applicability() {
    let mut value = query();
    value["contract"]["atoms"][1]["observation_id"] = json!("other");
    value["observations"]["other"] = value["observations"]["meter"].clone();
    assert_eq!(assess(&value).status, Status::Unknown);
}
#[test]
fn wrong_identity_unit_scope_interval_and_original_clock_fail() {
    for (pointer, wrong) in [
        ("/observations/meter/value/device", json!("serial-B")),
        ("/observations/meter/value/unit", json!("kPa")),
        (
            "/observations/meter/value/scope",
            json!({"configuration":"v2"}),
        ),
        (
            "/observations/meter/value/from",
            json!("2026-10-30T11:00:00Z"),
        ),
        (
            "/observations/meter/observed_at",
            json!("2026-10-30T11:58:59.999Z"),
        ),
        (
            "/observations/meter/observed_at",
            json!("2026-10-29T23:59:59Z"),
        ),
        (
            "/observations/meter/observed_at",
            json!("2026-10-30T12:00:02Z"),
        ),
    ] {
        let mut value = query();
        *value.pointer_mut(pointer).unwrap() = wrong;
        assert_eq!(assess(&value).status, Status::Fail, "{pointer}");
    }
    let mut boundary = query();
    boundary["assessed_at"] = json!("2026-10-30T12:01:00Z");
    assert_eq!(assess(&boundary).status, Status::Pass);
    boundary["assessed_at"] = json!("2026-10-30T12:01:00.000001Z");
    assert_eq!(assess(&boundary).status, Status::Fail);
}
#[test]
fn mathematical_mode_does_not_require_observed_truth_of_hypothetical_atoms() {
    let mut query: Query = serde_json::from_value(query()).unwrap();
    query.coverage.mode = Mode::Mathematical;
    query.contract.mode = Mode::Mathematical;
    query.coverage.required_obligations.clear();
    query.contract.obligations.clear();
    query.contract.atoms.clear();
    query.contract.applicability.clear();
    query.observations.clear();
    let report = verification::evaluate(
        &query.coverage,
        &query.contract,
        &Context {
            observations: query.observations,
            assessed_at: query.assessed_at,
        },
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(report.status, Status::Pass);
    assert!(
        report
            .formulas
            .iter()
            .all(|formula| formula.status == Status::NotRequired)
    );
    assert!(
        report
            .obligations
            .iter()
            .any(|item| item.details["inference_checked"] == false)
    );
}
#[test]
fn defeasible_mode_requires_explicit_warrant_coverage() {
    let mut value = query();
    value["coverage"]["mode"] = json!("defeasible");
    value["contract"]["mode"] = json!("defeasible");
    assert_eq!(assess(&value).status, Status::Unknown);
    value["coverage"]["required_obligations"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"review","role":"warrant"}));
    value["contract"]["obligations"].as_array_mut().unwrap().push(json!({"id":"review","role":"warrant","observation_id":"meter","predicate":{"path":"measurement.distinct","operator":"==","expected":true,"expression":null}}));
    let report = assess(&value);
    assert_eq!(report.status, Status::Pass);
    assert!(
        report
            .obligations
            .iter()
            .any(|item| item.role == Role::Warrant && item.status == Status::Pass)
    );
}
#[test]
fn integers_are_compared_exactly_and_reports_preserve_large_values_as_exact_objects() {
    let mut value = query();
    let large: Value = serde_json::from_str("9007199254740993").unwrap();
    value["contract"]["atoms"][0]["predicate"]["expected"] = large.clone();
    value["contract"]["obligations"][0]["predicate"]["expected"] = large.clone();
    value["observations"]["meter"]["value"]["measurement"]["count"] = large;
    let report = assess(&value);
    assert_eq!(report.status, Status::Pass);
    assert_eq!(
        report.atoms[0].details["read_fields"]["measurement.count"]["numerator"],
        "9007199254740993"
    );
    value["observations"]["meter"]["value"]["measurement"]["count"] =
        serde_json::from_str("9007199254740992").unwrap();
    assert_eq!(assess(&value).status, Status::Fail);
}
#[test]
fn malformed_constant_predicates_duplicate_bindings_and_work_exhaustion_are_rejected() {
    let mut value = query();
    value["contract"]["atoms"][0]["predicate"] = json!({"path":"","operator":"expression","expected":null,"expression":{"kind":"literal","value":true}});
    assert!(verification::evaluate_query(&value, &ExecutionLimits::default()).is_err());
    assert!(
        !methods::validate_query(
            "grounded/1",
            &json!({"coverage":value["coverage"],"contract":value["contract"]}),
            methods::default_registry()
        )
        .is_empty()
    );
    let mut value = query();
    let atom = value["contract"]["atoms"][0].clone();
    value["contract"]["atoms"]
        .as_array_mut()
        .unwrap()
        .push(atom);
    assert!(verification::evaluate_query(&value, &ExecutionLimits::default()).is_err());
    let limits = ExecutionLimits {
        method_work: 1,
        ..Default::default()
    };
    assert!(verification::evaluate_query(&query(), &limits).is_err());
}
#[test]
fn native_method_preserves_unknown_versus_completed_negative_results() {
    let result = methods::assess_input_with_limits(
        "grounded/1",
        &query(),
        methods::default_registry(),
        &ExecutionLimits::default(),
    );
    assert_eq!(result["status"], "supported", "{result}");
    assert_eq!(result["details"]["verified"], true);
    let mut negative = query();
    negative["observations"]["meter"]["value"]["measurement"]["distinct"] = json!(false);
    let result = methods::assess_input_with_limits(
        "grounded/1",
        &negative,
        methods::default_registry(),
        &ExecutionLimits::default(),
    );
    assert_eq!(result["status"], "supported");
    assert_eq!(result["details"]["verified"], false);
    assert_eq!(result["details"]["verification"]["status"], "fail");
    negative["contract"]["atoms"] = json!([]);
    let result = methods::assess_input_with_limits(
        "grounded/1",
        &negative,
        methods::default_registry(),
        &ExecutionLimits::default(),
    );
    assert_eq!(result["status"], "supported");
    assert_eq!(result["details"]["verification"]["status"], "unknown");
}

#[test]
fn composition_requires_every_selected_report_and_binds_actual_children() {
    let report = assess(&query());
    let limits = ExecutionLimits::default();
    let composed = verification::compose(
        "root",
        Mode::Empirical,
        &["output".into()],
        std::slice::from_ref(&report),
        &limits,
    )
    .unwrap();
    assert_eq!(composed.status, Status::Pass);
    assert_eq!(
        composed.obligations[0].details["report_sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    let missing = verification::compose(
        "root",
        Mode::Empirical,
        &["output".into(), "other".into()],
        std::slice::from_ref(&report),
        &limits,
    )
    .unwrap();
    assert_eq!(missing.status, Status::Unknown);
    let ambiguous = verification::compose(
        "root",
        Mode::Empirical,
        &["output".into()],
        &[report.clone(), report],
        &limits,
    )
    .unwrap();
    assert_eq!(ambiguous.status, Status::Unknown);
    assert_eq!(
        verification::compose("root", Mode::Mathematical, &[], &[], &limits)
            .unwrap()
            .status,
        Status::Unknown
    );
}
#[test]
fn composition_preserves_measured_failure_and_rejects_duplicate_coverage() {
    let mut value = query();
    value["observations"]["meter"]["value"]["device"] = json!("wrong-device");
    let report = assess(&value);
    let limits = ExecutionLimits::default();
    assert_eq!(
        verification::compose(
            "root",
            Mode::Empirical,
            &["output".into()],
            std::slice::from_ref(&report),
            &limits
        )
        .unwrap()
        .status,
        Status::Fail
    );
    assert!(
        verification::compose(
            "root",
            Mode::Empirical,
            &["output".into(), "output".into()],
            &[report],
            &limits
        )
        .is_err()
    );
}

#[test]
fn native_argument_uses_actual_assessment_time_instead_of_payload_clock() {
    use eal_core::{canonical, composition, evaluation};
    let payload = query();
    let declared = json!({"coverage":payload["coverage"],"contract":payload["contract"]});
    let source = r#"language "EAL/4"
environment bench {
 require true
}
tool reader {
 version "1"
}
evidence reading {
 tool reader
 kind verification_case
 environment bench
 max_age 3600
 input {}
 require schema == "EAL/typed-input/1"
}
reasoning source_checks {
 method "grounded/1"
 rationale "Check actual synthetic observations against explicit denotations and applicability."
}
claim result {
 statement "The declared source grounding obligations hold."
 environment bench
 proposition {
  subject "synthetic records"
  quantity "proposition"
  unit "1"
  scope "fixed bench"
  valid_from "2026-10-30T00:00:00Z"
  valid_until "2026-10-31T00:00:00Z"
  query __QUERY__
  result verified == true
 }
}
argument verification = [evidence reading] via source_checks => result binding reading
"#
    .replace("__QUERY__", &declared.to_string());
    let program = composition::compile(&source, None, "", ExecutionLimits::default()).unwrap();
    let context = json!({});
    let evidence = &program.evidence["reading"];
    let value = json!({"schema":"EAL/typed-input/1","method":"grounded/1","subject":"synthetic records","quantity":"proposition","unit":"1","scope":"fixed bench","valid_from":"2026-10-30T00:00:00Z","valid_until":"2026-10-31T00:00:00Z","payload":payload});
    let acquisition = json!({"tool":"reader","tool_version":"1","input":{},"context":context});
    let request = json!({"evidence_id":"reading","environment":"bench","tool":"reader","tool_version":"1","input":{},"context":context});
    let digest = |value: &Value| canonical::canonical_digest(value, &program.limits).unwrap();
    let record = json!({"evidence_id":"reading","source_digest":program.source_digest,"tool":"reader","tool_version":"1","tool_binding_digest":"0".repeat(64),"evidence_kind":evidence.kind,"environment":"bench","environment_fingerprint":evaluation::environment_fingerprint("bench",&context,&program.limits).unwrap(),"input_digest":digest(&json!({})),"input":{},"context":context,"acquisition_request_digest":digest(&acquisition),"acquisition_request":acquisition,"request_digest":digest(&request),"collected_at":"2026-10-30T12:00:00Z","run_id":"grounding-source-fixture","status":"ok","data_digest":digest(&value),"value":value});
    let records = json!({"reading":record});
    let current =
        evaluation::evaluate_default(&program, &records, "2026-10-30T12:00:01Z", &context).unwrap();
    assert_eq!(
        current["claims"]["result"]["status"], "supported",
        "{current}"
    );
    assert_eq!(
        current["arguments"]["verification"]["reasoning_result"]["details"]["verification"]["atoms"]
            [0]["details"]["source_digest"],
        program.source_digest
    );
    // A fresh nested timestamp cannot refresh an old retained evidence
    // container, even while the looser outer EAL max_age still admits it.
    let mut old_container = records.clone();
    old_container["reading"]["collected_at"] = json!("2026-10-30T11:30:00Z");
    let capped =
        evaluation::evaluate_default(&program, &old_container, "2026-10-30T12:00:01Z", &context)
            .unwrap();
    assert_eq!(
        capped["evidence"]["reading"]["status"], "available",
        "{capped}"
    );
    assert_eq!(
        capped["claims"]["result"]["status"], "unsupported",
        "{capped}"
    );
    assert!(capped["arguments"]["verification"]["reasoning_result"]["details"]["verification"]["obligations"].as_array().unwrap().iter().any(|obligation|obligation["details"]["observed_at"]=="2026-10-30T11:30:00Z"&&obligation["status"]=="fail"));
    let later =
        evaluation::evaluate_default(&program, &records, "2026-10-30T12:02:00Z", &context).unwrap();
    assert_eq!(
        later["claims"]["result"]["status"], "unsupported",
        "{later}"
    );
    assert_eq!(
        later["arguments"]["verification"]["reasoning_result"]["details"]["verification"]["status"],
        "fail"
    );
    assert_eq!(
        later["arguments"]["verification"]["reasoning_result"]["assessment_context"]["assessed_at"],
        "2026-10-30T12:02:00Z"
    );
}

#[allow(dead_code)]
fn types_are_public(_: Coverage, _: Contract, _: ObservationStatus) {}
