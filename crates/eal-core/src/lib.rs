//! Pure, typed EAL/3 lexical lowering. Binding and execution are separate stages.
mod ast;
mod format;
mod lower;

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
