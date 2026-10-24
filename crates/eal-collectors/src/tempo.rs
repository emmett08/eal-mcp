//! Bounded per-cluster existential TraceQL searches with explicit time scope.
use crate::config::number;
use serde_json::{Value, json};
use std::time::Instant;

pub(crate) async fn collect(
    config: &crate::Config,
    request: &crate::Request,
    started: Instant,
) -> Result<Value, String> {
    request.fields(&["start_unix", "end_unix"])?;
    let start = request.input["start_unix"]
        .as_u64()
        .ok_or("Tempo start_unix requires a positive integer")?;
    let end = request.input["end_unix"]
        .as_u64()
        .ok_or("Tempo end_unix requires a positive integer")?;
    let max_window = number(
        config.options.get("--max-window-seconds"),
        86400,
        1,
        2_592_000,
    )?;
    let now = chrono::Utc::now().timestamp() as u64;
    if start == 0 || end <= start || end - start > max_window || end > now.saturating_add(60) {
        return Err("Tempo window is invalid or exceeds the operator allowance".into());
    }
    let limit = number(config.options.get("--limit"), 20, 1, 1000)?;
    let (inventory, digest) = crate::inventory::read(config)?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .timeout(config.timeout)
        .build()
        .map_err(|_| "Cannot construct Tempo HTTP client")?;
    let token = credential(config, "--token-env")?;
    let tenant = credential(config, "--tenant-env")?;
    let mut clusters = Vec::new();
    let mut all_matched = true;
    let mut spent = 0usize;
    for cluster in &inventory.clusters {
        let q = query(config, &cluster.tempo_cluster)?;
        let remaining = config
            .timeout
            .checked_sub(started.elapsed())
            .filter(|v| !v.is_zero())
            .ok_or("Collector deadline exhausted")?;
        let mut url = url::Url::parse(config.get("--url")?).map_err(|_| "Invalid Tempo URL")?;
        url.query_pairs_mut().extend_pairs([
            ("q", q.clone()),
            ("start", start.to_string()),
            ("end", end.to_string()),
            ("limit", limit.to_string()),
        ]);
        let mut call = client
            .get(url)
            .header("Accept", "application/json")
            .timeout(remaining);
        if let Some(token) = &token {
            call = call.bearer_auth(token);
        }
        if let Some(tenant) = &tenant {
            call = call.header("X-Scope-OrgID", tenant);
        }
        let mut response = call
            .send()
            .await
            .map_err(|_| "Tempo request failed or reached its deadline")?;
        if !response.status().is_success() {
            return Err(format!(
                "Tempo returned HTTP {}",
                response.status().as_u16()
            ));
        }
        if response
            .content_length()
            .is_some_and(|n| n > config.max_bytes as u64)
        {
            return Err("Tempo response exceeds its byte allowance".into());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| "Tempo response transfer failed")?
        {
            if chunk.len() > config.max_bytes.saturating_sub(bytes.len()) {
                return Err("Tempo response exceeds its byte allowance".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        spent = spent.saturating_add(bytes.len());
        if spent > config.max_bytes {
            return Err("Tempo responses exceed the aggregate byte allowance".into());
        }
        let value = crate::process::object(&bytes)?;
        if [token.as_deref(), tenant.as_deref()]
            .into_iter()
            .flatten()
            .any(|secret| contains_credential(&value, secret))
        {
            return Err("Tempo response contains configured credential material".into());
        }
        if value.get("error").is_some() || value.get("errors").is_some() {
            return Err("Tempo response reports a search error".into());
        }
        let traces = value["traces"]
            .as_array()
            .ok_or("Tempo response lacks an explicit traces array")?;
        if traces.len() > limit as usize {
            return Err("Tempo returned more traces than the requested allowance".into());
        }
        for trace in traces {
            if trace["traceID"].as_str().is_none_or(|s| {
                ![16, 32].contains(&s.len())
                    || !s.bytes().all(|b| b.is_ascii_hexdigit())
                    || s.bytes().all(|b| b == b'0')
            }) {
                return Err("Tempo response has an invalid trace identity".into());
            }
        }
        let matched = !traces.is_empty();
        all_matched &= matched;
        let partial = match (
            value["metrics"]["totalJobs"].as_u64(),
            value["metrics"]["completedJobs"].as_u64(),
        ) {
            (Some(total), Some(completed)) if completed <= total => Some(completed < total),
            (Some(_), Some(_)) => return Err("Tempo returned inconsistent job counts".into()),
            _ => None,
        };
        clusters.push(json!({"cluster":cluster.id,"tempo_cluster":cluster.tempo_cluster,"query":q,"matched":matched,"result_limit_reached":traces.len()==limit as usize,"reported_partial":partial,"search_exhaustive":false,"traces":traces,"metrics":value.get("metrics")}));
    }
    Ok(
        json!({"inventory_revision":inventory.revision,"inventory_sha256":digest,"cluster_count":inventory.clusters.len(),"start_unix":start,"end_unix":end,"limit":limit,"all_matched":all_matched,"search_exhaustive":false,"negative_inference":"unavailable","clusters":clusters}),
    )
}
fn credential(config: &crate::Config, key: &str) -> Result<Option<String>, String> {
    match config.options.get(key) {
        None => Ok(None),
        Some(name) => {
            let value =
                std::env::var(name).map_err(|_| "Configured Tempo credential is unavailable")?;
            if value.is_empty() || value.len() > 8192 || value.chars().any(char::is_control) {
                return Err("Configured Tempo credential is malformed".into());
            }
            Ok(Some(value))
        }
    }
}
pub(crate) fn query(config: &crate::Config, cluster: &str) -> Result<String, String> {
    let quoted = |value: &str| {
        serde_json::to_string(value).map_err(|_| "Cannot encode TraceQL literal".to_string())
    };
    Ok(format!(
        "{{ {} = {} && {} = {} && {} = {} }}",
        config.optional("--kind-attribute", "span.dpe.rollout_kind"),
        quoted(config.get("--kind")?)?,
        config.optional("--name-attribute", "span.dpe.rollout_name"),
        quoted(config.get("--name")?)?,
        config.optional("--cluster-attribute", "resource.k8s.cluster.name"),
        quoted(cluster)?
    ))
}

fn contains_credential(value: &Value, secret: &str) -> bool {
    match value {
        Value::String(text) => text.contains(secret),
        Value::Array(items) => items.iter().any(|item| contains_credential(item, secret)),
        Value::Object(fields) => fields
            .iter()
            .any(|(key, value)| key.contains(secret) || contains_credential(value, secret)),
        _ => false,
    }
}
