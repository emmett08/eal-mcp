//! One bounded isolated execution lifecycle for authoring and deployed EAL tools.
#![forbid(unsafe_code)]
pub mod bounded_file;
pub mod contracts;
mod daemon_snapshot;
pub mod execution_diagnostics;
pub mod observation;
pub mod package;
pub mod profile;
pub mod runner;
pub use contracts::SourceFile;
pub use profile::ToolchainProfile;

#[derive(Debug)]
pub enum Error {
    InvalidInput(String),
    InvalidOutput(String),
    ToolExecution(String),
    Budget(String),
    Io(std::io::Error),
    Json(serde_json::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(s)
            | Self::InvalidOutput(s)
            | Self::ToolExecution(s)
            | Self::Budget(s) => f.write_str(s),
            Self::Io(e) => e.fmt(f),
            Self::Json(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
fn validate_language(language: &str) -> Result<(), Error> {
    if language.is_empty()
        || language.len() > 64
        || !language.as_bytes()[0].is_ascii_alphanumeric()
        || !language
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_+.-".contains(&b))
    {
        return Err(Error::InvalidInput(
            "tool language must be a bounded ASCII identifier".into(),
        ));
    }
    Ok(())
}
