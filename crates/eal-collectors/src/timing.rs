//! Conservative acquisition age with wall-clock discontinuities kept inspectable.
use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Value, json};

/// Wall time supplies age metadata only. Process deadlines always use Instant.
pub trait ObservationClock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}
pub struct SystemClock;
impl ObservationClock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub(crate) struct Timeline<'a> {
    clock: &'a dyn ObservationClock,
    first: DateTime<Utc>,
    earliest: DateTime<Utc>,
    previous: DateTime<Utc>,
    regressed: bool,
}
impl<'a> Timeline<'a> {
    pub fn new(clock: &'a dyn ObservationClock) -> Self {
        let now = clock.now();
        Self {
            clock,
            first: now,
            earliest: now,
            previous: now,
            regressed: false,
        }
    }
    pub fn mark(&mut self) -> DateTime<Utc> {
        let now = self.clock.now();
        self.regressed |= now < self.previous;
        self.earliest = self.earliest.min(now);
        self.previous = now;
        now
    }
    pub fn interval(&mut self, start: DateTime<Utc>) -> Value {
        let end = self.mark();
        json!({"started_at":stamp(start),"completed_at":stamp(end),"observed_at":stamp(start.min(end)),"wall_clock_regressed":end<start})
    }
    pub fn observed_at(&self) -> String {
        stamp(self.earliest)
    }
    pub fn finish(&mut self) -> Value {
        let end = self.mark();
        json!({"started_at":stamp(self.first),"completed_at":stamp(end),"oldest_observation_bound":stamp(self.earliest),"wall_clock_regressed":self.regressed,"snapshot_semantics":"sequential_acquisition"})
    }
}
fn stamp(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Nanos, true)
}
