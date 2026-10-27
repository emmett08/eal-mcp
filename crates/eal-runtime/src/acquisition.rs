//! Operator-selected collectors, private binding identities and strict observations.
mod adapters;
mod binding;
mod command;
mod error;
mod file;
mod files;
mod http;
mod json;
mod mcp;
mod normalization;
mod registry;
mod request_encoding;

pub use adapters::{acquire, acquire_with_limits};
pub use binding::{PinnedFile, ToolBinding, ToolKind, keyed_digest, keyed_digest_with_limits};
pub use error::{AcquisitionError, AcquisitionFailureCategory};
pub use files::{bounded_path, read_regular};
pub use json::{
    strict_json, strict_json_with_limits, validate_envelope, validate_envelope_with_limits,
};
pub use registry::ToolRegistry;
pub use request_encoding::encode_request;

pub const MAX_JSON_DEPTH: usize = 128;
pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_PINNED_FILE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub struct AcquisitionResult {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub metadata: serde_json::Value,
    pub error: Option<AcquisitionError>,
}
impl Default for AcquisitionResult {
    fn default() -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
            metadata: serde_json::json!({}),
            error: None,
        }
    }
}
