//! Optional native and MCP candidate discovery for authoring.
mod config;
mod dispatcher;
mod native;
pub use config::{CommandProtocol, NativePinnedFile, NativeProviderConfig, NativeTransport};
pub use dispatcher::EvidenceSources;
pub use native::NativeSources;
