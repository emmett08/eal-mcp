//! Source-aware diagnostics shared by lexer, parser and resource guards.
use crate::syntax::{SourcePosition, SourceSpan};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticStage {
    Lexer,
    Parser,
    Resource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseDiagnostic {
    pub stage: DiagnosticStage,
    pub message: String,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseError {
    pub diagnostics: Vec<ParseDiagnostic>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(first) = self.diagnostics.first() {
            write!(
                f,
                "{}:{}: {}",
                first.span.start.line, first.span.start.column, first.message
            )
        } else {
            f.write_str("EAL syntax recognition failed")
        }
    }
}
impl std::error::Error for ParseError {}

pub(crate) struct SourceMap {
    positions: Vec<SourcePosition>,
}

impl SourceMap {
    pub(crate) fn new(source: &str) -> Self {
        let mut position = SourcePosition {
            byte: 0,
            character: 0,
            line: 1,
            column: 1,
        };
        let mut positions = Vec::with_capacity(source.chars().count() + 1);
        for character in source.chars() {
            positions.push(position);
            position.byte += character.len_utf8();
            position.character += 1;
            // ANTLR's line counter, including the Python reference, advances on LF.
            if character == '\n' {
                position.line += 1;
                position.column = 1;
            } else {
                position.column += 1;
            }
        }
        positions.push(position);
        Self { positions }
    }

    pub(crate) fn position(&self, index: usize) -> SourcePosition {
        self.positions[index.min(self.positions.len() - 1)]
    }

    pub(crate) fn span(&self, start: isize, stop: isize) -> SourceSpan {
        let start = start.max(0) as usize;
        let end = if stop < start as isize {
            start
        } else {
            stop as usize + 1
        };
        SourceSpan {
            start: self.position(start),
            end: self.position(end),
        }
    }

    pub(crate) fn at_line_column(&self, line: isize, column: isize) -> SourceSpan {
        let position = self
            .positions
            .iter()
            .find(|position| {
                position.line == line.max(1) as usize
                    && position.column == column.max(0) as usize + 1
            })
            .copied()
            .unwrap_or_else(|| *self.positions.last().unwrap());
        SourceSpan {
            start: position,
            end: position,
        }
    }
}
