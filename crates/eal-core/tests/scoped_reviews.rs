//! A review's identity, shape and original clock cannot be replaced by its container.
use eal_core::{
    ExecutionLimits, acquisition, canonical, composition, evaluation, expressions,
    semantic::Program, validation,
};
use serde_json::{Value, json};
const NOW: &str = "2026-09-23T12:00:00Z";
const EXPECTED: &str =
    r#"{"criterion":"bounded decision","target":"decision","scope":{"episode":"A"}}"#;

fn program(direct: bool) -> Program {
    let source = if direct {
        format!(
            "evidence source {{\n tool reader\n kind scoped_review\n environment lab\n max_age 3600\n input {EXPECTED}\n require verdict == \"supported\"\n}}\n"
        )
    } else {
        "evidence source {\n tool reader\n kind observation\n environment lab\n max_age 3600\n require true\n}\n".into()
    };
    let input: Value = serde_json::from_str(EXPECTED).unwrap();
    let mut input = input.as_object().unwrap().clone();
    input.insert("review".into(), Value::Null);
    let kind = if direct {
        "calculation"
    } else {
        "scoped_review"
    };
    let pointer = if direct { "" } else { "/review" };
    let source = format!(
        r#"language "EAL/3" profile "execution/1"
environment lab {{
 require true
}}
tool reader {{
 version "1"
}}
{source}
evidence checked {{
 derive review
 kind {kind}
 environment lab
 max_age 3600
 input {input}
 bind "/review" from source "{pointer}"
 require verdict == "supported"
}}
evidence decision_input {{
 derive object("usable", review.verdict == "supported")
 kind calculation
 environment lab
 max_age 3600
 input {{"review":null}}
 bind "/review" from checked ""
 require usable == true
}}
reasoning retained {{
 method "structured/1"
 rationale "Use only the identified bounded review."
}}
claim decision {{
 statement "The scoped review supports this decision."
 environment lab
}}
argument supported = [evidence decision_input] via retained => decision
"#,
        input = Value::Object(input)
    );
    composition::compile(&source, None, "", ExecutionLimits::default()).unwrap()
}

fn record_set(program: &Program, review: &Value, direct: bool) -> Value {
    let mut records = json!({});
    for name in ["source", "checked", "decision_input"] {
        let evidence = &program.evidence[name];
        let resolved = acquisition::resolve_input(program, name, &records).unwrap();
        let value = if let Some(expression) = &evidence.derivation {
            expressions::evaluate_expression(expression, &resolved.input)
                .value
                .unwrap_or(Value::Null)
        } else if direct {
            review.clone()
        } else {
            json!({"review":review})
        };
        let tool = evidence.tool.as_ref().map(|name| &program.tools[name]);
        let tool_name = tool.map(|tool| &tool.name);
        let version = tool.map(|tool| &tool.version);
        let acquire =
            json!({"tool":tool_name,"tool_version":version,"input":resolved.input,"context":{}});
        let request = json!({"evidence_id":name,"environment":"lab","tool":tool_name,"tool_version":version,"input":resolved.input,"context":{}});
        let digest = |value: &Value| canonical::canonical_digest(value, &program.limits).unwrap();
        let mut clock = NOW.to_owned();
        for parent in resolved.dependencies.as_array().unwrap() {
            let parent = parent["collected_at"].as_str().unwrap();
            if parent < clock.as_str() {
                clock = parent.into();
            }
        }
        if evidence.kind == "scoped_review"
            && let Some(time) = review["reviewed_at"]
                .as_str()
                .filter(|time| validation::parse_time(time).is_ok())
            && time < clock.as_str()
        {
            clock = time.into();
        }
        let binding = if evidence.derivation.is_some() {
            acquisition::derivation_digest(program, name).unwrap()
        } else {
            "0".repeat(64)
        };
        records[name] = json!({"evidence_id":name,"source_digest":program.source_digest,"tool":tool_name,"tool_version":version,
            "evidence_kind":evidence.kind,"environment":"lab","environment_fingerprint":evaluation::environment_fingerprint("lab",&json!({}),&program.limits).unwrap(),
            "tool_binding_digest":binding,"input":resolved.input,"input_digest":digest(&resolved.input),"context":{},
            "acquisition_request":acquire,"acquisition_request_digest":digest(&acquire),"request_digest":digest(&request),
            "status":"ok","run_id":format!("retained-{name}"),"collected_at":clock,"computed_at":NOW,
            "value":value,"data_digest":digest(&value),"dependencies":resolved.dependencies});
    }
    records
}

fn valid_review() -> Value {
    json!({"criterion":"bounded decision","target":"decision","scope":{"episode":"A"},"reviewer":"named-reviewer","verdict":"supported","reasons":["The declared criterion holds in this episode."],"reviewed_at":NOW})
}

#[test]
fn direct_and_nested_review_admission_blocks_malformed_identity_and_shape() {
    for direct in [false, true] {
        let program = program(direct);
        let good = record_set(&program, &valid_review(), direct);
        let result = evaluation::evaluate_default(&program, &good, NOW, &json!({})).unwrap();
        assert_eq!(
            result["claims"]["decision"]["status"], "supported",
            "{result}"
        );
        for (field, value) in [
            ("reviewer", json!("  ")),
            ("reviewer", Value::Null),
            ("reasons", json!([])),
            ("reasons", json!([null])),
            ("reasons", json!([" "])),
            ("reviewed_at", json!("invalid")),
            ("scope", json!({"episode":"B"})),
            ("target", json!("another-decision")),
        ] {
            let mut review = valid_review();
            review[field] = value;
            let records = record_set(&program, &review, direct);
            let verdicts = evaluation::assess_evidence_records(
                &program,
                &["decision_input".into()],
                &records,
                validation::parse_time(NOW).unwrap(),
                &json!({}),
                None,
            )
            .unwrap();
            let gate = if direct { "source" } else { "checked" };
            assert!(
                !verdicts[gate].data_available,
                "{field}: {:?}",
                verdicts[gate]
            );
            assert!(!verdicts["decision_input"].data_available);
        }
    }
}

#[test]
fn fresh_container_cannot_refresh_old_or_future_review_clock() {
    for direct in [false, true] {
        let program = program(direct);
        for time in ["2026-09-23T10:00:00Z", "2026-09-23T13:00:00Z"] {
            let mut review = valid_review();
            review["reviewed_at"] = json!(time);
            let mut records = record_set(&program, &review, direct);
            // Simulate a fresh generic envelope and all descendants keeping only
            // that envelope time. The independent original review clock wins.
            for record in records.as_object_mut().unwrap().values_mut() {
                record["collected_at"] = json!(NOW);
            }
            let result = evaluation::evaluate_default(&program, &records, NOW, &json!({})).unwrap();
            assert_eq!(
                result["claims"]["decision"]["status"], "unsupported",
                "{direct}/{time}: {result}"
            );
        }
    }
}

#[test]
fn valid_rejected_review_remains_data_without_supporting_the_decision() {
    let program = program(false);
    let mut review = valid_review();
    review["verdict"] = json!("rejected");
    let records = record_set(&program, &review, false);
    let checked = evaluation::assess_evidence_records(
        &program,
        &["decision_input".into()],
        &records,
        validation::parse_time(NOW).unwrap(),
        &json!({}),
        None,
    )
    .unwrap();
    assert!(checked["checked"].data_available);
    assert_eq!(checked["checked"].entry["status"], "unavailable");
    let result = evaluation::evaluate_default(&program, &records, NOW, &json!({})).unwrap();
    assert_eq!(result["claims"]["decision"]["status"], "unsupported");
}
