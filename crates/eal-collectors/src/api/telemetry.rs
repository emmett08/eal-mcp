//! Window admission and selection of provider-owned telemetry measurements.
mod datadog_metrics;
mod datadog_monitor;
mod datadog_search;
mod datadog_slo;
mod loki;
mod prometheus;
use super::{ApiSettings, ApiSource};
use eal_adapters::http::JsonClient;
use reqwest::Method;
use serde_json::{Value, json};
pub(super) async fn collect(
    settings: &ApiSettings,
    input: &Value,
    client: &mut JsonClient,
    timeline: &mut crate::timing::Timeline<'_>,
) -> Result<Value, String> {
    if let ApiSource::DatadogMonitor { id } = &settings.source {
        return datadog_monitor::collect(client, *id).await;
    }
    let start = input["start_unix"]
        .as_u64()
        .ok_or("Query requires integer start_unix")?;
    let end = input["end_unix"]
        .as_u64()
        .ok_or("Query requires integer end_unix")?;
    let now =
        u64::try_from(timeline.mark().timestamp()).map_err(|_| "Wall clock precedes Unix epoch")?;
    if start == 0
        || end <= start
        || end - start > settings.max_window_seconds
        || end > now.saturating_add(60)
    {
        return Err("Query window exceeds the operator allowance".into());
    }
    let mut value = match &settings.source {
        ApiSource::Prometheus {
            query,
            step_seconds,
        } => {
            prometheus::collect(
                client,
                query,
                start,
                end,
                *step_seconds,
                settings.max_points,
            )
            .await
        }
        ApiSource::Loki { query, limit } => loki::collect(client, query, start, end, *limit).await,
        ApiSource::DatadogMetrics {
            query,
            interval_seconds,
        } => {
            datadog_metrics::collect(
                client,
                query,
                start,
                end,
                *interval_seconds,
                settings.max_points,
            )
            .await
        }
        ApiSource::DatadogLogs { query, limit } => {
            datadog_search::collect(client, query, start, end, *limit, false).await
        }
        ApiSource::DatadogSpans { query, limit } => {
            datadog_search::collect(client, query, start, end, *limit, true).await
        }
        ApiSource::DatadogSlo { id } => datadog_slo::collect(client, id, start, end).await,
        _ => Err("Unsupported telemetry measurement".into()),
    }?;
    value["start_unix"] = json!(start);
    value["end_unix"] = json!(end);
    if let Some(query) = settings.source.query() {
        value["query"] = json!(query);
    }
    Ok(value)
}
fn points(start: u64, end: u64, step: u64, max: u64) -> Result<(), String> {
    if (end - start) / step + 1 > max {
        Err("Metric query exceeds its point allowance".into())
    } else {
        Ok(())
    }
}
fn stamp(time: u64) -> Result<String, String> {
    chrono::DateTime::from_timestamp(i64::try_from(time).map_err(|_| "Invalid query time")?, 0)
        .map(|t| t.to_rfc3339())
        .ok_or("Invalid query time".into())
}
fn search(response: Value, count: usize, limit: u64, cursor: Option<Value>) -> Value {
    json!({"collected":true,"matched":count>0,"returned_count":count,"result_limit_reached":count as u64==limit,"search_exhaustive":false,"negative_inference":"unavailable","next_cursor":cursor,"observation":response})
}
fn errors(response: &Value) -> Result<(), String> {
    if !response.is_object()
        || response.get("error").is_some()
        || response
            .get("errors")
            .is_some_and(|v| v.as_array().is_none_or(|a| !a.is_empty()))
    {
        Err("Telemetry response reports an error or invalid object".into())
    } else {
        Ok(())
    }
}
async fn get(
    client: &mut JsonClient,
    path: &str,
    query: &[(&str, String)],
) -> Result<Value, String> {
    let mut url = client.url(path).map_err(|e| e.to_string())?;
    url.query_pairs_mut()
        .extend_pairs(query.iter().map(|(k, v)| (*k, v.as_str())));
    let response = client
        .request(Method::GET, url, None)
        .await
        .map_err(|e| e.to_string())?;
    errors(&response)?;
    Ok(response)
}
