//! Pure wire models for describing and configuring operator-selected collectors.
mod catalogue;
mod command;
mod request;

pub use catalogue::{CATALOGUE_SCHEMA, Catalogue, CollectorDescriptor, InputConstraint};
pub use command::{CommandBinding, CommandRegistry, CommandTemplate};
pub use request::{ACQUISITION_CONTRACT, AcquisitionRequest, ObservationEnvelope};

pub(crate) fn identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    value.len() <= 1024
        && bytes
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}

pub(crate) fn environment_name(value: &str) -> bool {
    identifier(value) && !value.contains(['.', '-'])
}
