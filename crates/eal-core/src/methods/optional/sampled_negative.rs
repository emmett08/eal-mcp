//! Separate observed counterexamples from calibrated sampled non-detection findings.
use crate::expressions::numbers;
use serde_json::{Value, json};

pub(super) fn assess(payload: &Value) -> Result<Value, String> {
    let (start, end, max_gap) = (&payload["start"], &payload["end"], &payload["max_gap"]);
    let compare = |a: &Value, b: &Value| {
        numbers::compare(a, b).ok_or_else(|| "Sampled values must be finite numbers".to_owned())
    };
    if compare(end, start)?.is_lt() || !compare(max_gap, &json!(0))?.is_gt() {
        return Err("Sampled interval requires end >= start and max_gap > 0".into());
    }
    let events = payload["events"]
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or("events must be nonempty")?;
    let mut previous: Option<&Value> = None;
    let mut largest_gap = json!(0);
    for event in events {
        let at = &event["time"];
        if compare(at, start)?.is_lt()
            || compare(at, end)?.is_gt()
            || previous.is_some_and(|p| compare(at, p).is_ok_and(|o| !o.is_gt()))
        {
            return Err("Samples must be strictly ordered within the declared interval".into());
        }
        if let Some(previous) = previous {
            let gap = numbers::arithmetic("-", at, previous)?;
            if compare(&gap, &largest_gap)?.is_gt() {
                largest_gap = gap;
            }
        }
        previous = Some(at);
    }
    let coverage = compare(&events[0]["time"], start)?.is_eq()
        && compare(&events.last().expect("samples")["time"], end)?.is_eq()
        && !compare(&largest_gap, max_gap)?.is_gt();
    let bound = &payload["property"]["value"];
    let strict = payload["property"]["operator"] == "lt";
    let violating = events
        .iter()
        .filter(|event| {
            compare(&event["value"], bound)
                .is_ok_and(|o| if strict { !o.is_lt() } else { o.is_gt() })
        })
        .count();
    let mode = payload["mode"].as_str().ok_or("mode must be a string")?;
    let detector_checked = if mode == "counterexample" {
        if violating == 0 {
            return Err(
                "No observed violating sample; partial sampling cannot establish absence".into(),
            );
        }
        false
    } else {
        if violating > 0 {
            return Err("A violating sample prevents a non-detection finding".into());
        }
        if !coverage {
            return Err(
                "Non-detection requires both endpoints and every sampled gap within max_gap".into(),
            );
        }
        let calibration = payload
            .get("calibration")
            .filter(|v| !v.is_null())
            .ok_or("Non-detection requires documented detector calibration")?;
        let contract = &payload["detector_contract"];
        if !compare(&contract["minimum_sensitivity"], &json!(0))?.is_gt() {
            return Err("Non-detection requires a positive minimum sensitivity".into());
        }
        if compare(&contract["maximum_detection_limit"], bound)?.is_gt() {
            return Err(
                "Detector limit could miss values violating the declared upper bound".into(),
            );
        }
        if compare(
            &calibration["detection_limit"],
            &contract["maximum_detection_limit"],
        )?
        .is_gt()
        {
            return Err("Documented detector limit exceeds the declared maximum".into());
        }
        if compare(
            &calibration["sensitivity_lower_bound"],
            &contract["minimum_sensitivity"],
        )?
        .is_lt()
        {
            return Err("Documented detector sensitivity is below the declared minimum".into());
        }
        true
    };
    Ok(
        json!({"finding":true,"mode":mode,"sample_size":events.len(),"violation_count":violating,"coverage":coverage,"largest_gap":largest_gap,"detector_contract_met":detector_checked,"continuous_truth_established":false}),
    )
}
