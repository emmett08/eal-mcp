//! Owned concrete syntax, independent of the generated ANTLR representation.
use serde::{Deserialize, Serialize};

/// Position in UTF-8 source. Lines and Unicode scalar columns are one based.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePosition {
    pub byte: usize,
    pub character: usize,
    pub line: usize,
    pub column: usize,
}

impl Default for SourcePosition {
    fn default() -> Self {
        Self {
            byte: 0,
            character: 0,
            line: 1,
            column: 1,
        }
    }
}

/// Half-open source range. Empty ranges represent EOF and empty grammar rules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

/// A lexer token, including comments and EOF; skipped spaces are retained in source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxToken {
    pub token_type: i32,
    pub kind: String,
    pub text: String,
    pub channel: i32,
    pub span: SourceSpan,
}

/// Arena index. Children retain source order.
pub type NodeId = usize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyntaxKind {
    Rule { name: String },
    Token { index: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub span: SourceSpan,
    pub children: Vec<NodeId>,
}

impl SyntaxNode {
    pub fn rule_name(&self) -> Option<&str> {
        match &self.kind {
            SyntaxKind::Rule { name } => Some(name),
            SyntaxKind::Token { .. } => None,
        }
    }
}

/// Strictly recognised EAL syntax. Semantic validation belongs to `eal-core`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxDocument {
    pub source: String,
    pub nodes: Vec<SyntaxNode>,
    pub root: NodeId,
    pub tokens: Vec<SyntaxToken>,
    /// ANTLR's rule-name tree, useful for reference-parser comparisons.
    pub tree: String,
}

impl SyntaxDocument {
    pub fn node(&self, id: NodeId) -> &SyntaxNode {
        &self.nodes[id]
    }

    pub fn token(&self, id: NodeId) -> Option<&SyntaxToken> {
        match self.nodes[id].kind {
            SyntaxKind::Token { index } => self.tokens.get(index),
            SyntaxKind::Rule { .. } => None,
        }
    }

    pub fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes[id].children.iter().copied()
    }

    pub fn children_named<'a>(
        &'a self,
        id: NodeId,
        name: &'a str,
    ) -> impl Iterator<Item = NodeId> + 'a {
        self.children(id)
            .filter(move |&child| self.nodes[child].rule_name() == Some(name))
    }

    pub fn source_text(&self, id: NodeId) -> &str {
        let span = self.nodes[id].span;
        &self.source[span.start.byte..span.end.byte]
    }
}
