use eal_core::{
    DeclarationKind, Expression, Field, JsonValue, ParseLimits, parse, parse_with_limits,
};

#[test]
fn duplicates_and_missing_fields_are_rejected() {
    for source in [
        "language \"EAL/3\"\ntool t {\nversion \"1\"\nversion \"2\"\n}\n",
        "language \"EAL/3\"\ncontext version \"1\", version \"2\" {}\n",
        "language \"EAL/3\"\ncontext input {\"a\": 1, \"\\u0061\": 2} {}\n",
        "language \"EAL/3\"\ntool t {}\n",
        "language \"EAL/3\"\nargument a = [evidence e, evidence f] via r => c\n",
        "language \"EAL/3\"\ntool t {\nversion \"\\ud800\"\n}\n",
    ] {
        assert!(
            parse(source).is_err(),
            "Accepted malformed structure: {source}"
        );
    }
}

#[test]
fn context_defaults_are_scoped_and_authored_fields_stay_explicit() {
    let source = "language \"EAL/3\"\ncontext version \"1\" {\ntool t {}\n}\n";
    let program = parse(source).unwrap();
    let DeclarationKind::Context { body, .. } = &program.declarations[0].value else {
        panic!("context expected")
    };
    let DeclarationKind::Tool { fields, .. } = &body[0].value else {
        panic!("tool expected")
    };
    assert!(fields.is_empty());
    assert!(parse(&format!("{source}tool outside {{}}\n")).is_err());
}

#[test]
fn number_lexemes_and_unresolved_imports_are_retained() {
    let source = "language \"EAL/3\"\nimport \"library.eal\" as library\ncontext input [99999999999999999999999999999999999, -2.500e+30] {}\n";
    let program = parse(source).unwrap();
    assert!(
        program
            .format()
            .contains("99999999999999999999999999999999999")
    );
    assert!(program.format().contains("-2.500e+30"));
    assert!(program.equivalent_to(&parse(&program.format()).unwrap()));
}

#[test]
fn expressions_preserve_operator_grouping_and_field_convention() {
    for expression in [
        "a - (b - c)",
        "(a or b) and c",
        "a / (b * c)",
        "not (a == b)",
        "(a == b) == c",
        "\"a.b\" >= 1",
        "field(\"a[1]\") != null",
        "- - -1 + +2",
        "f(a + 1, [2, {\"x\": true}])",
    ] {
        let source = format!("language \"EAL/3\"\nenvironment e {{\nrequire {expression}\n}}\n");
        let program = parse(&source).unwrap();
        let reparsed = parse(&program.format()).unwrap();
        assert!(
            program.equivalent_to(&reparsed),
            "{expression}: {}",
            program.format()
        );
    }
    let program = parse("language \"EAL/3\"\nenvironment e {\nrequire \"a.b\" == 2\n}\n").unwrap();
    let DeclarationKind::Environment { predicates, .. } = &program.declarations[0].value else {
        panic!()
    };
    let Expression::Binary { left, .. } = &predicates[0] else {
        panic!()
    };
    assert!(matches!(left.as_ref(), Expression::Field { name } if name == "a.b"));
}

#[test]
fn flat_operator_chains_have_a_bounded_typed_depth() {
    let source = format!(
        "language \"EAL/3\"\nenvironment e {{\nrequire {}\n}}\n",
        vec!["1"; 1_000].join("+")
    );
    let error = parse(&source).unwrap_err().to_string();
    assert!(error.contains("EXPRESSION_DEPTH"), "{error}");
    let source = format!(
        "language \"EAL/3\"\nenvironment e {{\nrequire {}\n}}\n",
        vec!["1"; 100].join("+")
    );
    let program = parse(&source).unwrap();
    let formatted = program.format();
    assert!(program.equivalent_to(&parse(&formatted).unwrap()));
}

#[test]
fn deeply_parenthesised_sources_use_the_bounded_lowering_worker() {
    let source = format!(
        "language \"EAL/3\"\nenvironment e {{\nrequire {}1{}\n}}\n",
        "(".repeat(110),
        ")".repeat(110)
    );
    let limits = ParseLimits {
        max_nesting: 128,
        ..ParseLimits::default()
    };
    let program = parse_with_limits(&source, limits).unwrap();
    assert!(program.equivalent_to(&parse(&program.format()).unwrap()));
}

#[test]
fn lexical_equality_detects_values_omitted_by_a_formatter() {
    let mut original = parse("language \"EAL/3\"\ncontext input [1] {}\n").unwrap();
    let mut changed = original.clone();
    let DeclarationKind::Context { defaults, .. } = &mut changed.declarations[0].value else {
        panic!()
    };
    let Field::Json {
        value: JsonValue::Array(values),
        ..
    } = &mut defaults[0]
    else {
        panic!()
    };
    values.push(JsonValue::Boolean(true));
    assert!(!original.equivalent_to(&changed));
    original.declarations[0].span.start.line = 9;
    assert!(original.equivalent_to(&parse("language \"EAL/3\"\ncontext input [1] {}\n").unwrap()));
}

#[test]
fn checked_format_reports_output_expansion_beyond_source_budget() {
    let mut source = "language \"EAL/3\"\n".to_owned();
    for index in 0..60 {
        source.push_str(&format!("module m{index} {{\n"));
    }
    for index in 0..2_000 {
        source.push_str(&format!("tool t{index} {{\nversion \"1\"\n}}\n"));
    }
    source.push_str(&"}\n".repeat(60));
    let limits = ParseLimits::default();
    assert!(source.len() < limits.max_source_bytes);
    let program = parse_with_limits(&source, limits).unwrap();
    assert!(program.format().len() > limits.max_source_bytes);
    let error = program.format_with_limits(limits).unwrap_err();
    let eal_core::CoreError::Recognition(error) = error else {
        panic!("expected an output resource diagnostic")
    };
    assert!(
        error
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.stage == eal_parser::DiagnosticStage::Resource })
    );
}

#[test]
fn checked_format_reports_output_expansion_beyond_token_budget() {
    let source = "language \"EAL/3\"\nenvironment e {\nrequire a\nrequire b\n}\n";
    let limits = ParseLimits {
        max_tokens: 20,
        ..ParseLimits::default()
    };
    let program = parse_with_limits(source, limits).unwrap();
    assert!(program.format_with_limits(limits).is_err());
    let formatted = program.format_with_limits(ParseLimits::default()).unwrap();
    assert!(program.equivalent_to(&parse(&formatted).unwrap()));
}

#[test]
fn checked_format_reports_property_path_nesting_expansion() {
    let source = format!(
        "language \"EAL/3\"\nenvironment e {{\nrequire {}x\n}}\n",
        "not ".repeat(63)
    );
    let program = parse(&source).unwrap();
    assert!(program.format_with_limits(ParseLimits::default()).is_err());
    let limits = ParseLimits {
        max_nesting: 65,
        ..ParseLimits::default()
    };
    let formatted = program.format_with_limits(limits).unwrap();
    assert!(program.equivalent_to(&parse_with_limits(&formatted, limits).unwrap()));
}
