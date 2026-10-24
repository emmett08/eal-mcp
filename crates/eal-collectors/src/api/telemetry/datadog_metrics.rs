//! datadog metrics measurement under the shared acquisition contract.
use super::*;
pub(super) async fn collect(
    client: &mut JsonClient,
    query: &str,
    start: u64,
    end: u64,
    interval_seconds: u64,
    max_points: u64,
) -> Result<Value, String> {
    points(start, end, interval_seconds, max_points)?;
    let body = json!({"data":{"type":"timeseries_request","attributes":{"from":start*1000,"to":end*1000,"interval":interval_seconds*1000,
                "queries":[{"data_source":"metrics","name":"a","query":query}],"formulas":[{"formula":"a"}]}}});
    let url = client
        .url("/api/v2/query/timeseries")
        .map_err(|e| e.to_string())?;
    let response = client
        .request(Method::POST, url, Some(&body))
        .await
        .map_err(|e| e.to_string())?;
    errors(&response)?;
    let attributes = &response["data"]["attributes"];
    let series = attributes["series"]
        .as_array()
        .ok_or("Datadog metrics response lacks explicit series")?;
    let values = attributes["values"]
        .as_array()
        .ok_or("Datadog metrics response lacks explicit values")?;
    let times = attributes["times"]
        .as_array()
        .ok_or("Datadog metrics response lacks explicit times")?;
    if response["data"]["type"] != "timeseries_response"
        || series.len() != values.len()
        || times.len() as u64 > max_points
        || times.iter().any(|time| time.as_u64().is_none())
        || values.iter().any(|row| {
            row.as_array().is_none_or(|samples| {
                samples.len() != times.len()
                    || samples
                        .iter()
                        .any(|sample| !sample.is_null() && !sample.is_number())
            })
        })
    {
        return Err("Datadog metrics response has inconsistent series or times".into());
    }
    Ok(
        json!({"collected":true,"returned_series":series.len(),"empty_result":series.is_empty(),"requested_interval_seconds":interval_seconds,"observation":response}),
    )
}
