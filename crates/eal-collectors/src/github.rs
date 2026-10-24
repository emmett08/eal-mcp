//! An exact read-only GitHub commit observation through an operator-owned gh.
use serde_json::{Value, json};

pub(crate) fn collect(
    config: &crate::Config,
    request: &crate::Request,
    started: std::time::Instant,
) -> Result<Value, String> {
    request.fields(&["repository", "commit"])?;
    let repository = config.get("--repository")?;
    let commit = config.get("--commit")?;
    request.matches("repository", repository)?;
    request.matches("commit", commit)?;
    let output = crate::process::run(
        config,
        vec![
            "api".into(),
            "--hostname".into(),
            "github.com".into(),
            "--method".into(),
            "GET".into(),
            format!("repos/{repository}/commits/{commit}"),
        ],
        started,
    )?;
    let response = crate::process::object(&output)?;
    let sha = response["sha"]
        .as_str()
        .ok_or("GitHub response lacks commit identity")?;
    if !sha.eq_ignore_ascii_case(commit) {
        return Err("GitHub returned a different commit identity".into());
    }
    let tree = response["commit"]["tree"]["sha"]
        .as_str()
        .filter(|v| v.len() == 40 && v.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or("GitHub response lacks tree identity")?;
    Ok(json!({"repository":repository,"commit":sha,"tree":tree,"found":true}))
}
