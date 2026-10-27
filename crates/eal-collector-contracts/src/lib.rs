//! Pure wire models for describing and configuring operator-selected collectors.
mod binding;
mod direct_binding;
pub use direct_binding::{DirectBinding, McpServerIdentity, ResponseFormat};
pub mod capability;
mod catalogue;
mod command;
mod identifiers;
mod json;
mod opaque_json;
mod profile;
mod request;
mod retained_judgement;
mod schema;
pub mod scoped_review;
mod sensitive_json;
mod toml_diagnostic;

pub use binding::{
    BindingSpec, OPERATOR_ENVIRONMENT_POLICY, OperatorEnvironmentPolicy, PinnedFile,
    RUNTIME_EXPORTABLE_PROFILE, RegistryBinding, RegistryDocument, ToolKind,
    validate_environment_names,
};
pub use catalogue::{CATALOGUE_SCHEMA, Catalogue, CollectorDescriptor, InputConstraint};
pub use command::{CommandBinding, CommandRegistry, CommandTemplate, PreparedCommandRegistry};
pub use identifiers::{CollectorId, EnvironmentName, ToolId};
pub use json::{
    JsonDiagnostic, JsonDiagnosticCode, JsonLimits, JsonNumericProfile, strict_json,
    strict_json_diagnostic, strict_json_exact, strict_json_exact_diagnostic,
};
pub use opaque_json::{
    deserialize_optional_value_exact, deserialize_value_exact, deserialize_value_map_exact,
};
pub use profile::{AuthoringProfile, checked_predicate};
pub use request::{ACQUISITION_CONTRACT, AcquisitionRequest, ObservationEnvelope};
pub use sensitive_json::{SensitiveJsonError, SensitiveJsonLimits, contains_sensitive_json};
pub use toml_diagnostic::{TomlDocumentKind, toml_diagnostic};

pub fn identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    value.len() <= 1024
        && bytes
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}

pub fn environment_name(value: &str) -> bool {
    identifier(value) && !value.contains(['.', '-'])
}

/// Independent registry admission cases shared with downstream compatibility CI.
pub const REGISTRY_CORPUS: &str = include_str!("../registry-corpus.json");

/// Independent catalogue/schema/profile cases, including semantic-only restrictions.
pub const CATALOGUE_CORPUS: &str = include_str!("../catalogue-corpus.json");

pub mod execution;
