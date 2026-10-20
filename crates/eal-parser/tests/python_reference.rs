//! Compare generated Rust recognition with retained results from Python ANTLR.
use eal_parser::parse;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Reference {
    schema: String,
    source_revision: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    path: String,
    recognised: bool,
    tree: Option<String>,
    tokens: Option<Vec<Token>>,
}

#[derive(Debug, Deserialize)]
struct Token {
    #[serde(rename = "type")]
    token_type: i32,
    text: String,
    line: usize,
    column: usize,
    start: usize,
    end: usize,
    channel: i32,
}

fn reference() -> Reference {
    let reference: Reference =
        serde_json::from_str(include_str!("../../../fixtures/parser/reference.json")).unwrap();
    assert_eq!(reference.schema, "eal-parser-reference/1");
    assert_eq!(
        reference.source_revision,
        "e2d6f346a37f92483b9f9a13923b98feb053447b"
    );
    reference
}

fn fixture_source(case: &Case) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parser");
    std::fs::read_to_string(root.join(&case.path)).unwrap()
}

#[test]
fn accepted_sources_match_python_trees_tokens_and_unicode_positions() {
    let reference = reference();
    let mut compared = 0;
    for case in reference.cases.iter().filter(|case| case.recognised) {
        let source = fixture_source(case);
        let actual = parse(&source).unwrap_or_else(|error| {
            panic!("{}: Python accepted, Rust rejected: {error:?}", case.name)
        });
        assert_eq!(actual.tree, *case.tree.as_ref().unwrap(), "{}", case.name);
        let tokens: Vec<_> = actual
            .tokens
            .iter()
            .filter(|token| token.token_type != -1)
            .collect();
        let expected = case.tokens.as_ref().unwrap();
        assert_eq!(tokens.len(), expected.len(), "{}", case.name);
        for (index, (actual, expected)) in tokens.iter().zip(expected).enumerate() {
            let label = format!("{} token {index}", case.name);
            assert_eq!(actual.token_type, expected.token_type, "{label}");
            assert_eq!(actual.text, expected.text, "{label}");
            assert_eq!(actual.channel, expected.channel, "{label}");
            assert_eq!(actual.span.start.character, expected.start, "{label}");
            assert_eq!(actual.span.end.character, expected.end, "{label}");
            assert_eq!(actual.span.start.line, expected.line, "{label}");
            assert_eq!(actual.span.start.column, expected.column, "{label}");
            // Byte spans are a Rust API addition. Check them against source,
            // independently of the Python reference's scalar offset fields.
            let range = actual.span.start.byte..actual.span.end.byte;
            assert_eq!(source.get(range), Some(actual.text.as_str()), "{label}");
            assert_eq!(
                source[..actual.span.start.byte].chars().count(),
                expected.start,
                "{label}"
            );
            assert_eq!(
                source[..actual.span.end.byte].chars().count(),
                expected.end,
                "{label}"
            );
        }
        let eof = actual.tokens.last().expect("EOF token");
        assert_eq!(eof.token_type, -1, "{}", case.name);
        assert_eq!(eof.text, "<EOF>", "{}", case.name);
        assert_eq!(eof.span.start.byte, source.len(), "{}", case.name);
        assert_eq!(eof.span.start, eof.span.end, "{}", case.name);
        compared += 1;
    }
    assert!(
        compared > 80,
        "The accepted comparison corpus unexpectedly shrank"
    );
}

#[test]
fn grammar_errors_are_rejected_after_lexer_or_parser_recovery() {
    let reference = reference();
    let mut compared = 0;
    for case in reference.cases.iter().filter(|case| !case.recognised) {
        let error = match parse(&fixture_source(case)) {
            Ok(_) => panic!("{}: Python rejected, Rust accepted", case.name),
            Err(error) => error,
        };
        assert!(
            !error.diagnostics.is_empty(),
            "{}: rejection should explain the recognition failure",
            case.name
        );
        compared += 1;
    }
    assert!(
        compared >= 7,
        "The rejection comparison corpus unexpectedly shrank"
    );
}
