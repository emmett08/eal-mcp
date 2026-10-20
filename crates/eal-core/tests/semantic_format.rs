//! Checked semantic formatting retains source meaning and captured imports.
use eal_core::{ExecutionLimits, composition, methods, semantic_format, validation};
use serde_json::{Value, json};

fn compile(source: &str) -> eal_core::semantic::Program {
    composition::compile(source, None, "", ExecutionLimits::default()).unwrap()
}

const PLAIN: &str = r#"language "EAL/3"
environment lab {
require site == "bench"
}
tool runner {
version "1"
}
evidence observed {
tool runner
kind test
environment lab
max_age 60
input {"span": {"line": 19}, "ordinary": true}
require passed == true
}
reasoning relation {
method "structured/1"
rationale "The measured condition supports the bounded claim."
}
claim passed {
statement "The bounded condition holds."
environment lab
}
argument assess = [evidence observed] via relation => passed
"#;

#[test]
fn every_reference_case_is_checked_and_valid_sources_roundtrip() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/semantics/reference.json")).unwrap();
    let mut successes = 0;
    let mut rejected = 0;
    let mut input_errors = 0;
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let limits =
            serde_json::from_value(case.get("limits").cloned().unwrap_or_else(|| json!({})))
                .unwrap();
        let resolver = |path: &str| {
            case["imports"]
                .get(path)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| format!("No trusted source {path}"))
        };
        let program = match composition::compile(
            case["source"].as_str().unwrap(),
            case.get("imports")
                .map(|_| &resolver as &composition::SourceResolver<'_>),
            case["origin"].as_str().unwrap_or(""),
            limits,
        ) {
            Ok(program) => program,
            Err(_) => {
                input_errors += 1;
                continue;
            }
        };
        let valid = validation::validate_default(&program).is_empty();
        match semantic_format::format_default(&program) {
            Ok(output) if valid => {
                successes += 1;
                let replay = composition::compile(
                    &output,
                    case.get("imports")
                        .map(|_| &resolver as &composition::SourceResolver<'_>),
                    &program.root_origin,
                    program.limits.clone(),
                )
                .unwrap();
                assert_eq!(
                    semantic_format::semantic_ir(&program).unwrap(),
                    semantic_format::semantic_ir(&replay).unwrap(),
                    "{name}"
                );
                assert_eq!(
                    semantic_format::format_default(&replay).unwrap(),
                    output,
                    "{name}"
                );
            }
            Err(_) if !valid => {
                rejected += 1;
            }
            Err(error) => failures.push(format!("{name}: {error}")),
            Ok(_) => failures.push(format!("{name}: invalid programme was formatted")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!((successes, rejected, input_errors), (71, 18, 4));
}

#[test]
fn plain_direct_ir_has_a_faithful_fallback() {
    let variants = [
        PLAIN.to_owned(),
        format!("{PLAIN}objection challenge = [evidence observed] -x> argument assess\n"),
        format!("{PLAIN}rank assess 1 reviewed \"An ordering review.\"\n"),
        format!("{PLAIN}strict assess reviewed \"A bounded review.\"\n"),
    ];
    for source in variants {
        let mut program = compile(&source);
        program.authored = None;
        let diagnostics = validation::validate_default(&program);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let output = semantic_format::format_default(&program).unwrap();
        let reparsed = compile(&output);
        assert_eq!(
            semantic_format::semantic_ir(&program).unwrap(),
            semantic_format::semantic_ir(&reparsed).unwrap()
        );
        assert_eq!(semantic_format::format_default(&reparsed).unwrap(), output);
    }
}

#[test]
fn payload_keys_named_span_remain_semantic() {
    let original = compile(PLAIN);
    let mut altered = original.clone();
    altered.evidence["observed"].input["span"]["line"] = json!(20);
    assert_ne!(
        semantic_format::semantic_ir(&original).unwrap(),
        semantic_format::semantic_ir(&altered).unwrap()
    );
    let error = semantic_format::format_default(&altered).unwrap_err();
    assert!(
        error.to_string().contains("stale_pattern_expansion"),
        "{error}"
    );
}

#[test]
fn captured_imports_format_without_a_source_resolver() {
    let imported = r#"language "EAL/3"
environment lab {
require site == "bench"
}
"#;
    let source = r#"language "EAL/3"
import "support.eal" as support
claim result {
statement "A bounded imported condition."
environment support.lab
}
"#;
    let resolver = |path: &str| {
        if path == "trusted/support.eal" {
            Ok(imported.into())
        } else {
            Err(format!("Unexpected source {path}"))
        }
    };
    let program = composition::compile(
        source,
        Some(&resolver),
        "trusted/root.eal",
        ExecutionLimits::default(),
    )
    .unwrap();
    let output = semantic_format::format_default(&program).unwrap();
    assert!(output.contains("import \"support.eal\" as support"));
    assert_eq!(program.import_sources.len(), 1);
    assert!(
        semantic_format::semantic_ir(&program).unwrap()["imports"]
            .get("trusted/support.eal")
            .is_some()
    );
}

#[test]
fn direct_templates_require_authored_syntax() {
    let mut program = compile(&format!(
        "{PLAIN}pattern support(c: claim, r: reasoning, e: evidence) = [evidence e] via r => c\napply result = support(c=passed, r=relation, e=observed)\n"
    ));
    program.authored = None;
    let error = semantic_format::format_default(&program).unwrap_err();
    assert!(
        error.to_string().contains("FORMAT_AUTHORED_REQUIRED"),
        "{error}"
    );
}

#[test]
fn formatted_output_obeys_source_budgets() {
    let source = "language \"EAL/3\"\nenvironment lab{\nrequire ok\n}\n";
    let mut program = compile(source);
    assert!(semantic_format::format_default(&program).unwrap().len() > source.len());
    program.limits.source_bytes = source.len();
    let error = semantic_format::format_default(&program).unwrap_err();
    assert!(error.to_string().contains("FORMAT_RESOURCE"), "{error}");
    let mut program = compile(source);
    program.limits.source_tokens = eal_parser::parse(source).unwrap().tokens.len();
    let error = semantic_format::format_default(&program).unwrap_err();
    assert!(error.to_string().contains("FORMAT_RESOURCE"), "{error}");
    let source = format!(
        "language \"EAL/3\"\nenvironment lab {{\nrequire {}ok\n}}\n",
        "not ".repeat(127)
    );
    let program = compile(&source);
    let error = semantic_format::format_default(&program).unwrap_err();
    assert!(error.to_string().contains("FORMAT_RESOURCE"), "{error}");
    let mut program = compile(&source);
    program.limits.structural_values = 1;
    assert!(semantic_format::semantic_ir(&program).is_err());
    assert!(semantic_format::format_program(&program, methods::default_registry()).is_err());
}
