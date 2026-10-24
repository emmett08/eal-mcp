//! Native static validation, typed input correspondence and dimensional results.
use eal_core::{
    ExecutionLimits, composition, methods::default_registry, propositions, semantic, validation,
};
use serde_json::json;
const SOURCE: &str = include_str!("support/validation.eal");
fn proposition() -> semantic::Proposition {
    serde_json::from_value(json!({"subject":"pump-A","quantity":"pressure","unit":"kPa","scope":"experiment-v1","valid_from":"2026-09-23T10:00:00Z","valid_until":"2026-09-23T11:00:00Z","result":{"path":"estimate","operator":">=","expected":5,"expression":null},"query":{"assignment":"randomised"}})).unwrap()
}

#[test]
fn modified_composed_tables_require_fresh_expansion() {
    let source = SOURCE;
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
    let source = SOURCE;
    let mut program = composition::compile(source, None, "", ExecutionLimits::default()).unwrap();
    program.limits.declarations = 1;
    assert_eq!(
        validation::validate(&program, default_registry())[0].code,
        "resource_limit"
    );
}

#[test]
fn streaming_ir_bound_has_an_explicit_exact_boundary() {
    let source = SOURCE;
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
    let mut proposition = proposition();
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
    let source=SOURCE.replace(
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

#[test]
fn typed_input_preserves_query_identity_and_rejects_scope_changes() {
    let proposition = proposition();
    let mut envelope = json!({"schema":"EAL/typed-input/1","method":"causal/1","subject":"pump-A","quantity":"pressure","unit":"Pa","scope":"experiment-v1","valid_from":"2026-09-23T10:00:00Z","valid_until":"2026-09-23T11:00:00Z","payload":{"assignment":"randomised","treatment":[10000,12000],"control":[3000,4000]}});
    let (payload, trace) = propositions::prepare_binding(
        &proposition,
        "causal/1",
        "trial",
        &envelope,
        default_registry(),
    );
    assert_eq!(payload.unwrap(), envelope["payload"]);
    assert_eq!(trace["formal_query"], envelope["payload"]);
    assert_eq!(trace["status"], "bound");
    envelope["scope"] = json!("another-experiment");
    let (payload, trace) = propositions::prepare_binding(
        &proposition,
        "causal/1",
        "trial",
        &envelope,
        default_registry(),
    );
    assert!(payload.is_none());
    assert_eq!(trace["status"], "unsupported");
}

#[test]
fn rational_unit_conversion_retains_thresholds_and_requires_an_output() {
    let proposition = proposition();
    for (estimate, holds) in [(4999, false), (5000, true), (5001, true)] {
        let result = propositions::check_result(
            &proposition,
            "causal/1",
            &json!({"estimate":estimate}),
            &json!({"input_unit":"Pa"}),
            default_registry(),
        );
        assert_eq!(result["holds"], holds);
    }
    let result = propositions::check_result(
        &proposition,
        "causal/1",
        &json!({}),
        &json!({"input_unit":"Pa"}),
        default_registry(),
    );
    assert_eq!(result["status"], "unsupported");
}

#[test]
fn iso_instants_preserve_timezone_equivalence_and_half_open_precision() {
    let utc = validation::parse_time("2026-09-23T12:00:00.000001Z").unwrap();
    let shifted = validation::parse_time("2026-09-23T13:00:00.000001+01:00").unwrap();
    assert_eq!(utc, shifted);
    let previous = validation::parse_time("2026-09-23T12:00:00Z").unwrap();
    assert_eq!((utc - previous).num_microseconds(), Some(1));
    assert!(validation::parse_time("2026-09-23T12:00:00").is_err());
    assert!(validation::parse_time("2026-13-01T12:00:00Z").is_err());
}
