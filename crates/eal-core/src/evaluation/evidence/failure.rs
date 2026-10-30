//! Bound and escape retained execution errors without changing observation admission.
use crate::{ExecutionLimits, expressions};
use serde_json::{Value, json};

pub(super) fn reason(record: &Value, limits: &ExecutionLimits) -> String {
    let Some(message) = record["error"]["message"]
        .as_str()
        .filter(|message| !message.is_empty())
    else {
        return "Recorded execution failure has no usable retained error diagnostic".into();
    };
    let error_type = record["error"]["type"].as_str().unwrap_or("Error");
    // Copy only bounded explanatory text. Arbitrary error details neither enter
    // assessment output nor supply observation or provenance fields.
    // Insert keys in lexical order so diagnostic text is identical with either
    // serde_json's ordered-map or sorted-map configuration in a consumer build.
    let detail = json!({
        "message":bounded_text(message, 512),
        "type":bounded_text(error_type, 64),
    });
    match expressions::diagnostic_repr(&detail, limits) {
        Ok(detail) => format!("Recorded execution failure: {detail}"),
        Err(_) => "Recorded execution failure diagnostic exceeds the selected JSON limits".into(),
    }
}

fn bounded_text(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{} [truncated]", &value[..end])
}
