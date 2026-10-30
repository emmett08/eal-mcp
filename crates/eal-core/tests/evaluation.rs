//! Check observation completeness and pure evaluation boundaries independently of prose.
use eal_core::{
    ExecutionLimits, canonical, composition, evaluation, methods, semantic::Program, validation,
};
use serde_json::{Value, json};
const SOURCE: &str = include_str!("support/evaluation.eal");
const NOW: &str = "2026-09-23T12:00:00Z";

fn observation(program: &Program, context: &Value) -> Value {
    let evidence = &program.evidence["observed"];
    let tool = &program.tools[evidence.tool.as_ref().unwrap()];
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
        (json!({}), false, json!(["predicate_unresolved"])),
        (json!({"passed":1}), false, json!(["predicate_unresolved"])),
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
        assert!(verdict.data_available);
        assert_eq!(verdict.entry["availability_issues"], issues);
    }
    let verdict = evaluation::assess_evidence_record(
        &program, "observed", None, instant, &context, false, None, false,
    )
    .unwrap();
    assert!(!verdict.complete);
    assert!(!verdict.data_available);
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
    assert!(!verdict.data_available);
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
fn failed_acquisition_retains_its_primary_cause_and_every_observation_check() {
    let (program, records, context, now) = baseline();
    let instant = validation::parse_time(&now).unwrap();
    for (bound, message) in [
        (false, "Tool 'runner' is not in the operator registry"),
        (true, "Collector process exited with status 2"),
    ] {
        let mut record = records["observed"].clone();
        record["status"] = json!("error");
        record["error"] = json!({"type":"ValueError","message":message});
        let fields = record.as_object_mut().unwrap();
        fields.remove("value");
        fields.remove("data_digest");
        if !bound {
            fields.remove("tool_binding_digest");
        }
        // A retained error explains failure; it cannot excuse altered identity.
        record["input_digest"] = json!("f".repeat(64));
        let verdict = evaluation::assess_evidence_record(
            &program,
            "observed",
            Some(&record),
            instant,
            &context,
            true,
            Some(&json!("0".repeat(64))),
            true,
        )
        .unwrap();
        assert!(!verdict.complete);
        assert!(!verdict.data_available);
        assert_eq!(verdict.entry["status"], "unavailable");
        assert_eq!(
            verdict.entry["availability_issues"],
            json!(["invalid_observation", "tool_error"])
        );
        let reasons = verdict.entry["reasons"].as_array().unwrap();
        assert!(reasons[0].as_str().unwrap().contains(message));
        for reason in [
            "Record input_digest does not match the declared evidence request",
            "Tool execution did not produce an ok observation",
            "Invalid observation value: Record has no JSON value",
        ] {
            assert!(reasons.contains(&json!(reason)));
        }
        for reason in [
            "Record requires a tool binding configuration digest",
            "Current operator tool binding differs from the collected binding",
        ] {
            assert_eq!(reasons.contains(&json!(reason)), !bound);
        }
    }
}

#[test]
fn retained_failure_diagnostics_have_stable_key_order() {
    let (program, records, context, now) = baseline();
    let instant = validation::parse_time(&now).unwrap();
    let mut entries = Vec::new();
    for error in [
        r#"{"type":"ValueError","message":"Collector exited with status 2"}"#,
        r#"{"message":"Collector exited with status 2","type":"ValueError"}"#,
    ] {
        let mut record = records["observed"].clone();
        record["status"] = json!("error");
        record["error"] = serde_json::from_str(error).unwrap();
        let result = evaluation::assess_evidence_record(
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
        assert!(!result.data_available);
        assert!(!result.complete);
        assert_eq!(
            result.entry["reasons"][0],
            "Recorded execution failure: {'message': 'Collector exited with status 2', 'type': 'ValueError'}"
        );
        entries.push(result.entry);
    }
    assert_eq!(entries[0], entries[1]);
}

#[test]
fn retained_failure_text_is_bounded_escaped_and_never_changes_success() {
    let (program, records, context, now) = baseline();
    let instant = validation::parse_time(&now).unwrap();
    let mut record = records["observed"].clone();
    record["error"] = json!({
        "type":"型".repeat(100),"message":"é\n".repeat(1000),
        "details":{"ignored":"This is not part of the diagnostic"}
    });
    let assess = |record: &Value| {
        evaluation::assess_evidence_record(
            &program,
            "observed",
            Some(record),
            instant,
            &context,
            true,
            None,
            false,
        )
        .unwrap()
    };
    let success = assess(&record);
    assert!(success.complete);
    assert!(success.data_available);
    assert_eq!(success.entry["status"], "available");
    record["status"] = json!("error");
    let failed = assess(&record);
    assert!(!failed.complete);
    assert!(!failed.data_available);
    assert_eq!(failed.entry["availability_issues"], json!(["tool_error"]));
    let reason = failed.entry["reasons"][0].as_str().unwrap();
    assert!(reason.len() <= 4096);
    assert!(reason.contains("[truncated]"));
    assert!(reason.contains("\\n"));
    assert!(!reason.contains('\n'));
    assert!(!reason.contains("This is not part of the diagnostic"));
    record["error"] = json!({"message":42});
    let missing = assess(&record);
    assert!(!missing.data_available);
    assert_eq!(
        missing.entry["reasons"][0],
        "Recorded execution failure has no usable retained error diagnostic"
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
    // Scope scalars and Boolean results fit; the 15-byte observation does not.
    program.limits.record_bytes = 10;
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
