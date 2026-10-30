//! Independent finite probability definitions exercise exact inference and mapping errors.
use eal_core::{ExecutionLimits, methods};
use serde_json::{Value, json};

fn query() -> Value {
    json!({
        "nodes":[
            {"name":"Fault","states":["healthy","faulty"],"parents":[],"probabilities":["49/50","1/50"]},
            {"name":"Alarm","states":["false","true"],"parents":["Fault"],"probabilities":["19/20","1/20","1/20","19/20"]},
            {"name":"HeartbeatLost","states":["false","true"],"parents":["Fault"],"probabilities":["9/10","1/10","1/5","4/5"]}
        ],
        "evidence":{"Alarm":"true","HeartbeatLost":"true"},
        "target":{"variable":"Fault","state":"faulty"},
        "comparison":{"operator":">=","threshold":"3/4"}
    })
}

fn assess(query: &Value) -> Value {
    methods::assess_input_with_limits(
        "finite_probability/1",
        query,
        methods::default_registry(),
        &ExecutionLimits::default(),
    )
}

fn exact(numerator: &str, denominator: &str) -> Value {
    json!({"schema":"eal.exact-number/1","numerator":numerator,"denominator":denominator,"unit":"1"})
}

#[test]
fn exact_model_consequence_and_adverse_event_have_independently_known_answers() {
    let result = assess(&query());
    assert_eq!(result["status"], "supported", "{result}");
    assert_eq!(result["input_origin"], "authored_program");
    assert!(result.get("evidence_id").is_none());
    assert_eq!(result["details"]["defined"], true);
    assert_eq!(result["details"]["satisfies"], true);
    assert_eq!(result["details"]["posterior_exact"], exact("152", "201"));
    assert_eq!(result["details"]["normalizer_exact"], exact("201", "10000"));
    assert_eq!(result["details"]["numerator_exact"], exact("19", "1250"));
    assert_eq!(result["details"]["assignments"], 8);
    let mut adverse = query();
    adverse["evidence"]["HeartbeatLost"] = json!("false");
    let result = assess(&adverse);
    assert_eq!(
        result["status"], "supported",
        "Completed negative calculations remain results"
    );
    assert_eq!(result["details"]["satisfies"], false);
    assert_eq!(result["details"]["posterior_exact"], exact("38", "479"));
    assert_eq!(result["details"]["normalizer_exact"], exact("479", "10000"));
}

#[test]
fn different_graph_and_state_order_use_supplied_definitions() {
    let different = json!({"nodes":[
        {"name":"Hypothesis","states":["absent","present"],"parents":[],"probabilities":["4/5","1/5"]},
        {"name":"Signal","states":["off","on"],"parents":["Hypothesis"],"probabilities":["23/24","1/24","0","1"]}
    ],"evidence":{"Signal":"on"},"target":{"variable":"Hypothesis","state":"present"},
        "comparison":{"operator":"==","threshold":"6/7"}});
    let result = assess(&different);
    assert_eq!(result["details"]["posterior_exact"], exact("6", "7"));
    assert_eq!(result["details"]["normalizer_exact"], exact("7", "30"));
    assert_eq!(result["details"]["satisfies"], true);
    let mut reordered = query();
    reordered["nodes"].as_array_mut().unwrap().reverse();
    let result = assess(&reordered);
    assert_eq!(result["details"]["posterior_exact"], exact("152", "201"));
    assert_eq!(
        result["details"]["mapping"]["node_order"][0],
        "HeartbeatLost"
    );
    let mut wrong_states = query();
    wrong_states["nodes"][0]["states"] = json!(["faulty", "healthy"]);
    let result = assess(&wrong_states);
    assert_eq!(result["status"], "supported");
    assert_eq!(result["details"]["posterior_exact"], exact("49", "201"));
    assert_eq!(result["details"]["satisfies"], false);
}

#[test]
fn ordered_parent_columns_and_child_states_are_explicit() {
    // A,B are fair coins; C is true exactly for A=true,B=false.
    let mut value = json!({"nodes":[
        {"name":"A","states":["f","t"],"parents":[],"probabilities":["1/2","1/2"]},
        {"name":"B","states":["f","t"],"parents":[],"probabilities":["1/2","1/2"]},
        {"name":"C","states":["f","t"],"parents":["A","B"],"probabilities":["1","0","1","0","0","1","1","0"]}
    ],"evidence":{"A":"t","B":"f"},"target":{"variable":"C","state":"t"},"comparison":{"operator":"==","threshold":"1"}});
    assert_eq!(assess(&value)["details"]["satisfies"], true);
    value["nodes"][2]["parents"] = json!(["B", "A"]);
    assert_eq!(assess(&value)["details"]["satisfies"], false);
}

#[test]
fn exact_threshold_comparison_does_not_round_through_binary64() {
    let mut value = query();
    value["comparison"] = json!({"operator":">","threshold":"7562189054726368159203980099502487562189/10000000000000000000000000000000000000000"});
    assert_eq!(assess(&value)["details"]["satisfies"], true);
    value["comparison"] = json!({"operator":"==","threshold":"152/201"});
    assert_eq!(assess(&value)["details"]["satisfies"], true);
}

#[test]
fn impossible_evidence_is_defined_as_undefined_conditioning_without_a_posterior() {
    let mut value = query();
    value["nodes"][1]["probabilities"] = json!(["1", "0", "1", "0"]);
    let result = assess(&value);
    assert_eq!(
        result["status"], "supported",
        "The finite computation completed"
    );
    assert_eq!(result["details"]["defined"], false);
    assert_eq!(result["details"]["satisfies"], false);
    assert_eq!(result["details"]["normalizer_exact"], exact("0", "1"));
    assert!(result["details"].get("posterior").is_none());
    assert!(result["details"].get("posterior_exact").is_none());
}

#[test]
fn malformed_and_unbounded_models_never_produce_supported_results() {
    let mut cases = Vec::new();
    let mut value = query();
    value["nodes"][0]["probabilities"] = json!(["1/2", "1/3"]);
    cases.push(value);
    let mut value = query();
    value["nodes"][0]["probabilities"] = json!(["1/0", "1"]);
    cases.push(value);
    let mut value = query();
    value["nodes"][0]["probabilities"] = json!([true, "1"]);
    cases.push(value);
    let mut value = query();
    value["nodes"][0]["probabilities"] = json!(["-1", "2"]);
    cases.push(value);
    let mut value = query();
    value["nodes"][0]["probabilities"] = json!(["0.98", "0.02"]);
    cases.push(value);
    let mut value = query();
    value["nodes"][1]["parents"] = json!(["missing"]);
    cases.push(value);
    let mut value = query();
    value["nodes"][1]["parents"] = json!(["Fault", "Fault"]);
    cases.push(value);
    let mut value = query();
    value["nodes"][1]["probabilities"] = json!(["1", "0"]);
    cases.push(value);
    let mut value = query();
    value["nodes"][0]["parents"] = json!(["Alarm"]);
    value["nodes"][0]["probabilities"] = json!(["1/2", "1/2", "1/2", "1/2"]);
    cases.push(value);
    let mut value = query();
    value["nodes"][1]["name"] = json!("Fault");
    cases.push(value);
    let mut value = query();
    value["nodes"][0]["states"] = json!(["healthy", "healthy"]);
    cases.push(value);
    let mut value = query();
    value["evidence"]["Alarm"] = json!("unknown");
    cases.push(value);
    let mut value = query();
    value["comparison"]["threshold"] = json!("1".repeat(129));
    cases.push(value);
    for value in cases {
        let result = assess(&value);
        assert_eq!(result["status"], "unsupported", "{value}: {result}");
        assert!(
            !methods::validate_query("finite_probability/1", &value, methods::default_registry())
                .is_empty()
        );
    }
}

#[test]
fn native_work_exhaustion_is_incomplete_and_ordinary_equivalence_has_countermodels() {
    let limits = ExecutionLimits {
        method_work: 1,
        ..Default::default()
    };
    let result = methods::assess_input_with_limits(
        "finite_probability/1",
        &query(),
        methods::default_registry(),
        &limits,
    );
    assert_eq!(result["status"], "incomplete");
    assert_eq!(result["details"], json!({}));
    for (left, right, expected) in [
        (
            json!({"implies":["p","q"]}),
            json!({"or":[{"not":"p"},"q"]}),
            true,
        ),
        (
            json!({"implies":["p","q"]}),
            json!({"implies":["q","p"]}),
            false,
        ),
    ] {
        let query = json!({"premises":[],"conclusion":{"and":[{"implies":[left,right]},{"implies":[right,left]}]}});
        let result = methods::assess_input_with_limits(
            "deductive/1",
            &query,
            methods::default_registry(),
            &ExecutionLimits::default(),
        );
        assert_eq!(result["details"]["entailed"], expected);
        assert_eq!(result["details"]["counterexample"].is_object(), !expected);
    }
}

#[test]
fn assignment_and_intermediate_numeric_budgets_are_enforced_separately() {
    let mut large = query();
    large["nodes"] = json!((0..7).map(|i|json!({"name":format!("N{i}"),"states":["a","b","c","d","e","f","g","h"],"parents":[],"probabilities":["1/8","1/8","1/8","1/8","1/8","1/8","1/8","1/8"]})).collect::<Vec<_>>());
    large["target"] = json!({"variable":"N0","state":"a"});
    large["evidence"] = json!({});
    let result = assess(&large);
    assert_eq!(
        result["status"], "unsupported",
        "Oversized model is a domain error, not partial enumeration"
    );
    assert!(result["reasons"].to_string().contains("65536"));
    let states = json!(["a", "b", "c", "d", "e", "f", "g", "h"]);
    let mut probabilities = Vec::new();
    for index in 0..64 {
        let denominator = format!("1{}{:03}", "0".repeat(56), index * 2 + 1);
        let denominator: num_bigint::BigInt = denominator.parse().unwrap();
        probabilities.push(format!("1/{denominator}"));
        probabilities.push(format!("{}/{denominator}", &denominator - 1));
    }
    let value = json!({"nodes":[
        {"name":"X","states":states,"parents":[],"probabilities":["1/8","1/8","1/8","1/8","1/8","1/8","1/8","1/8"]},
        {"name":"Y","states":states,"parents":[],"probabilities":["1/8","1/8","1/8","1/8","1/8","1/8","1/8","1/8"]},
        {"name":"Z","states":["f","t"],"parents":["X","Y"],"probabilities":probabilities}
    ],"evidence":{},"target":{"variable":"Z","state":"f"},"comparison":{"operator":">","threshold":"0"}});
    let result = assess(&value);
    assert_eq!(result["status"], "incomplete", "{result}");
    assert!(result["reasons"].to_string().contains("8192"));
    assert_eq!(result["details"], json!({}));
}

fn source(method: &str, query: &Value, predicate: &str) -> String {
    format!(
        r#"language "EAL/4"
environment mathematical {{
 require true
}}
reasoning exact {{
 method {method:?}
 rationale "Compute only the declared mathematical consequence; prose correspondence and physical adequacy remain separate."
}}
claim result {{
 statement "The authored mathematical query satisfies its result predicate."
 environment mathematical
 proposition {{
  subject "immutable authored definitions"
  quantity "proposition"
  unit "1"
  scope "the exact supplied finite definitions only"
  valid_from "2026-01-01T00:00:00Z"
  valid_until "2027-01-01T00:00:00Z"
  query {query}
  result {predicate}
 }}
}}
argument theorem = [] via exact => result
"#
    )
}

#[test]
fn complete_mathematical_queries_execute_in_normal_eal_without_observations() {
    for (query, expected) in [
        (query(), "supported"),
        (
            {
                let mut q = query();
                q["evidence"]["HeartbeatLost"] = json!("false");
                q
            },
            "unsupported",
        ),
    ] {
        let source = source("finite_probability/1", &query, "satisfies == true");
        let program =
            eal_core::composition::compile(&source, None, "", ExecutionLimits::default()).unwrap();
        assert!(eal_core::validation::validate_default(&program).is_empty());
        for now in ["2026-10-28T12:00:00Z", "2026-10-30T12:00:00Z"] {
            let result =
                eal_core::evaluation::evaluate_default(&program, &json!({}), now, &json!({}))
                    .unwrap();
            assert_eq!(result["valid"], true, "{result}");
            assert_eq!(result["claims"]["result"]["status"], expected, "{result}");
            assert_eq!(
                result["arguments"]["theorem"]["reasoning_result"]["input_origin"],
                "authored_program"
            );
            assert_eq!(result["claims"]["result"]["prose_verified"], false);
            assert_eq!(result["evidence"], json!({}));
        }
    }
    let tautology = json!({"premises":[],"conclusion":{"implies":["p","p"]}});
    let program = eal_core::composition::compile(
        &source("deductive/1", &tautology, "entailed == true"),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    assert!(eal_core::validation::validate_default(&program).is_empty());
    let result = eal_core::evaluation::evaluate_default(
        &program,
        &json!({}),
        "2026-10-30T12:00:00Z",
        &json!({}),
    )
    .unwrap();
    assert_eq!(result["claims"]["result"]["status"], "supported");
    let false_query = json!({"premises":[],"conclusion":{"and":["p",{"not":"p"}]}});
    let program = eal_core::composition::compile(
        &source("deductive/1", &false_query, "entailed == true"),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    let result = eal_core::evaluation::evaluate_default(
        &program,
        &json!({}),
        "2026-10-30T12:00:00Z",
        &json!({}),
    )
    .unwrap();
    assert_eq!(result["claims"]["result"]["status"], "unsupported");
    assert_eq!(
        result["arguments"]["theorem"]["reasoning_result"]["details"]["entailed"],
        false
    );
    let mut undefined = query();
    undefined["nodes"][1]["probabilities"] = json!(["1", "0", "1", "0"]);
    let program = eal_core::composition::compile(
        &source("finite_probability/1", &undefined, "satisfies == true"),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    let result = eal_core::evaluation::evaluate_default(
        &program,
        &json!({}),
        "2026-10-30T12:00:00Z",
        &json!({}),
    )
    .unwrap();
    assert_eq!(result["claims"]["result"]["status"], "unsupported");
    assert_eq!(
        result["arguments"]["theorem"]["reasoning_result"]["details"]["defined"],
        false
    );
}

#[test]
fn empty_arguments_do_not_authorise_empirical_or_untyped_assertions() {
    for (method, query, predicate) in [
        (
            "deductive/1",
            json!({"premises":["p"],"conclusion":"p"}),
            "entailed == true",
        ),
        (
            "deductive/1",
            json!({"premises":[],"conclusion":"p"}),
            "consistent_premises == true",
        ),
        (
            "inductive/1",
            json!({"successes":90,"trials":100,"confidence":0.95}),
            "lower > 0.8",
        ),
        (
            "finite_probability/1",
            json!({"nodes":[]}),
            "satisfies == true",
        ),
        ("structured/1", json!({}), "mechanically_proved == true"),
    ] {
        let program = eal_core::composition::compile(
            &source(method, &query, predicate),
            None,
            "",
            ExecutionLimits::default(),
        )
        .unwrap();
        assert!(
            eal_core::validation::validate_default(&program)
                .iter()
                .any(|diagnostic| diagnostic.code == "empty_argument"),
            "{method}"
        );
    }
    let plain = "language \"EAL/4\"\nenvironment e {\nrequire true\n}\nreasoning r {\nmethod \"finite_probability/1\"\nrationale \"No typed query.\"\nrequire satisfies == true\n}\nclaim c {\nstatement \"Unproved prose.\"\nenvironment e\n}\nargument a = [] via r => c\n";
    let program =
        eal_core::composition::compile(plain, None, "", ExecutionLimits::default()).unwrap();
    assert!(
        eal_core::validation::validate_default(&program)
            .iter()
            .any(|diagnostic| diagnostic.code == "empty_argument")
    );
}
