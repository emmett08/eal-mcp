//! Preserve a selected Datadog monitor state without inferring service health.
use super::*;
pub(super) async fn collect(client: &mut JsonClient, id: u64) -> Result<Value, String> {
    let response = get(client, &format!("/api/v1/monitor/{id}"), &[]).await?;
    if response["id"].as_u64() != Some(id) || response["overall_state"].as_str().is_none() {
        return Err("Datadog monitor response lacks matching identity or state".into());
    }
    Ok(json!({"collected":true,"id":id,"observation":response}))
}
