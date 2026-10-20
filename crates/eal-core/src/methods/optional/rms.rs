//! RMS deviation from an explicitly supplied dimensional origin, with scaled norms.
use crate::expressions::numbers;
use num_rational::BigRational;
use num_traits::Zero;
use serde_json::{Value, json};

pub(super) fn assess(payload: &Value) -> Result<Value, String> {
    let samples = payload["samples"]
        .as_array()
        .ok_or("samples must be a list")?;
    let deviations: Vec<_> = samples
        .iter()
        .map(|v| {
            numbers::arithmetic("-", v, &payload["origin"])?
                .as_f64()
                .filter(|v| v.is_finite())
                .ok_or_else(|| "RMS deviation exceeds finite binary64 range".to_owned())
        })
        .collect::<Result<_, _>>()?;
    let squares =
        deviations
            .iter()
            .try_fold(BigRational::zero(), |sum, value| -> Result<_, String> {
                let n = BigRational::from_float(*value).ok_or("RMS deviations must be finite")?;
                Ok(sum + &n * &n)
            })?;
    let norm = super::super::numerical::sqrt_fraction(&squares)?;
    let rms = norm / (samples.len() as f64).sqrt();
    if !rms.is_finite() {
        return Err("RMS result exceeds finite binary64 range".into());
    }
    Ok(json!({"rms":rms,"sample_size":samples.len()}))
}
