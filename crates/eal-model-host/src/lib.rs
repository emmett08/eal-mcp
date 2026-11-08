//! Explicit local-model inference and task-bound MCP execution.
//!
//! Operators select the model bundle, executable, registry and workspace. Model
//! proposals select discovered tools and retain exact source, context and IDs.
#![forbid(unsafe_code)]

pub mod bundle;
pub mod contract;
pub mod host;
pub mod model;
pub mod policy;
