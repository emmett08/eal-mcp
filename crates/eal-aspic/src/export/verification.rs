//! Retain recorded grounding reports even when their argument was not constructed.
use serde_json::{Value, json};

pub(super) fn reports(source: Option<&Value>) -> Vec<Value> {
    source
        .and_then(|value| value["arguments"].as_object())
        .into_iter()
        .flat_map(|arguments| arguments.iter())
        .filter_map(|(name, argument)| {
            let computation = &argument["computation"];
            if computation["method"] != "grounded/1" {
                return None;
            }
            Some(json!({
                "argument_name": name,
                "source": "native_method",
                "method_status": computation["status"],
                "reasons": computation["reasons"],
                "report": computation["details"].get("verification")
            }))
        })
        .collect()
}
