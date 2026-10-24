//! prometheus measurement under the shared acquisition contract.
use super::*;
pub(super) async fn collect(
    client: &mut JsonClient,
    query: &str,
    start: u64,
    end: u64,
    step_seconds: u64,
    max_points: u64,
) -> Result<Value, String> {
    points(start, end, step_seconds, max_points)?;
    let response = get(
        client,
        "/api/v1/query_range",
        &[
            ("query", query.to_owned()),
            ("start", start.to_string()),
            ("end", end.to_string()),
            ("step", step_seconds.to_string()),
        ],
    )
    .await?;
    if response["status"] != "success" || response["data"]["resultType"] != "matrix" {
        return Err("Prometheus response lacks a successful matrix result".into());
    }
    let result = response["data"]["result"]
        .as_array()
        .ok_or("Prometheus response lacks explicit results")?;
    let count = result.len();
    for series in result {
        let samples = series["values"]
            .as_array()
            .ok_or("Prometheus float series lacks explicit values")?;
        if !series["metric"].is_object()
            || samples.len() as u64 > max_points
            || samples.iter().any(|sample| {
                sample.as_array().is_none_or(|pair| {
                    pair.len() != 2
                        || !pair[0].is_number()
                        || pair[1]
                            .as_str()
                            .is_none_or(|text| text.parse::<f64>().is_err())
                })
            })
        {
            return Err("Prometheus float series has malformed samples".into());
        }
    }
    Ok::<Value, String>(
        json!({"collected":true,"returned_series":count,"empty_result":count==0,"requested_step_seconds":step_seconds,"observation":response}),
    )
}
