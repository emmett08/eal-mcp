//! Closed input schemas and shared admission, before operation effects.
pub(crate) use eal_mcp_client::schema::bounded_json;
pub use eal_mcp_client::schema::{compile_schema, validate_schema};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;

/// Rejected direct Rust callers can supply deeper values than a wire decoder.
/// Tear those down without invoking Value's recursive container destructor.
pub(crate) fn discard(value: Value) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Array(values) => pending.extend(values),
            Value::Object(values) => pending.extend(values.into_values()),
            _ => {}
        }
    }
}

#[derive(Clone)]
pub(crate) struct Admission(Arc<Semaphore>);
impl Admission {
    pub fn new(max_in_flight: usize) -> Result<Self, String> {
        if !(1..=1024).contains(&max_in_flight) {
            return Err("max_in_flight must be an integer from 1 through 1024".into());
        }
        Ok(Self(Arc::new(Semaphore::new(max_in_flight))))
    }
    pub async fn acquire(
        &self,
        cancellation: &CancellationToken,
    ) -> Result<OwnedSemaphorePermit, String> {
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => Err("Tool request cancelled before admission".into()),
            permit = Arc::clone(&self.0).acquire_owned() => permit.map_err(|_| "Tool admission closed".into()),
        }
    }
}
