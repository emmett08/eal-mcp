//! Per-capability execution admission shared by all clones of one bridge.
//! A rolling minute bounds acquisition attempts; source failures never refund
//! admission or trigger retries. Independent processes retain separate budgets.
use super::*;
use std::{collections::VecDeque, sync::Condvar};

pub(super) struct CapabilityGate {
    concurrency: usize,
    requests_per_minute: Option<u64>,
    state: Mutex<GateState>,
    changed: Condvar,
}
#[derive(Default)]
struct GateState {
    active: usize,
    requests: VecDeque<Instant>,
}
pub(super) struct Permit {
    gate: Arc<CapabilityGate>,
}
impl CapabilityGate {
    fn new(descriptor: &CapabilityDescriptor) -> Self {
        Self {
            concurrency: descriptor.limits.concurrency as usize,
            requests_per_minute: descriptor.limits.requests_per_minute,
            state: Mutex::new(GateState::default()),
            changed: Condvar::new(),
        }
    }
    fn acquire(self: Arc<Self>, deadline: Instant) -> Result<Permit, BridgeError> {
        let mut state = lock(&self.state)?;
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(error(
                    BridgeErrorKind::Budget,
                    "MCP capability admission deadline exhausted",
                ));
            }
            while state
                .requests
                .front()
                .is_some_and(|at| now.saturating_duration_since(*at) >= Duration::from_secs(60))
            {
                state.requests.pop_front();
            }
            let rate_ready = if self
                .requests_per_minute
                .is_some_and(|maximum| state.requests.len() as u64 >= maximum)
            {
                let ready = state
                    .requests
                    .front()
                    .ok_or_else(|| {
                        error(
                            BridgeErrorKind::Protocol,
                            "MCP capability rate state is unavailable",
                        )
                    })?
                    .checked_add(Duration::from_secs(60))
                    .ok_or_else(|| {
                        error(
                            BridgeErrorKind::Budget,
                            "MCP capability rate interval exceeds clock resources",
                        )
                    })?;
                if ready >= deadline {
                    return Err(error(
                        BridgeErrorKind::Budget,
                        "MCP capability request rate would exceed the execution deadline",
                    ));
                }
                Some(ready)
            } else {
                None
            };
            if state.active < self.concurrency && rate_ready.is_none() {
                state.active += 1;
                if self.requests_per_minute.is_some() {
                    state.requests.push_back(now);
                }
                drop(state);
                return Ok(Permit { gate: self });
            }
            let wake = rate_ready.unwrap_or(deadline).min(deadline);
            let timeout = wake
                .saturating_duration_since(now)
                .min(Duration::from_secs(1));
            state = self
                .changed
                .wait_timeout(state, timeout)
                .map_err(|_| {
                    error(
                        BridgeErrorKind::Protocol,
                        "MCP capability admission state is unavailable",
                    )
                })?
                .0;
        }
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        if let Ok(mut state) = self.gate.state.lock() {
            state.active = state.active.saturating_sub(1);
            self.gate.changed.notify_all();
        }
    }
}
impl EvidenceBridge {
    pub(super) fn admit_execution(
        &self,
        descriptor: &CapabilityDescriptor,
        deadline: Instant,
    ) -> Result<Permit, BridgeError> {
        let gate = {
            let mut gates = lock(&self.inner.gates)?;
            gates
                .entry(descriptor.id.clone())
                .or_insert_with(|| Arc::new(CapabilityGate::new(descriptor)))
                .clone()
        };
        gate.acquire(deadline)
    }
}
