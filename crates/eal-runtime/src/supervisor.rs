//! Compatibility entry point for the legacy installed companion.
//! Remove the legacy binary alias in EAL 0.2 after deployment migration.
pub use eal_process::supervisor::entrypoint;
