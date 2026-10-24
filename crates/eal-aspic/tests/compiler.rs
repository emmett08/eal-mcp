//! Native snapshot identities and scope remain independently checkable.
#[path = "../../../tests/native_case.rs"]
mod native_case;
use eal_aspic::{CompileOptions, compile_eal_aspic};
use eal_core::{ExecutionLimits, canonical, methods};
use serde_json::{Value, json};
fn compile(case: &Value) -> Value {
    compile_eal_aspic(
        case["source"].as_str().unwrap(),
        &case["records"],
        CompileOptions::new(
            case["goal"].as_str().unwrap(),
            case["now"].as_str().unwrap(),
            &case["context"],
        ),
    )
    .unwrap()
    .to_dict()
    .unwrap()
}
#[test]
fn source_snapshot_retains_exact_observation_identity_and_authored_results() {
    let case = native_case::case();
    let result = compile(&case);
    assert_eq!(result["claim_status"], "supported");
    assert_eq!(result["authored_claim_status"], "supported");
    assert_eq!(result["formal"]["query_accepted"], true);
    let mapping = &result["source_map"];
    let limits = ExecutionLimits::default();
    assert_eq!(
        mapping["theory_digest"],
        canonical::canonical_digest(&result["theory"], &limits).unwrap()
    );
    assert_eq!(
        mapping["method_registry_fingerprint"],
        methods::default_registry().fingerprint()
    );
    assert_eq!(
        mapping["evidence"]["observed"]["identity"]["collected_at"],
        native_case::NOW
    );
    assert_eq!(
        mapping["backend"]["module_digest"],
        eal_aspic::registry::module_identity()
    );
    assert_eq!(compile(&case), result);
}
#[test]
fn stale_missing_and_measured_negative_records_remain_distinct() {
    let mut case = native_case::case();
    case["now"] = json!("2026-10-24T12:01:00.000001Z");
    let stale = compile(&case);
    assert_eq!(stale["claim_status"], "unsupported");
    assert_eq!(
        stale["source_map"]["evidence"]["observed"]["availability_issues"],
        json!(["stale_observation"])
    );
    case["now"] = json!(native_case::NOW);
    case["records"] = json!({});
    let missing = compile(&case);
    assert_eq!(
        missing["source_map"]["evidence"]["observed"]["availability_issues"],
        json!(["missing_observation"])
    );
    case["records"] = json!({"observed":native_case::observation(native_case::SOURCE,"observed",&case["context"],json!({"passed":false}))});
    let negative = compile(&case);
    assert_eq!(negative["claim_status"], "unsupported");
    assert_eq!(
        negative["source_map"]["evidence"]["observed"]["availability_issues"],
        json!(["predicate_not_met"])
    );
}
#[test]
fn compiler_options_preserve_all_selected_semantics_and_query_modes() {
    let case = native_case::case();
    for semantics in ["grounded", "preferred", "stable"] {
        for query in ["credulous", "sceptical"] {
            for preference in ["minimum_rank", "last_link_rank", "last_link_partial"] {
                let mut options =
                    CompileOptions::new("working", native_case::NOW, &case["context"]);
                options.semantics = semantics;
                options.query_mode = query;
                options.preference = Some(preference);
                let result = compile_eal_aspic(native_case::SOURCE, &case["records"], options)
                    .unwrap()
                    .solve()
                    .unwrap();
                assert_eq!(result["formal"]["semantics"], semantics);
                assert_eq!(result["formal"]["query_mode"], query);
                assert_eq!(result["formal"]["query_accepted"], true);
            }
        }
    }
}

#[test]
fn explicit_formal_method_is_assessed_without_recursive_source_compilation() {
    let case = native_case::typed_case();
    let registry = eal_aspic::aspic_registry();
    let mut options = CompileOptions::new("accepted", native_case::NOW, &case["context"]);
    options.registry = Some(&registry);
    let error =
        compile_eal_aspic(case["source"].as_str().unwrap(), &case["records"], options).unwrap_err();
    assert!(
        matches!(error, eal_aspic::AspicError::Invalid(_)),
        "{error}"
    );
    assert!(
        error.to_string().contains("argumentation/aspic/2"),
        "{error}"
    );
}
