//! datadog search measurement under the shared acquisition contract.
use super::*;
pub(super) async fn collect(
    client: &mut JsonClient,
    query: &str,
    start: u64,
    end: u64,
    limit: u64,
    spans: bool,
) -> Result<Value, String> {
    let filter = json!({"query":query,"from":stamp(start)?,"to":stamp(end)?});
    let attributes = json!({"filter":filter,"sort":"timestamp","page":{"limit":limit}});
    let body = if spans {
        json!({"data":{"type":"search_request","attributes":attributes}})
    } else {
        attributes
    };
    let path = if spans {
        "/api/v2/spans/events/search"
    } else {
        "/api/v2/logs/events/search"
    };
    let url = client.url(path).map_err(|e| e.to_string())?;
    let response = client
        .request(Method::POST, url, Some(&body))
        .await
        .map_err(|e| e.to_string())?;
    errors(&response)?;
    let records = response["data"]
        .as_array()
        .ok_or("Datadog search lacks explicit results")?;
    let count = records.len();
    if records
        .iter()
        .any(|record| record["id"].as_str().is_none_or(|id| id.is_empty()))
    {
        return Err("Datadog search result lacks record identity".into());
    }
    if count > limit as usize {
        return Err("Datadog search exceeds its result allowance".into());
    }
    let cursor = response.pointer("/meta/page/after").cloned();
    Ok(search(response, count, limit, cursor))
}
