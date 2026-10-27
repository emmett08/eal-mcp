use crate::profile::ToolchainProfile;

pub fn bounded(text: &str, maximum_bytes: usize) -> String {
    let mut end = text.len().min(maximum_bytes);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}

pub fn feedback_redactions(profile: &ToolchainProfile) -> Vec<String> {
    let mut values: Vec<_> = std::env::vars_os()
        .filter_map(|(name, value)| {
            let name = name.to_string_lossy();
            let upper = name.to_ascii_uppercase();
            (profile
                .inherit_env
                .iter()
                .any(|selected| selected == name.as_ref())
                || [
                    "SECRET",
                    "TOKEN",
                    "PASSWORD",
                    "CREDENTIAL",
                    "AUTH",
                    "API_KEY",
                ]
                .iter()
                .any(|marker| upper.contains(marker))
                || upper.ends_with("_KEY"))
            .then(|| value.to_string_lossy().into_owned())
            .filter(|value| !value.is_empty())
        })
        .collect();
    values.sort_by_key(|value| std::cmp::Reverse(value.len()));
    values.dedup();
    values
}

/// Redact before bounding so a token crossing the truncation boundary cannot
/// leave its visible prefix in a subsequent model request.
pub fn redact_feedback(text: &str, sensitive: &[String]) -> String {
    sensitive
        .iter()
        .filter(|value| !value.is_empty())
        .fold(text.to_owned(), |text, value| {
            let escaped = serde_json::to_string(value).expect("string JSON");
            text.replace(value, "[redacted]")
                .replace(&escaped[1..escaped.len() - 1], "[redacted]")
        })
}
