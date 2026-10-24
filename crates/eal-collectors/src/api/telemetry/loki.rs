//! loki measurement under the shared acquisition contract.
use super::*;
pub(super) async fn collect(
    client: &mut JsonClient,
    query: &str,
    start: u64,
    end: u64,
    limit: u64,
) -> Result<Value, String> {
    let response = get(
        client,
        "/loki/api/v1/query_range",
        &[
            ("query", query.to_owned()),
            ("start", format!("{start}000000000")),
            ("end", format!("{end}000000000")),
            ("limit", limit.to_string()),
            ("direction", "forward".into()),
        ],
    )
    .await?;
    if response["status"] != "success" || response["data"]["resultType"] != "streams" {
        return Err("Loki search requires a successful streams result".into());
    }
    let streams = response["data"]["result"]
        .as_array()
        .ok_or("Loki response lacks explicit streams")?;
    let mut count = 0usize;
    for stream in streams {
        if !stream["stream"].is_object()
            || stream["stream"]
                .as_object()
                .is_some_and(|labels| labels.values().any(|v| !v.is_string()))
        {
            return Err("Loki stream lacks literal labels".into());
        }
        for entry in stream["values"]
            .as_array()
            .ok_or("Loki stream lacks explicit values")?
        {
            if entry.as_array().is_none_or(|pair| {
                pair.len() != 2
                    || pair[0].as_str().is_none_or(|s| s.parse::<u64>().is_err())
                    || !pair[1].is_string()
            }) {
                return Err("Loki stream has malformed entries".into());
            }
        }
        count = count
            .checked_add(
                stream["values"]
                    .as_array()
                    .ok_or("Loki stream lacks explicit values")?
                    .len(),
            )
            .ok_or("Loki result count overflow")?;
    }
    if count > limit as usize {
        return Err("Loki returned more entries than its allowance".into());
    }
    Ok(search(response, count, limit, None))
}
