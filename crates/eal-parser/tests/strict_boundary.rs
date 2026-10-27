//! Adversarial checks for recognition diagnostics and resource limits.
use eal_parser::{DiagnosticStage, ParseLimits, parse, parse_with_limits};

#[test]
fn lexer_errors_are_never_accepted_after_recovery() {
    let error = parse("language \"EAL/4\"\n@\n").unwrap_err();
    assert!(
        error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.stage == DiagnosticStage::Lexer)
    );
}

#[test]
fn parser_recovery_does_not_produce_a_document() {
    let error = parse("language \"EAL/4\"\nclaim broken { statement \"x\"\n").unwrap_err();
    assert!(
        error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.stage == DiagnosticStage::Parser)
    );
}

#[test]
fn bad_token_after_unicode_has_scalar_column_and_byte_position() {
    let source = "language \"EAL/4\"\n/* λ😀 */ @\n";
    let error = parse(source).unwrap_err();
    let span = error.diagnostics[0].span;
    assert_eq!(span.start.line, 2);
    assert_eq!(span.start.column, 10);
    assert_eq!(span.start.byte, source.find('@').unwrap());
    assert_eq!(
        span.start.character,
        source[..source.find('@').unwrap()].chars().count()
    );
}

#[test]
fn source_size_is_checked_before_allocation() {
    let limits = ParseLimits {
        max_source_bytes: 2,
        ..ParseLimits::default()
    };
    let error = parse_with_limits("language \"EAL/4\"\n", limits).unwrap_err();
    assert_eq!(error.diagnostics[0].stage, DiagnosticStage::Resource);
}

#[test]
fn token_limit_includes_comments_and_eof() {
    let limits = ParseLimits {
        max_tokens: 3,
        ..ParseLimits::default()
    };
    assert!(parse_with_limits("language \"EAL/4\"\n// comment\n", limits).is_err());
}

#[test]
fn excessive_delimiter_recursion_is_rejected_before_generated_parser() {
    let source = format!(
        "language \"EAL/4\"\nenvironment e {{ require {}true{}\n}}\n",
        "(".repeat(500),
        ")".repeat(500)
    );
    let error = parse(&source).unwrap_err();
    assert_eq!(error.diagnostics[0].stage, DiagnosticStage::Resource);
}

#[test]
fn excessive_unary_recursion_is_rejected_before_generated_parser() {
    let source = format!(
        "language \"EAL/4\"\nenvironment e {{ require {}true\n}}\n",
        "not ".repeat(500)
    );
    let error = parse(&source).unwrap_err();
    assert_eq!(error.diagnostics[0].stage, DiagnosticStage::Resource);
}

#[test]
fn zero_diagnostic_limit_still_rejects_invalid_source() {
    let limits = ParseLimits {
        max_diagnostics: 0,
        ..ParseLimits::default()
    };
    let error = parse_with_limits("language \"EAL/4\"\n@", limits).unwrap_err();
    assert_eq!(error.diagnostics.len(), 1);
}

#[test]
fn node_limit_rejects_before_returning_partial_syntax() {
    let limits = ParseLimits {
        max_nodes: 1,
        ..ParseLimits::default()
    };
    let error = parse_with_limits("language \"EAL/4\"\n", limits).unwrap_err();
    assert_eq!(error.diagnostics[0].stage, DiagnosticStage::Resource);
}

#[test]
fn accepted_recursion_boundary_is_safe_on_a_small_embedding_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for depth in [63, 127] {
                let limits = ParseLimits {
                    max_nesting: depth + 1,
                    ..ParseLimits::default()
                };
                for expression in [
                    format!("{}true{}", "(".repeat(depth), ")".repeat(depth)),
                    format!("{}true", "not ".repeat(depth)),
                    format!("{}true{}", "not (".repeat(depth / 2), ")".repeat(depth / 2)),
                ] {
                    let source =
                        format!("language \"EAL/4\"\nenvironment e {{ require {expression}\n}}\n");
                    parse_with_limits(&source, limits).unwrap();
                }
                let source = format!(
                    "language \"EAL/4\"\nevidence e {{ input {}true{}\n}}\n",
                    "[".repeat(depth),
                    "]".repeat(depth)
                );
                parse_with_limits(&source, limits).unwrap();
                let source = format!(
                    "language \"EAL/4\"\n{}{}\n",
                    "context kind demo {\n".repeat(depth + 1),
                    "}\n".repeat(depth + 1)
                );
                parse_with_limits(&source, limits).unwrap();
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn a_large_configured_depth_cannot_exceed_the_hard_stack_bound() {
    let source = format!(
        "language \"EAL/4\"\nenvironment e {{ require {}true{}\n}}\n",
        "(".repeat(128),
        ")".repeat(128)
    );
    let limits = ParseLimits {
        max_nesting: usize::MAX,
        ..ParseLimits::default()
    };
    let error = parse_with_limits(&source, limits).unwrap_err();
    assert_eq!(error.diagnostics[0].stage, DiagnosticStage::Resource);
}

#[test]
fn long_single_tokens_respect_the_source_budget() {
    let limits = ParseLimits {
        max_source_bytes: 2048,
        ..ParseLimits::default()
    };
    let within = format!(
        "language \"EAL/4\"\nevidence e {{ input {}\n}}\n",
        "1".repeat(1900)
    );
    parse_with_limits(&within, limits).unwrap();
    let within = format!(
        "language \"EAL/4\"\nclaim c {{ statement \"{}\"\n}}\n",
        "λ".repeat(900)
    );
    parse_with_limits(&within, limits).unwrap();
    let oversized_number = format!(
        "language \"EAL/4\"\nevidence e {{ input {}\n}}\n",
        "1".repeat(2100)
    );
    let oversized_string = format!(
        "language \"EAL/4\"\nclaim c {{ statement \"{}\"\n}}\n",
        "λ".repeat(1100)
    );
    for source in [oversized_number, oversized_string] {
        let error = parse_with_limits(&source, limits).unwrap_err();
        assert_eq!(error.diagnostics[0].stage, DiagnosticStage::Resource);
    }
}

#[test]
fn unary_operators_retained_across_delimiters_share_one_depth_budget() {
    let prefix = format!("{}(", "not ".repeat(60));
    let source = format!(
        "language \"EAL/4\"\nenvironment e {{ require {}true{}\n}}\n",
        prefix.repeat(60),
        ")".repeat(60)
    );
    let limits = ParseLimits {
        max_nesting: 128,
        ..ParseLimits::default()
    };
    let error = parse_with_limits(&source, limits).unwrap_err();
    assert_eq!(error.diagnostics[0].stage, DiagnosticStage::Resource);
}

#[test]
fn unary_operators_before_qualified_calls_share_one_depth_budget() {
    let prefix = format!("{}f.g(", "not ".repeat(60));
    let source = format!(
        "language \"EAL/4\"\nenvironment e {{ require {}true{}\n}}\n",
        prefix.repeat(60),
        ")".repeat(60)
    );
    let error = parse(&source).unwrap_err();
    assert_eq!(error.diagnostics[0].stage, DiagnosticStage::Resource);
}

#[test]
fn mismatched_delimiters_cannot_reduce_the_recursion_budget() {
    let source = "language \"EAL/4\"\nenvironment e { require (}true\n}\n";
    let error = parse(source).unwrap_err();
    assert_eq!(error.diagnostics[0].stage, DiagnosticStage::Parser);
}
