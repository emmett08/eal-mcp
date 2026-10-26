//! Qualified evidence discovery and execution outside the pure argument language.
mod acquisition;
pub mod bridge;
pub mod config;
pub mod discovery;
mod evaluation;
pub mod execution;
mod freshness;
mod identifier_audit;
mod judgement;
mod normalization;
mod numeric;
pub mod plan;
pub mod projection;
pub mod providers;
mod runtime_bridge;
pub mod storage;
pub use bridge::{
    BridgeBinding, BridgeError, BridgeErrorKind, BridgeIdentity, BridgeObservation, CallMetrics,
    EvidenceBridge,
};
pub use config::{
    ApprovedCatalogue, ConnectionConfig, DiscoveryLimits, EvidenceConfig, ObservationTime,
    ResponseFormat, ToolMapping, TransportConfig,
};
pub use discovery::{DiscoveryBudgetUsage, DiscoveryGap, DiscoveryReport, InterfaceCandidate};
