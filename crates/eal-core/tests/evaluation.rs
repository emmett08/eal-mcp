//! Check observation completeness and pure evaluation boundaries independently of prose.
use eal_core::{
    ExecutionLimits, canonical, composition, evaluation, methods, semantic::Program, validation,
};
use serde_json::{Value, json};
const SOURCE: &str = include_str!("support/evaluation.eal");
const NOW: &str = "2026-09-23T12:00:00Z";

fn observation(program: &Program, context: &Value) -> Value {
    let evidence = &program.evidence["observed"];
    let tool = &program.tools[&evidence.tool];
    let acquisition = json!({"tool":tool.name,"tool_version":tool.version,"input":evidence.input,"context":context});
    let request = json!({"evidence_id":"observed","environment":evidence.environment,"tool":tool.name,"tool_version":tool.version,"input":evidence.input,"context":context});
    let digest = |value: &Value| canonical::canonical_digest(value, &program.limits).unwrap();
    let value = json!({"passed":true});
    json!({"evidence_id":"observed","source_digest":program.source_digest,
        "tool":tool.name,"tool_version":tool.version,"tool_binding_digest":"0".repeat(64),
        "evidence_kind":evidence.kind,"environment":evidence.environment,
        "environment_fingerprint":evaluation::environment_fingerprint(&evidence.environment,context,&program.limits).unwrap(),
        "input_digest":digest(&evidence.input),"input":evidence.input,"context":context,
        "acquisition_request_digest":digest(&acquisition),"acquisition_request":acquisition,
        "request_digest":digest(&request),"collected_at":NOW,"run_id":"native-test-observation",
        "status":"ok","data_digest":digest(&value),"value":value})
}

fn baseline() -> (Program, Value, Value, String) {
    let program = composition::compile(SOURCE, None, "", ExecutionLimits::default()).unwrap();
    let context = json!({"site":"bench"});
    let records = json!({"observed":observation(&program,&context)});
    (program, records, context, NOW.into())
}

#[test]
fn complete_negative_predicate_differs_from_missing_or_malformed_observation() {
    let (program, records, context, now) = baseline();
    let instant = validation::parse_time(&now).unwrap();
    for (value, complete, issues) in [
        (json!({"passed":true}), true, json!([])),
        (json!({"passed":false}), true, json!(["predicate_not_met"])),
        (json!({}), false, json!(["invalid_observation"])),
        (json!({"passed":1}), false, json!(["invalid_observation"])),
    ] {
        let mut record = records["observed"].clone();
        record["data_digest"] =
            json!(canonical::canonical_digest(&value, &program.limits).unwrap());
        record["value"] = value;
        let verdict = evaluation::assess_evidence_record(
            &program,
            "observed",
            Some(&record),
            instant,
            &context,
            true,
            None,
            false,
        )
        .unwrap();
        assert_eq!(verdict.complete, complete);
        assert_eq!(verdict.entry["availability_issues"], issues);
    }
    let verdict = evaluation::assess_evidence_record(
        &program, "observed", None, instant, &context, false, None, false,
    )
    .unwrap();
    assert!(!verdict.complete);
    assert_eq!(
        verdict.entry["availability_issues"],
        json!(["missing_observation", "out_of_scope"])
    );
}

#[test]
fn current_binding_is_checked_separately_from_record_self_consistency() {
    let (program, records, context, now) = baseline();
    let instant = validation::parse_time(&now).unwrap();
    let expected = json!("1".repeat(64));
    let verdict = evaluation::assess_evidence_record(
        &program,
        "observed",
        Some(&records["observed"]),
        instant,
        &context,
        true,
        Some(&expected),
        true,
    )
    .unwrap();
    assert!(!verdict.complete);
    assert_eq!(
        verdict.entry["availability_issues"],
        json!(["invalid_observation"])
    );
    assert!(
        verdict.entry["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!(
                "Current operator tool binding differs from the collected binding"
            ))
    );
}

#[test]
fn nested_environment_context_is_selected_explicitly() {
    let (program, records, context, now) = baseline();
    let selected = json!({"$environments":{"lab":context,"other":{"site":"field"}}});
    let result = evaluation::evaluate_default(&program, &records, &now, &selected).unwrap();
    assert_eq!(result["claims"]["working"]["status"], "supported");
    assert_eq!(
        evaluation::environment_context("unknown", &selected).unwrap(),
        json!({})
    );
    assert!(evaluation::environment_context("lab", &json!({"$environments":{"lab":42}})).is_err());
}

#[test]
fn record_byte_budget_blocks_use_without_manufacturing_a_negative_claim() {
    let (mut program, records, context, now) = baseline();
    program.limits.record_bytes = 1;
    let result = evaluation::evaluate_default(&program, &records, &now, &context).unwrap();
    assert_eq!(result["valid"], true);
    assert_eq!(result["claims"]["working"]["status"], "unsupported");
    assert_eq!(
        result["evidence"]["observed"]["availability_issues"],
        json!(["invalid_observation"])
    );
    assert!(
        result["evidence"]["observed"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!(
                "Invalid observation value: Record value exceeds byte limit"
            ))
    );
}

#[test]
fn pure_entrypoint_requires_explicit_time_and_mapping_inputs() {
    let (program, records, context, now) = baseline();
    let registry = methods::default_registry();
    assert!(
        evaluation::evaluate(
            &program,
            &records,
            "2026-09-23T12:00:00",
            &context,
            registry,
            None
        )
        .is_err()
    );
    assert_eq!(
        evaluation::evaluate(&program, &json!([]), &now, &context, registry, None).unwrap_err(),
        "Context and records must be mappings"
    );
    assert_eq!(
        evaluation::evaluate(
            &program,
            &records,
            &now,
            &context,
            registry,
            Some(&json!([]))
        )
        .unwrap_err(),
        "binding_digests must map evidence IDs to current operator configurations"
    );
}

#[test]
fn evaluation_and_projection_work_on_an_ordinary_embedding_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let (program, records, context, now) = baseline();
            let result = evaluation::evaluate_default(&program, &records, &now, &context).unwrap();
            assert_eq!(result["claims"]["downstream"]["status"], "supported");
            serde_json::to_vec(&result).unwrap();
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn maximum_premise_edge_depth_is_safe_and_one_less_budget_rejects() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut source = SOURCE.to_owned();
            for index in (0..=128).rev() {
                source.push_str(&format!(
                    "\nclaim c{index} {{\nstatement \"Scoped chain claim\"\nenvironment lab\n}}\n"
                ));
            }
            source.push_str("\nargument a0 = [evidence observed] via method => c0\n");
            for index in 1..=128 {
                source.push_str(&format!(
                    "argument a{index} = [premises c{}] via method => c{index}\n",
                    index - 1
                ));
            }
            let mut program =
                composition::compile(&source, None, "", ExecutionLimits::default()).unwrap();
            let context = json!({"site":"bench"});
            let records = json!({"observed":observation(&program,&context)});
            let result = evaluation::evaluate_default(&program, &records, NOW, &context).unwrap();
            assert_eq!(result["valid"], true, "{}", result["diagnostics"]);
            assert_eq!(result["claims"]["c128"]["status"], "supported");
            program.limits.premise_depth = 127;
            let rejected = evaluation::evaluate_default(&program, &records, NOW, &context).unwrap();
            assert_eq!(rejected["valid"], false);
            assert_eq!(rejected["claims"], json!({}));
        })
        .unwrap()
        .join()
        .unwrap();
}
