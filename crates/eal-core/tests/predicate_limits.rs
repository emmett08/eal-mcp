//! Runtime predicates use the admitted programme's JSON allowance in every role.
use eal_core::{
    ExecutionLimits, canonical, composition, evaluation, semantic::Program, validation,
};
use serde_json::{Value, json};

const NOW: &str = "2026-10-27T12:00:00Z";

fn compile(source: &str, json_nodes: usize) -> Program {
    let program = composition::compile(
        source,
        None,
        "predicate-limits.eal",
        ExecutionLimits {
            json_nodes,
            ..ExecutionLimits::default()
        },
    )
    .unwrap();
    let diagnostics = validation::validate_default(&program);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    program
}

fn expanded_equality(field: &str, copies: usize) -> String {
    let fields = (0..copies)
        .map(|index| format!("\"copy{index}\", {field}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("equal(object({fields}), object({fields}))")
}

fn observation(program: &Program, value: &Value) -> Value {
    let evidence = &program.evidence["observed"];
    let context = json!({});
    let acquisition =
        json!({"tool":"reader","tool_version":"1","input":evidence.input,"context":context});
    let request = json!({"evidence_id":"observed","environment":"lab","tool":"reader","tool_version":"1","input":evidence.input,"context":context});
    let digest = |value: &Value| canonical::canonical_digest(value, &program.limits).unwrap();
    json!({
        "evidence_id":"observed","source_digest":program.source_digest,
        "tool":"reader","tool_version":"1","tool_binding_digest":"0".repeat(64),
        "evidence_kind":"test","environment":"lab",
        "environment_fingerprint":evaluation::environment_fingerprint("lab",&context,&program.limits).unwrap(),
        "input_digest":digest(&evidence.input),"input":evidence.input,"context":context,
        "acquisition_request_digest":digest(&acquisition),"acquisition_request":acquisition,
        "request_digest":digest(&request),"collected_at":NOW,"run_id":"predicate-limit-observation",
        "status":"ok","data_digest":digest(value),"value":value
    })
}

fn evidence_source(predicate: &str) -> String {
    format!(
        "language \"EAL/4\"\nenvironment lab {{\nrequire true\n}}\ntool reader {{\nversion \"1\"\n}}\nevidence observed {{\ntool reader\nkind test\nenvironment lab\nmax_age 60\ninput {{}}\nrequire {predicate}\n}}\n"
    )
}

#[test]
fn environment_predicate_can_use_a_raised_json_node_allowance() {
    let program = compile(
        "language \"EAL/4\"\nenvironment lab {\nrequire distinct(items) == true\n}\n",
        200_000,
    );
    let context = json!({"items":(0..100_001).collect::<Vec<_>>()});
    let result = evaluation::evaluate_default(&program, &json!({}), NOW, &context).unwrap();
    assert_eq!(
        result["environments"]["lab"]["status"], "matched",
        "{result}"
    );
}

#[test]
fn environment_predicate_cannot_expand_past_a_lowered_json_node_allowance() {
    let source = format!(
        "language \"EAL/4\"\nenvironment lab {{\nrequire {}\n}}\n",
        expanded_equality("items", 2)
    );
    let context = json!({"items":(0..600).collect::<Vec<_>>()});
    for (allowance, expected) in [(3_000, "matched"), (1_000, "out_of_scope")] {
        let program = compile(&source, allowance);
        let result = evaluation::evaluate_default(&program, &json!({}), NOW, &context).unwrap();
        assert_eq!(
            result["environments"]["lab"]["status"], expected,
            "{result}"
        );
    }
}

#[test]
fn evidence_predicate_can_use_a_raised_json_node_allowance() {
    let program = compile(&evidence_source("distinct(items) == true"), 200_000);
    let value = json!({"items":(0..100_001).collect::<Vec<_>>()});
    let records = json!({"observed":observation(&program,&value)});
    let verdicts = evaluation::assess_evidence_records(
        &program,
        &["observed".into()],
        &records,
        validation::parse_time(NOW).unwrap(),
        &json!({}),
        None,
    )
    .unwrap();
    assert_eq!(verdicts["observed"].entry["status"], "available");
    assert!(verdicts["observed"].complete);
}

#[test]
fn evidence_predicate_exhaustion_preserves_usable_data_without_accepting_it() {
    let source = evidence_source(&expanded_equality("items", 2));
    let value = json!({"items":(0..600).collect::<Vec<_>>()});
    for (allowance, expected) in [(3_000, "available"), (1_000, "unavailable")] {
        let program = compile(&source, allowance);
        let records = json!({"observed":observation(&program,&value)});
        let verdicts = evaluation::assess_evidence_records(
            &program,
            &["observed".into()],
            &records,
            validation::parse_time(NOW).unwrap(),
            &json!({}),
            None,
        )
        .unwrap();
        let verdict = &verdicts["observed"];
        assert_eq!(verdict.entry["status"], expected, "{:?}", verdict.entry);
        assert!(verdict.data_available);
        if allowance == 1_000 {
            assert!(!verdict.complete);
            assert_eq!(
                verdict.entry["availability_issues"],
                json!(["predicate_unresolved"])
            );
        }
    }
}

fn native_source(reasoning_predicate: Option<&str>, result_predicate: &str) -> String {
    let reasoning_predicate = reasoning_predicate
        .map(|predicate| format!("require {predicate}\n"))
        .unwrap_or_default();
    format!(
        r#"language "EAL/4"
environment lab {{
 require true
}}
tool reader {{
 version "1"
}}
evidence observed {{
 tool reader
 kind test
 environment lab
 max_age 60
 input {{}}
 require passed == true
}}
reasoning native {{
 method "deductive/1"
 rationale "Finite entailment checks the authored identity."
 {reasoning_predicate}
}}
claim result {{
 statement "The finite premise entails itself."
 environment lab
 proposition {{
  subject "predicate-budget-check"
  quantity "proposition"
  unit "1"
  scope "one finite atom"
  valid_from "2026-01-01T00:00:00Z"
  valid_until "2027-01-01T00:00:00Z"
  query {{"premises":["p"],"conclusion":"p"}}
  result {result_predicate}
 }}
}}
argument proof = [evidence observed] via native => result
"#
    )
}

fn check_native_predicate(source: &str) {
    for (allowance, expected) in [(2_000, "supported"), (256, "unsupported")] {
        let program = compile(source, allowance);
        let records = json!({"observed":observation(&program,&json!({"passed":true}))});
        let result = evaluation::evaluate_default(&program, &records, NOW, &json!({})).unwrap();
        assert_eq!(
            result["claims"]["result"]["status"],
            expected,
            "method status {:?}; binding status {:?}",
            result["arguments"]["proof"]["reasoning_result"]["status"],
            result["arguments"]["proof"]["reasoning_result"]["binding"]["status"]
        );
        assert_eq!(
            result["arguments"]["proof"]["reasoning_result"]["details"]["entailed"],
            true
        );
    }
}

#[test]
fn reasoning_predicate_uses_the_programme_json_node_allowance() {
    check_native_predicate(&native_source(
        Some(&expanded_equality("entailed", 320)),
        "entailed == true",
    ));
}

#[test]
fn proposition_result_uses_the_programme_json_node_allowance() {
    check_native_predicate(&native_source(None, &expanded_equality("entailed", 320)));
}
