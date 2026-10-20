//! Durable delivery and effect reservations for registered-argument coordination.
#![forbid(unsafe_code)]

pub mod bus;
pub mod cli;
pub mod contracts;
pub mod coordinator;
pub mod gateway;
pub mod host;
pub mod lab_mcp;
pub mod lab_server;
pub mod lab_store;
pub mod ledger;
pub mod persistent_mcp;
pub mod profiles;
pub mod protocol;
pub mod state;
pub mod workers;

pub const RESPONSE_SCHEMA: &str =
    include_str!("../../../contracts/schemas/coordination-worker-response.schema.json");
