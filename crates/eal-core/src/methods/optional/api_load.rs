//! Evaluate inclusive finite API sample criteria without authenticating the report.
use crate::expressions::numbers;
use serde_json::{Value, json};

pub(super) fn assess(report: &Value) -> Result<Value, String> {
    let count = &report["request_count"];
    if count.is_null() {
        return Err("An admitted report requires a measured request count".into());
    }
    let count = count
        .as_u64()
        .ok_or_else(|| "A measured request count must be a nonnegative integer".to_owned())?;
    let (latency, errors) = if count > 0 {
        if report["p95_ms"].is_null() || report["error_rate_percent"].is_null() {
            return Err("A nonempty report requires both measured statistics".into());
        }
        (
            Some(numbers::compare(&report["p95_ms"], &json!(200)).is_some_and(|o| !o.is_gt())),
            Some(
                numbers::compare(&report["error_rate_percent"], &json!(1))
                    .is_some_and(|o| !o.is_gt()),
            ),
        )
    } else {
        if !report["p95_ms"].is_null() || !report["error_rate_percent"].is_null() {
            return Err("An empty report cannot carry latency or error-rate statistics".into());
        }
        (None, None)
    };
    let sample = count >= 100;
    Ok(
        json!({"sample_size_met":sample,"latency_met":latency,"error_limit_met":errors,"passes":sample&&latency==Some(true)&&errors==Some(true),"fails":!sample||latency==Some(false)||errors==Some(false)}),
    )
}
