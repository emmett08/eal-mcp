//! Original user-turn deadline and model/tool cycle accounting.
use super::HistoryError;
pub struct TurnBudget {
    pub started_monotonic: f64,
    pub deadline_seconds: f64,
    pub max_cycles: usize,
    pub cycles: usize,
}
impl TurnBudget {
    pub fn new(started_monotonic: f64) -> Self {
        Self {
            started_monotonic,
            deadline_seconds: 120.0,
            max_cycles: 6,
            cycles: 0,
        }
    }
    pub fn before_cycle(&mut self, now_monotonic: f64) -> Result<(), HistoryError> {
        if !now_monotonic.is_finite()
            || !self.started_monotonic.is_finite()
            || !self.deadline_seconds.is_finite()
            || self.deadline_seconds <= 0.0
            || now_monotonic < self.started_monotonic
        {
            return Err(HistoryError::Invalid(
                "monotonic clock and deadline must be finite and ordered",
            ));
        }
        if now_monotonic - self.started_monotonic >= self.deadline_seconds {
            return Err(HistoryError::Timeout);
        }
        if self.cycles >= self.max_cycles {
            return Err(HistoryError::RetrievalExhausted);
        }
        self.cycles += 1;
        Ok(())
    }
}
