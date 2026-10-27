//! EAL/4 recognition generated from the pinned `grammar/EAL.g4`.
//!
//! This layer rejects every lexer/parser error and owns its concrete syntax.
//! Name resolution, field validation and reasoning are separate core passes.
mod adapter;
mod diagnostics;
mod generated;
mod limits;
mod syntax;

pub use adapter::{parse, parse_with_limits};
pub use diagnostics::{DiagnosticStage, ParseDiagnostic, ParseError};
pub use limits::ParseLimits;
pub use syntax::{
    NodeId, SourcePosition, SourceSpan, SyntaxDocument, SyntaxKind, SyntaxNode, SyntaxToken,
};
