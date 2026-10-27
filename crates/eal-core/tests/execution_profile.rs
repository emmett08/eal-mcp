//! Execution dependencies and pure derivations preserve meaning across compilation.
use eal_core::{
    ExecutionLimits, composition, methods, planning::EvidencePlanner, semantic_format, validation,
};

const SOURCE: &str = r#"language "EAL/3" profile "execution/1"
environment lab {
  require site == "bench"
}
tool reader {
  version "1"
}
evidence checked {
  derive object("count", count(items))
  kind calculation
  environment lab
  max_age 60
  input {"items": null}
  bind "/items" from observed "/items"
  require count == 3
}
evidence observed {
  tool reader
  kind observation
  environment lab
  max_age 60
  input {}
  require complete == true
}
reasoning composition {
  method "structured/1"
  rationale "The source and exact count establish this finite observation."
}
claim three {
  statement "The finite source contains three items."
  environment lab
}
argument conclude = [evidence checked] via composition => three
"#;

fn compile(source: &str) -> eal_core::semantic::Program {
    composition::compile(source, None, "", ExecutionLimits::default()).unwrap()
}
fn codes(source: &str) -> Vec<String> {
    validation::validate_default(&compile(source))
        .into_iter()
        .map(|d| d.code)
        .collect()
}

#[test]
fn profile_bindings_and_derivations_round_trip_both_views() {
    let authored = eal_core::parse(SOURCE).unwrap();
    let formatted = authored
        .format_with_limits(eal_core::ParseLimits::default())
        .unwrap();
    assert!(authored.equivalent_to(&eal_core::parse(&formatted).unwrap()));
    let mut program = compile(SOURCE);
    assert!(validation::validate_default(&program).is_empty());
    let expected = semantic_format::semantic_ir(&program).unwrap();
    for retained_syntax in [true, false] {
        if !retained_syntax {
            program.authored = None;
        }
        let formatted =
            semantic_format::format_program(&program, methods::default_registry()).unwrap();
        assert_eq!(
            semantic_format::semantic_ir(&compile(&formatted)).unwrap(),
            expected
        );
    }
}

#[test]
fn planner_includes_transitive_acquisitions_and_counts_only_external_calls() {
    let program = compile(SOURCE);
    let plan = EvidencePlanner::new(&program)
        .unwrap()
        .plan("three")
        .unwrap();
    assert_eq!(plan.evidence_ids(), ["checked", "observed"]);
    assert_eq!(plan.estimated_calls(), 1);
    assert_eq!(
        eal_core::acquisition::dependency_order(&program, &["checked".into()]).unwrap(),
        ["observed", "checked"]
    );
}

#[test]
fn execution_features_require_a_known_declared_profile() {
    for source in [
        SOURCE.replace(" profile \"execution/1\"", ""),
        SOURCE.replace("execution/1", "execution/2"),
    ] {
        let error = eal_core::parse(&source).unwrap_err();
        assert!(error.to_string().contains("EXECUTION_PROFILE"), "{error}");
    }
    let literal = SOURCE
        .replace(" profile \"execution/1\"", "")
        .replace("derive object(\"count\", count(items))", "tool reader")
        .replace("  bind \"/items\" from observed \"/items\"\n", "");
    assert!(codes(&literal).is_empty());
}

#[test]
fn sources_require_exactly_one_external_tool_or_pure_derivation() {
    for source in [
        SOURCE.replace("  derive object(\"count\", count(items))\n", ""),
        SOURCE.replace("  derive object", "  tool reader\n  derive object"),
    ] {
        assert!(
            eal_core::parse(&source)
                .unwrap_err()
                .to_string()
                .contains("EVIDENCE_SOURCE")
        );
    }
}

#[test]
fn unknown_references_cycles_and_scopes_are_static_errors() {
    assert!(
        codes(&SOURCE.replace("from observed", "from missing"))
            .iter()
            .any(|c| c == "unknown_reference")
    );
    assert!(
        codes(&SOURCE.replace("from observed", "from checked"))
            .iter()
            .any(|c| c == "acquisition_dependency")
    );
    let wrong_scope = SOURCE
        .replace(
            "evidence observed {",
            "environment elsewhere {\nrequire site == \"elsewhere\"\n}\nevidence observed {",
        )
        .replace(
            "kind observation\n  environment lab",
            "kind observation\n  environment elsewhere",
        );
    assert!(
        codes(&wrong_scope)
            .iter()
            .any(|c| c == "environment_mismatch")
    );
}

#[test]
fn malformed_missing_or_overlapping_destinations_are_rejected() {
    for source in [
        SOURCE.replace("bind \"/items\"", "bind \"/missing\""),
        SOURCE.replace("bind \"/items\"", "bind \"/it~2ems\""),
        SOURCE.replace(
            "  require count",
            "  bind \"\" from observed \"\"\n  require count",
        ),
    ] {
        assert!(
            codes(&source)
                .iter()
                .any(|c| c.starts_with("input_binding_"))
        );
    }
    assert!(
        codes(&SOURCE.replace("from observed \"/items\"", "from observed \"#fragment\""))
            .iter()
            .any(|c| c == "input_binding_pointer")
    );
}

#[test]
fn invalid_derivation_types_and_duplicate_object_keys_are_rejected() {
    for expression in [
        "count(items)",
        "object(\"n\", 1, \"n\", 2)",
        "object(2, 1)",
        "object(\"n\")",
        "object(\"n\", quantity(1, \"m\"))",
    ] {
        let source = SOURCE.replace("object(\"count\", count(items))", expression);
        assert!(
            codes(&source).iter().any(|c| c == "invalid_derivation"),
            "{expression}"
        );
    }
}

#[test]
fn lexical_modules_resolve_binding_sources_without_global_aliases() {
    let source = SOURCE.replacen("environment lab", "module scoped {\nenvironment lab", 1) + "}\n";
    let program = compile(&source);
    assert!(validation::validate_default(&program).is_empty());
    assert_eq!(
        program.evidence["scoped.checked"].bindings[0].evidence,
        "scoped.observed"
    );
}

#[test]
fn profile_changes_cannot_reinterpret_retained_authored_syntax() {
    let mut program = compile(SOURCE);
    program.profile = None;
    let diagnostics = validation::validate_default(&program);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == "stale_pattern_expansion")
    );
    assert!(diagnostics.iter().any(|d| d.code == "execution_profile"));
}

#[test]
fn executable_digest_keeps_semantics_and_source_identity_separate() {
    let original = compile(SOURCE);
    let whitespace = compile(&format!(
        "// Retained source provenance differs.\n{SOURCE}\n"
    ));
    assert_ne!(original.source_digest, whitespace.source_digest);
    let digest = |p: &eal_core::semantic::Program| {
        semantic_format::executable_digest(p, methods::default_registry()).unwrap()
    };
    assert_eq!(digest(&original), digest(&whitespace));
    assert_ne!(
        digest(&original),
        digest(&compile(
            &SOURCE.replace("require count == 3", "require count == 4")
        ))
    );
    assert_ne!(
        digest(&original),
        digest(&compile(&SOURCE.replace(
            "from observed \"/items\"",
            "from observed \"/other\""
        )))
    );
}

#[test]
fn escaped_pointer_keys_remain_distinct_and_root_binding_is_valid() {
    let escaped = SOURCE
        .replace(
            "input {\"items\": null}",
            "input {\"a/b\": null, \"a~b\": null}",
        )
        .replace(
            "bind \"/items\" from observed \"/items\"",
            "bind \"/a~1b\" from observed \"/items\"\n  bind \"/a~0b\" from observed \"/items\"",
        )
        .replace(
            "derive object(\"count\", count(items))",
            "derive object(\"count\", 3)",
        );
    assert!(codes(&escaped).is_empty());
    let root = SOURCE.replace(
        "bind \"/items\" from observed \"/items\"",
        "bind \"\" from observed \"\"",
    );
    assert!(codes(&root).is_empty());
}

#[test]
fn input_derived_fields_do_not_inherit_an_external_tool() {
    let source = SOURCE
        .replace(
            "evidence checked {",
            "context tool reader {\nevidence checked {",
        )
        .replace("evidence observed {", "}\nevidence observed {");
    let program = compile(&source);
    assert!(validation::validate_default(&program).is_empty());
    assert!(program.evidence["checked"].tool.is_none());
}

#[test]
fn input_bindings_obey_json_depth_budget_before_collection() {
    let source = SOURCE.replace("from observed \"/items\"", "from observed \"/a/b/c\"");
    let mut program = compile(&source);
    program.limits.json_depth = 2;
    assert!(
        validation::validate_default(&program)
            .iter()
            .any(|d| d.code == "input_binding_depth")
    );
}

#[test]
fn imported_execution_features_require_the_root_profile_and_keep_lexical_identity() {
    let resolver = |_: &str| Ok(SOURCE.to_owned());
    let root = "language \"EAL/3\" profile \"execution/1\"\nimport \"case.eal\" as imported\n";
    let program = composition::compile(
        root,
        Some(&resolver),
        "root.eal",
        ExecutionLimits::default(),
    )
    .unwrap();
    assert!(validation::validate_default(&program).is_empty());
    assert_eq!(
        program.evidence["imported.checked"].bindings[0].evidence,
        "imported.observed"
    );
    let unprofiled = root.replace(" profile \"execution/1\"", "");
    let program = composition::compile(
        &unprofiled,
        Some(&resolver),
        "root.eal",
        ExecutionLimits::default(),
    )
    .unwrap();
    assert!(
        validation::validate_default(&program)
            .iter()
            .any(|d| d.code == "execution_profile")
    );
}

#[test]
fn pattern_dependencies_bind_to_arguments_and_remain_separate_acquisitions() {
    let source = format!(
        r#"{SOURCE}
pattern check_count(src: evidence, env: environment, goal: claim, inference: reasoning) {{
 evidence checked_count {{
  derive object("count", count(items))
  kind calculation
  environment env
  max_age 60
  input {{"items": null}}
  bind "/items" from src "/items"
  require count == 3
 }}
 argument support = [evidence checked_count] via inference => goal
}}
apply first = check_count(src=observed, env=lab, goal=three, inference=composition)
apply second = check_count(src=observed, env=lab, goal=three, inference=composition)
"#
    );
    let program = compile(&source);
    let diagnostics = validation::validate_default(&program);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    for name in ["first.checked_count", "second.checked_count"] {
        assert_eq!(program.evidence[name].bindings[0].evidence, "observed");
    }
    let plan = EvidencePlanner::new(&program)
        .unwrap()
        .plan("three")
        .unwrap();
    assert_eq!(plan.estimated_calls(), 1);
    assert_eq!(plan.closure.evidence.len(), 4);
}
