//! Typed EAL/3 syntax, composition, validation and authored reasoning with explicit inputs.
mod ast;
pub mod canonical;
pub mod composition;
pub mod dialectic;
pub mod evaluation;
pub mod expressions;
mod format;
pub mod json;
mod limits;
mod lower;
pub mod methods;
pub mod planning;
pub mod propositions;
pub mod scope_transfer;
pub mod semantic;
pub mod semantic_format;
mod semantic_spans;
pub mod validation;

pub use limits::ExecutionLimits;

pub use ast::*;
pub use eal_parser::ParseLimits;
pub use lower::{CoreError, lower};

/// Recognise and structurally check one source without resolving imports or names.
pub fn parse(source: &str) -> Result<Program, CoreError> {
    lower(&eal_parser::parse(source).map_err(CoreError::Recognition)?)
}

/// Apply explicit recognition resource limits before typed structural lowering.
pub fn parse_with_limits(source: &str, limits: ParseLimits) -> Result<Program, CoreError> {
    lower(&eal_parser::parse_with_limits(source, limits).map_err(CoreError::Recognition)?)
}
