//! Strict ANTLR boundary and external lowering to owned concrete syntax.
use crate::diagnostics::SourceMap;
use crate::generated::eallexer::EALLexer;
use crate::generated::ealparser::{self, EALParser, EALParserContext, EALParserContextType};
use crate::limits::{check_recursion, resource_error};
use crate::{
    DiagnosticStage, ParseDiagnostic, ParseError, ParseLimits, SourceSpan, SyntaxDocument,
    SyntaxKind, SyntaxNode, SyntaxToken,
};
use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::error_listener::ErrorListener;
use antlr4rust::errors::ANTLRError;
use antlr4rust::parser::Parser;
use antlr4rust::recognizer::Recognizer;
use antlr4rust::token::{TOKEN_EOF, Token};
use antlr4rust::token_factory::TokenFactory;
use antlr4rust::tree::TerminalNode;
use antlr4rust::{InputStream, TidExt, TokenSource};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
struct DiagnosticListener {
    diagnostics: Rc<RefCell<Vec<ParseDiagnostic>>>,
    map: Rc<SourceMap>,
    stage: DiagnosticStage,
    max_diagnostics: usize,
}
impl<'input, T: Recognizer<'input>> ErrorListener<'input, T> for DiagnosticListener {
    fn syntax_error(
        &self,
        _recognizer: &T,
        offending_symbol: Option<&<T::TF as TokenFactory<'input>>::Inner>,
        line: isize,
        column: isize,
        message: &str,
        _error: Option<&ANTLRError>,
    ) {
        let mut diagnostics = self.diagnostics.borrow_mut();
        // Always retain at least one error, even if the caller requests zero.
        if diagnostics.len() < self.max_diagnostics.max(1) {
            let span = offending_symbol
                .map(|token| self.map.span(token.get_start(), token.get_stop()))
                .unwrap_or_else(|| self.map.at_line_column(line, column));
            diagnostics.push(ParseDiagnostic {
                stage: self.stage.clone(),
                message: message.to_owned(),
                span,
            });
        }
    }
}

/// Recognise EAL with conservative default resource limits.
pub fn parse(source: &str) -> Result<SyntaxDocument, ParseError> {
    parse_with_limits(source, ParseLimits::default())
}

/// Recognise syntax only. Every lexer and parser diagnostic prevents a document.
///
/// The lexer uses Unicode code points rather than UTF-8 bytes, matching the
/// Python reference. Bounds are checked before entering recursive parser rules.
pub fn parse_with_limits(source: &str, limits: ParseLimits) -> Result<SyntaxDocument, ParseError> {
    if source.len() > limits.max_source_bytes {
        return Err(resource_error(
            format!("source byte limit exceeded ({})", limits.max_source_bytes),
            SourceSpan::default(),
        ));
    }
    // Generated recognisers use deep native recursion even for modest EAL
    // expression nesting. A fixed stack makes the hard depth cap independent
    // of the embedding application's (often 2 MiB) caller stack.
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("eal-parser".to_owned())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, move || recognise(source, limits))
            .map_err(|error| {
                resource_error(
                    format!("could not start parser worker: {error}"),
                    SourceSpan::default(),
                )
            })?;
        worker.join().map_err(|_| ParseError {
            diagnostics: vec![ParseDiagnostic {
                stage: DiagnosticStage::Parser,
                message: "parser worker failed internally".to_owned(),
                span: SourceSpan::default(),
            }],
        })?
    })
}

fn recognise(source: &str, limits: ParseLimits) -> Result<SyntaxDocument, ParseError> {
    let map = Rc::new(SourceMap::new(source));
    let input: Vec<u32> = source.chars().map(u32::from).collect();
    let diagnostics = Rc::new(RefCell::new(Vec::new()));
    let listener = DiagnosticListener {
        diagnostics: diagnostics.clone(),
        map: map.clone(),
        stage: DiagnosticStage::Lexer,
        max_diagnostics: limits.max_diagnostics,
    };
    let mut lexer = EALLexer::new(InputStream::new(input.as_slice()));
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(listener.clone()));
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        let token_type = token.get_token_type();
        let span = map.span(token.get_start(), token.get_stop());
        if tokens.len() >= limits.max_tokens {
            return Err(resource_error(
                format!("token limit exceeded ({})", limits.max_tokens),
                span,
            ));
        }
        let kind = if token_type == TOKEN_EOF {
            "EOF"
        } else {
            ealparser::_SYMBOLIC_NAMES
                .get(token_type as usize)
                .copied()
                .flatten()
                .or_else(|| {
                    ealparser::_LITERAL_NAMES
                        .get(token_type as usize)
                        .copied()
                        .flatten()
                })
                .unwrap_or("unknown")
        };
        tokens.push(SyntaxToken {
            token_type,
            kind: kind.to_owned(),
            text: if token_type == TOKEN_EOF {
                "<EOF>".to_owned()
            } else {
                token.get_text().to_string()
            },
            channel: token.get_channel(),
            span,
        });
        if token_type == TOKEN_EOF {
            break;
        }
    }
    if !diagnostics.borrow().is_empty() {
        return Err(ParseError {
            diagnostics: diagnostics.borrow().clone(),
        });
    }
    check_recursion(&tokens, limits)?;

    // Preflight completes lexing before any recursive recognition. A fresh
    // deterministic lexer feeds the generated parser without retaining its API.
    let mut lexer = EALLexer::new(InputStream::new(input.as_slice()));
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(listener));
    let mut parser = EALParser::new(CommonTokenStream::new(lexer));
    parser.remove_error_listeners();
    parser.add_error_listener(Box::new(DiagnosticListener {
        diagnostics: diagnostics.clone(),
        map: map.clone(),
        stage: DiagnosticStage::Parser,
        max_diagnostics: limits.max_diagnostics,
    }));
    let parsed = parser.program();
    if !diagnostics.borrow().is_empty() {
        return Err(ParseError {
            diagnostics: diagnostics.borrow().clone(),
        });
    }
    let root = parsed.map_err(|error| ParseError {
        diagnostics: vec![ParseDiagnostic {
            stage: DiagnosticStage::Parser,
            message: format!("parser failed: {error:?}"),
            span: tokens.last().map(|token| token.span).unwrap_or_default(),
        }],
    })?;
    let mut nodes = Vec::new();
    let root = lower_syntax(root.as_ref(), &map, &tokens, &mut nodes, limits.max_nodes)?;
    let mut tree = String::new();
    render_tree(root, &nodes, &tokens, &mut tree);
    Ok(SyntaxDocument {
        source: source.to_owned(),
        nodes,
        root,
        tokens,
        tree,
    })
}

fn lower_syntax<'input>(
    node: &(dyn EALParserContext<'input> + 'input),
    map: &SourceMap,
    tokens: &[SyntaxToken],
    nodes: &mut Vec<SyntaxNode>,
    max_nodes: usize,
) -> Result<usize, ParseError> {
    if nodes.len() >= max_nodes {
        return Err(resource_error(
            format!("syntax node limit exceeded ({max_nodes})"),
            tokens.last().map(|token| token.span).unwrap_or_default(),
        ));
    }
    let (kind, span) =
        if let Some(terminal) = node.downcast_ref::<TerminalNode<'input, EALParserContextType>>() {
            let index = terminal.symbol.get_token_index().max(0) as usize;
            (SyntaxKind::Token { index }, tokens[index].span)
        } else {
            let first = node.start();
            let last = node.stop();
            let span = map.span(first.get_start(), last.get_stop());
            (
                SyntaxKind::Rule {
                    name: ealparser::ruleNames[node.get_rule_index()].to_owned(),
                },
                span,
            )
        };
    let id = nodes.len();
    nodes.push(SyntaxNode {
        kind,
        span,
        children: Vec::new(),
    });
    let mut children = Vec::with_capacity(node.get_child_count());
    for child in node.get_children() {
        children.push(lower_syntax(child.as_ref(), map, tokens, nodes, max_nodes)?);
    }
    nodes[id].children = children;
    Ok(id)
}

fn render_tree(id: usize, nodes: &[SyntaxNode], tokens: &[SyntaxToken], output: &mut String) {
    let node = &nodes[id];
    let text = match &node.kind {
        SyntaxKind::Rule { name } => name.as_str(),
        SyntaxKind::Token { index } => tokens[*index].text.as_str(),
    };
    if !node.children.is_empty() {
        output.push('(');
    }
    for character in text.chars() {
        match character {
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ => output.push(character),
        }
    }
    for &child in &node.children {
        output.push(' ');
        render_tree(child, nodes, tokens, output);
    }
    if !node.children.is_empty() {
        output.push(')');
    }
}
