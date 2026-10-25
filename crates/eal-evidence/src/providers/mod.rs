//! One graph dispatcher for qualified MCP, command, HTTP and retained sources.
mod config;
mod dispatcher;
mod native;
pub use config::{CommandProtocol, NativePinnedFile, NativeProviderConfig, NativeTransport};
pub use dispatcher::EvidenceSources;
pub use native::NativeSources;
