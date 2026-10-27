//! Optional evidence discovery and bounded authoring probes. EAL runtime owns execution.
pub mod bridge;
pub mod config;
pub mod discovery;
mod normalization;
pub mod providers;
pub use bridge::{
    BridgeBinding, BridgeError, BridgeErrorKind, BridgeIdentity, BridgeObservation, CallMetrics,
    EvidenceBridge,
};
pub use config::{
    ApprovedCatalogue, ConnectionConfig, DiscoveryLimits, EvidenceConfig, ObservationTime,
    ResponseFormat, ToolMapping, TransportConfig,
};
pub use discovery::{DiscoveryBudgetUsage, DiscoveryGap, DiscoveryReport, InterfaceCandidate};
