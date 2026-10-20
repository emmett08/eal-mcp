//! Differential static checks, typed input correspondence and dimensional results.
use eal_core::{
    ExecutionLimits, composition, methods::default_registry, propositions, semantic, validation,
};
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/validation/reference.json")).unwrap()
}

fn diagnostic_json(diagnostic: &semantic::Diagnostic) -> Value {
    json!({"code":diagnostic.code,"message":diagnostic.message,"declaration":diagnostic.declaration,"expected":diagnostic.expected,"actual":diagnostic.actual,
        "span":diagnostic.span.map(|span|json!({"line":span.start.line,"column":span.start.column,"end_line":span.end.line,"end_column":span.end.column}))})
}

#[test]
fn static_diagnostics_match_the_pinned_python_passes() {
    let fixture = fixture();
    let cases = fixture["cases"].as_array().unwrap();
    assert!(cases.len() >= 35);
    for case in cases {
        let program = composition::compile(
            case["source"].as_str().unwrap(),
            None,
            "",
            ExecutionLimits::default(),
        )
        .unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
        let actual: Vec<_> = validation::validate(&program, default_registry())
            .iter()
            .map(diagnostic_json)
            .collect();
        assert_eq!(json!(actual), case["diagnostics"], "{}", case["name"]);
    }
}

#[test]
fn explicit_iso_instants_match_python_microsecond_utc_semantics() {
    for case in fixture()["times"].as_array().unwrap() {
        let actual = validation::parse_time(case["input"].as_str().unwrap())
            .ok()
            .map(|time| time.timestamp_micros());
        assert_eq!(
            json!(actual),
            case["unix_microseconds"],
            "{}",
            case["input"]
        );
    }
}

#[test]
fn typed_envelopes_preserve_the_whole_query_and_identity() {
    let fixture = fixture();
    let proposition: semantic::Proposition =
        serde_json::from_value(fixture["binding_proposition"].clone()).unwrap();
    for case in fixture["bindings"].as_array().unwrap() {
        let (payload, mut trace) = propositions::prepare_binding(
            &proposition,
            "causal/1",
            "trial",
            &case["input"],
            default_registry(),
        );
        trace.as_object_mut().unwrap().remove("method_contract");
        assert_eq!(json!(payload), case["payload"], "{}", case["name"]);
        assert_eq!(trace, case["trace"], "{}", case["name"]);
    }
}

#[test]
fn rational_unit_conversion_matches_python_and_rejects_erased_results() {
    let fixture = fixture();
    let proposition: semantic::Proposition =
        serde_json::from_value(fixture["binding_proposition"].clone()).unwrap();
    for case in fixture["scalar_results"].as_array().unwrap() {
        let result = propositions::check_result(
            &proposition,
            "causal/1",
            &json!({"estimate":case["estimate"]}),
            &json!({"input_unit":case["input_unit"]}),
            default_registry(),
        );
        assert_eq!(result, case["result"], "{case}");
    }
}

#[test]
fn modified_composed_tables_require_fresh_expansion() {
    let fixture = fixture();
    let source = fixture["cases"][0]["source"].as_str().unwrap();
    let mut program = composition::compile(source, None, "", ExecutionLimits::default()).unwrap();
    program.claims["working"].statement = "Edited executable declaration".into();
    assert!(
        validation::validate(&program, default_registry())
            .iter()
            .any(|diagnostic| diagnostic.code == "stale_pattern_expansion")
    );
    let mut program = composition::compile(source, None, "", ExecutionLimits::default()).unwrap();
    program.arguments["first"].evidence.clear();
    let diagnostics = validation::validate(&program, default_registry());
    let stale = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "stale_pattern_expansion")
        .unwrap();
    assert_eq!(stale.declaration.as_deref(), Some("first"));
}

#[test]
fn host_json_and_declaration_limits_are_enforced() {
    let limits = ExecutionLimits {
        json_nodes: 2,
        ..ExecutionLimits::default()
    };
    assert!(validation::check_json_resources(&json!([1, 2]), &limits).is_err());
    let fixture = fixture();
    let source = fixture["cases"][0]["source"].as_str().unwrap();
    let mut program = composition::compile(source, None, "", ExecutionLimits::default()).unwrap();
    program.limits.declarations = 1;
    assert_eq!(
        validation::validate(&program, default_registry())[0].code,
        "resource_limit"
    );
}

#[test]
fn streaming_ir_bound_has_an_explicit_exact_boundary() {
    let fixture = fixture();
    let source = fixture["cases"][0]["source"].as_str().unwrap();
    let program = composition::compile(source, None, "", ExecutionLimits::default()).unwrap();
    let count = validation::structural_value_count(&program, usize::MAX).unwrap();
    assert!(count > 100);
    assert_eq!(
        validation::structural_value_count(&program, count).unwrap(),
        count
    );
    assert!(validation::structural_value_count(&program, count - 1).is_err());
    let mut limited = program;
    limited.limits.structural_values = 1;
    let result = validation::validate(&limited, default_registry());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, "resource_limit");
}

#[test]
fn scalar_result_comparison_preserves_exact_large_integer_order() {
    let fixture = fixture();
    let mut proposition: semantic::Proposition =
        serde_json::from_value(fixture["binding_proposition"].clone()).unwrap();
    proposition.unit = "Pa".into();
    proposition.result.operator = "==".into();
    proposition.result.expected = serde_json::from_str("9007199254740993").unwrap();
    let result = propositions::check_result(
        &proposition,
        "causal/1",
        &json!({"estimate":9007199254740992.0}),
        &json!({"input_unit":"Pa"}),
        default_registry(),
    );
    assert_eq!(result["holds"], json!(false));
    let result = propositions::check_result(
        &proposition,
        "causal/1",
        &json!({"estimate":9007199254740993u64}),
        &json!({"input_unit":"Pa"}),
        default_registry(),
    );
    assert_eq!(result["holds"], json!(true));
}

#[test]
fn ir_traversal_handles_the_configured_language_nesting_boundary() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let source = format!(
                "language \"EAL/3\"\n{}{}",
                "module m {\n".repeat(127),
                "}\n".repeat(127)
            );
            let program =
                composition::compile(&source, None, "", ExecutionLimits::default()).unwrap();
            let count =
                validation::structural_value_count(&program, program.limits.structural_values)
                    .unwrap();
            assert!(count > 127);
            assert!(validation::validate(&program, default_registry()).is_empty());
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn direct_compact_ir_is_relowered_before_execution() {
    let fixture = fixture();
    let source=fixture["cases"][0]["source"].as_str().unwrap().replace(
        "argument first = [evidence observation, assumptions stable] via measured => working",
        "pattern support(e: evidence, a: assumption, r: reasoning, c: claim) = [evidence e, assumptions a] via r => c\napply first = support(e=observation, a=stable, r=measured, c=working)",
    );
    let mut program = composition::compile(&source, None, "", ExecutionLimits::default()).unwrap();
    program.authored = None;
    assert!(validation::validate(&program, default_registry()).is_empty());
    program.arguments["first"].evidence.clear();
    let stale = validation::validate(&program, default_registry());
    assert!(
        stale
            .iter()
            .any(|diagnostic| diagnostic.code == "stale_pattern_expansion"
                && diagnostic.declaration.as_deref() == Some("first"))
    );
    let refreshed = composition::relower(&program).unwrap();
    assert!(validation::validate(&refreshed, default_registry()).is_empty());
}
