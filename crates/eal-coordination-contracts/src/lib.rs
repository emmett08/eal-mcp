//! Pure configuration, message and transition contracts for directed debate.
//!
//! The coordinator routes opaque application payloads. Applications validate
//! their contents and perform provider calls outside this crate.

mod frame_budget;
mod payload;
mod protocol;
mod session;
pub mod topology;

pub use protocol::{
    CallContext, ClientMessage, Dispatch, PeerValue, RoundResult, ServerMessage, SessionSpec,
};
pub use session::Session;
pub use topology::{Adaptation, DebateConfig, Edge, Graph, Topology};

/// Exact wire version admitted by this coordination implementation.
pub const SCHEMA_VERSION: &str = "eal.debate/1";
/// Maximum encoded JSON bytes in one opaque participant response.
pub const MAX_PAYLOAD_BYTES: usize = 1_048_576;
/// Maximum nesting depth below an opaque payload's root value.
pub const MAX_PAYLOAD_DEPTH: usize = 64;
/// Maximum values in one opaque payload, further bounded by its byte allowance.
pub const MAX_PAYLOAD_NODES: usize = 1_000_000;
/// Maximum bytes of one client or server JSON line, including its newline.
pub const MAX_FRAME_BYTES: usize = 33_554_432;

/// Configuration or protocol failure without runtime or transport dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidInput(String),
    Protocol(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid input: {message}"),
            Self::Protocol(message) => write!(formatter, "invalid debate transition: {message}"),
        }
    }
}

impl std::error::Error for Error {}
