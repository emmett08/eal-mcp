//! Immutable versioned method contracts and pure built-in reasoning dispatch.
mod contracts;
mod dispatch;
mod logical;
mod numerical;
pub mod optional;
mod probabilistic;
mod schema;
mod structural;
mod validation;

pub use contracts::{
    MethodContract, MethodError, MethodImplementation, MethodRegistry, MethodResultStatus,
    NativeMethodIdentity, default_registry, is_method_identifier,
};
pub use dispatch::{
    assess_mode, assess_mode_with_limits, evidence_kind, validate_mode, validate_mode_with_limits,
};
pub use optional::{engineering_registry, optional_registry};
pub use schema::{schema_errors, schema_errors_with_limits, validate_schema};
