//! Finite ASPIC reasoning and explicit, observation-bound EAL projections.
pub mod compiler;
mod error;
pub mod export;
pub mod registry;
pub mod solver;

pub use compiler::{CompileOptions, CompiledTheory, compile_eal_aspic};
pub use error::AspicError;
pub use export::export_aspic_view;
pub use registry::{aspic_contract, aspic_registry, with_aspic};
pub use solver::{solve_aspic, validate_theory};

pub const METHOD: &str = "argumentation/aspic/2";
pub const PROFILE: &str = "EAL/3-compiled-aspic/5";
