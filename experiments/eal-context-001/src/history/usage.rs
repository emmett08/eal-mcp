//! Exact provider-adapted token readings and deadline-restricted latency.
use super::HistoryError;
use num_bigint::BigInt;
use serde_json::Value;
pub fn normalise_usage(
    raw: Option<&Value>,
    output_includes_reasoning: bool,
) -> Result<Value, HistoryError> {
    if raw.is_some_and(|value| !value.is_object()) {
        return Err(HistoryError::Invalid("usage must be an object or null"));
    }
    let names = [
        "input_tokens",
        "output_tokens",
        "cached_input_tokens",
        "reasoning_tokens",
    ];
    let mut numbers = vec![];
    let mut readings = serde_json::Map::new();
    readings.insert("usage_raw".into(), raw.cloned().unwrap_or(Value::Null));
    for name in names {
        let value = raw
            .and_then(|map| map.get(name))
            .filter(|value| !value.is_null());
        let number = value
            .map(|value| {
                if !value.is_number() {
                    return Err(HistoryError::Invalid(
                        "usage readings must be nonnegative integers or null",
                    ));
                }
                let number = value.to_string().parse::<BigInt>().map_err(|_| {
                    HistoryError::Invalid("usage readings must be nonnegative integers or null")
                })?;
                if number < BigInt::from(0) {
                    return Err(HistoryError::Invalid(
                        "usage readings must be nonnegative integers or null",
                    ));
                }
                Ok(number)
            })
            .transpose()?;
        numbers.push(number);
        readings.insert(name.into(), value.cloned().unwrap_or(Value::Null));
    }
    if let (Some(input), Some(cached)) = (&numbers[0], &numbers[2])
        && cached > input
    {
        return Err(HistoryError::Invalid(
            "cached input must be a subset of input",
        ));
    }
    let mut parts = vec![numbers[0].clone(), numbers[1].clone()];
    if !output_includes_reasoning {
        parts.push(numbers[3].clone());
    }
    let total = parts
        .into_iter()
        .try_fold(BigInt::from(0), |sum, part| part.map(|number| sum + number));
    readings.insert(
        "total_tokens".into(),
        total
            .as_ref()
            .map(|number| {
                serde_json::from_str(&number.to_string()).expect("arbitrary JSON integer")
            })
            .unwrap_or(Value::Null),
    );
    readings.insert(
        "usage_missing_reason".into(),
        if total.is_none() {
            Value::from("missing_required_usage")
        } else {
            Value::Null
        },
    );
    Ok(Value::Object(readings))
}

pub fn deadline_restricted_latency(
    observed: Option<f64>,
    completed: bool,
    deadline: f64,
) -> Result<f64, HistoryError> {
    if !deadline.is_finite()
        || deadline <= 0.0
        || observed.is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        return Err(HistoryError::Invalid(
            "latency must be nonnegative and deadline must be positive and finite",
        ));
    }
    Ok(if completed {
        observed
            .map(|value| value.min(deadline))
            .unwrap_or(deadline)
    } else {
        deadline
    })
}
