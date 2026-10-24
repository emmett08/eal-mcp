//! Exact-revision CI observations with bounded pagination and retained attempt identity.
use crate::config::{GithubChecksConfig, GithubWorkflowConfig};
use serde_json::{Value, json};
use std::{collections::BTreeSet, time::Instant};

pub(crate) fn checks(
    config: &crate::Config,
    selected: &GithubChecksConfig,
    request: &crate::Request,
    started: Instant,
) -> Result<Value, String> {
    request.fields(&["repository", "commit"])?;
    let source = &selected.github;
    request.matches("repository", &source.repository)?;
    request.matches("commit", &source.commit)?;
    let mut runs = Vec::new();
    let mut ids = BTreeSet::new();
    let mut total = None;
    let mut spent = 0usize;
    let mut pages = 0;
    for page in 1..=selected.max_pages {
        let output = read(
            config,
            format!(
                "repos/{}/commits/{}/check-runs?per_page=100&page={page}&filter=all",
                source.repository, source.commit
            ),
            started,
        )?;
        spent = spent.saturating_add(output.len());
        if spent > config.limits().max_bytes() {
            return Err("GitHub checks exceed aggregate byte allowance".into());
        }
        let response = crate::process::object(&output)?;
        let reported = response["total_count"]
            .as_u64()
            .ok_or("GitHub checks lack total_count")?;
        if total.is_some_and(|old| old != reported) {
            return Err("GitHub check pagination changed its reported total".into());
        }
        total = Some(reported);
        let batch = response["check_runs"]
            .as_array()
            .filter(|a| a.len() <= 100)
            .ok_or("GitHub checks lack a bounded check_runs array")?;
        for run in batch {
            revision(run, &source.commit)?;
            let id = run["id"].as_u64().ok_or("GitHub check lacks identity")?;
            if !ids.insert(id) || run["name"].as_str().is_none() {
                return Err("GitHub check identities are missing or repeated".into());
            }
            runs.push(run.clone());
        }
        pages = page;
        if runs.len() as u64 > reported {
            return Err("GitHub checks exceed their reported total".into());
        }
        if batch.len() < 100 || runs.len() as u64 == reported {
            break;
        }
    }
    Ok(
        json!({"collected":true,"measurement_only":true,"repository":source.repository,"commit":source.commit,
        "reported_total":total,"returned_count":runs.len(),"pages":pages,"reported_pages_complete":Some(runs.len() as u64)==total,
        "search_exhaustive":false,"negative_inference":"unavailable","checks":runs}),
    )
}
pub(crate) fn workflow(
    config: &crate::Config,
    selected: &GithubWorkflowConfig,
    request: &crate::Request,
    started: Instant,
) -> Result<Value, String> {
    request.fields(&["repository", "commit", "run_id", "attempt"])?;
    let source = &selected.github;
    request.matches("repository", &source.repository)?;
    request.matches("commit", &source.commit)?;
    if request.input["run_id"].as_u64() != Some(selected.run_id)
        || request.input["attempt"].as_u64() != Some(selected.attempt)
    {
        return Err("GitHub workflow input differs from operator run or attempt".into());
    }
    let output = read(
        config,
        format!(
            "repos/{}/actions/runs/{}/attempts/{}",
            source.repository, selected.run_id, selected.attempt
        ),
        started,
    )?;
    let run = crate::process::object(&output)?;
    revision(&run, &source.commit)?;
    if run["id"].as_u64() != Some(selected.run_id)
        || run["run_attempt"].as_u64() != Some(selected.attempt)
        || run["repository"]["full_name"].as_str() != Some(source.repository.as_str())
    {
        return Err("GitHub workflow returned a different run, attempt or repository".into());
    }
    Ok(
        json!({"collected":true,"measurement_only":true,"repository":source.repository,"commit":source.commit,"run_id":selected.run_id,"attempt":selected.attempt,"workflow":run}),
    )
}
fn revision(run: &Value, commit: &str) -> Result<(), String> {
    if run["head_sha"]
        .as_str()
        .is_none_or(|sha| !sha.eq_ignore_ascii_case(commit))
        || run["status"].as_str().is_none()
        || run
            .get("conclusion")
            .is_none_or(|v| !v.is_null() && !v.is_string())
    {
        return Err("GitHub CI response lacks matching revision or explicit result state".into());
    }
    Ok(())
}
fn read(config: &crate::Config, path: String, started: Instant) -> Result<Vec<u8>, String> {
    crate::process::run(
        config,
        vec![
            "api".into(),
            "--hostname".into(),
            "github.com".into(),
            "--method".into(),
            "GET".into(),
            path,
        ],
        started,
    )
}
