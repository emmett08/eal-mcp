//! Official-SDK MCP operations and the strict one-request text host.
pub mod application;
pub mod auth;
pub use eal_mcp_client::client;
pub mod guard;
pub mod host;
pub mod operations;
pub use eal_mcp_client::persistent_mcp;
pub mod redaction;
pub mod server;
pub mod settings;
pub mod transport;
