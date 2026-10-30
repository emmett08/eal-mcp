//! Bind native method observation metadata to the retained evidence container.
use crate::validation::parse_time;
use serde_json::{Value, json};

pub(crate) fn context(payload: &Value, record: Option<&Value>, assessed_at: &str) -> Value {
    let mut payload = payload.clone();
    if !payload.is_object() {
        return payload;
    }
    payload["assessed_at"] = json!(assessed_at);
    let Some(observations) = payload
        .get_mut("observations")
        .and_then(Value::as_object_mut)
    else {
        return payload;
    };
    let actual_time = record.and_then(|record| record["collected_at"].as_str());
    let actual_digest = record
        .and_then(|record| record["source_digest"].as_str())
        .filter(|digest| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let available = record.is_some_and(|record| record["status"] == "ok");
    for observation in observations
        .values_mut()
        .filter(|observation| observation.is_object())
    {
        observation["source_digest"] = json!(actual_digest);
        if !available {
            observation["status"] = json!("unavailable");
            observation["observed_at"] = Value::Null;
            continue;
        }
        let supplied = observation["observed_at"].as_str();
        // Both original clocks must exist. The container can only make an
        // observation older, never supply a missing original measurement time.
        let bounded = match actual_time.zip(supplied).and_then(|(actual, supplied)| {
            Some((
                actual,
                supplied,
                parse_time(actual).ok()?,
                parse_time(supplied).ok()?,
            ))
        }) {
            Some((actual, supplied, actual_instant, supplied_instant)) => {
                // Keep an explicitly future-dated source clock visible so the
                // applicability check rejects it rather than repairing it.
                let future = parse_time(assessed_at).is_ok_and(|now| supplied_instant > now);
                Some(
                    if future || supplied_instant <= actual_instant {
                        supplied
                    } else {
                        actual
                    }
                    .to_owned(),
                )
            }
            None => None,
        };
        observation["observed_at"] = json!(bounded);
    }
    payload
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binding_never_creates_original_times_and_malformed_values_do_not_panic() {
        assert_eq!(
            context(&Value::Null, None, "2026-10-30T12:00:00Z"),
            Value::Null
        );
        let record = json!({"status":"ok","collected_at":"2026-10-30T12:00:00Z","source_digest":"a".repeat(64)});
        let input = json!({"observations":{"source":{"status":"available","value":{"voltage":5},"observed_at":null,"source_digest":"forged"}}});
        let bound = context(&input, Some(&record), "2026-10-30T12:00:01Z");
        assert!(bound["observations"]["source"]["observed_at"].is_null());
        assert_eq!(
            bound["observations"]["source"]["source_digest"],
            "a".repeat(64)
        );
    }
    #[test]
    fn future_clock_remains_visible_to_applicability_instead_of_becoming_fresh() {
        let record = json!({"status":"ok","collected_at":"2026-10-30T12:00:00Z","source_digest":"a".repeat(64)});
        let input = json!({"observations":{"source":{"status":"available","value":{},"observed_at":"2026-10-30T12:01:00Z"}}});
        let bound = context(&input, Some(&record), "2026-10-30T12:00:01Z");
        assert_eq!(
            bound["observations"]["source"]["observed_at"],
            "2026-10-30T12:01:00Z"
        );
    }
}
