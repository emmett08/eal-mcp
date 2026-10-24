//! datadog slo measurement under the shared acquisition contract.
use super::*;
pub(super) async fn collect(
    client: &mut JsonClient,
    id: &str,
    start: u64,
    end: u64,
) -> Result<Value, String> {
    let response = get(
        client,
        &format!("/api/v1/slo/{id}/history"),
        &[("from_ts", start.to_string()), ("to_ts", end.to_string())],
    )
    .await?;
    if !response["data"].is_object() {
        return Err("Datadog SLO history lacks explicit data".into());
    }
    Ok(json!({"collected":true,"id":id,"observation":response}))
}
