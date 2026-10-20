//! Limits checked before recursive generated recognition begins.
use crate::{DiagnosticStage, ParseDiagnostic, ParseError, SourceSpan, SyntaxToken};

/// Operator-selected parser limits. Nesting is capped at 128 for stack safety.
#[derive(Clone, Copy, Debug)]
pub struct ParseLimits {
    pub max_source_bytes: usize,
    pub max_tokens: usize,
    pub max_nesting: usize,
    pub max_nodes: usize,
    pub max_diagnostics: usize,
}
impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 1_048_576,
            max_tokens: 100_000,
            max_nesting: 64,
            max_nodes: 1_000_000,
            max_diagnostics: 32,
        }
    }
}

pub(crate) fn resource_error(message: impl Into<String>, span: SourceSpan) -> ParseError {
    ParseError {
        diagnostics: vec![ParseDiagnostic {
            stage: DiagnosticStage::Resource,
            message: message.into(),
            span,
        }],
    }
}

pub(crate) fn check_recursion(
    tokens: &[SyntaxToken],
    limits: ParseLimits,
) -> Result<(), ParseError> {
    let mut delimiter_unary: Vec<(&str, usize)> = Vec::new();
    let mut retained_unary = 0usize;
    let mut unary_run = 0usize;
    let ceiling = limits.max_nesting.min(128);
    let visible: Vec<_> = tokens.iter().filter(|token| token.channel == 0).collect();
    for (index, token) in visible.iter().enumerate() {
        match token.text.as_str() {
            "not" | "+" | "-" => {
                unary_run += 1;
            }
            "{" | "[" | "(" => {
                // Unary operations before a parenthesised operand remain on
                // the parser stack until that operand's closing delimiter.
                let call_unary = if token.text == "(" {
                    call_prefix_unary(&visible, index)
                } else {
                    0
                };
                let pending = unary_run + call_unary;
                let expected = match token.text.as_str() {
                    "(" => ")",
                    "[" => "]",
                    _ => "}",
                };
                delimiter_unary.push((expected, pending));
                retained_unary += pending;
                unary_run = 0;
            }
            "}" | "]" | ")" => {
                match delimiter_unary.pop() {
                    Some((expected, pending)) if expected == token.text => {
                        retained_unary -= pending;
                    }
                    _ => {
                        return Err(ParseError {
                            diagnostics: vec![ParseDiagnostic {
                                stage: DiagnosticStage::Parser,
                                message: "unmatched or mismatched closing delimiter".to_owned(),
                                span: token.span,
                            }],
                        });
                    }
                }
                unary_run = 0;
            }
            _ => {
                unary_run = 0;
            }
        }
        let depth = delimiter_unary
            .len()
            .saturating_add(retained_unary)
            .saturating_add(unary_run);
        if depth > ceiling {
            return Err(resource_error(
                format!("parser nesting limit exceeded ({ceiling})"),
                token.span,
            ));
        }
    }
    Ok(())
}

// A unary operand may start with a qualified function name rather than `(`.
// Retain its unary stack until the call closes. Accepting every word token as a
// candidate is conservative for malformed input and leaves grammar decisions to
// ANTLR; `not` itself remains an operator.
fn call_prefix_unary(tokens: &[&SyntaxToken], open: usize) -> usize {
    fn name(token: &SyntaxToken) -> bool {
        token.text != "not"
            && token
                .text
                .as_bytes()
                .first()
                .is_some_and(|first| first.is_ascii_alphabetic() || *first == b'_')
    }
    let mut cursor = open;
    if cursor == 0 || !name(tokens[cursor - 1]) {
        return 0;
    }
    cursor -= 1;
    while cursor >= 2 && tokens[cursor - 1].text == "." && name(tokens[cursor - 2]) {
        cursor -= 2;
    }
    let end = cursor;
    while cursor > 0 && matches!(tokens[cursor - 1].text.as_str(), "not" | "+" | "-") {
        cursor -= 1;
    }
    end - cursor
}
