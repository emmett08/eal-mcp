//! Exercise independently authored requests through installed text and MCP entry points.
use super::{
    Result, ensure,
    harness::{Harness, SOURCE, TOKEN},
};
use serde_json::{Value, json};

pub(super) fn check(
    h: &Harness,
    runtime: &tokio::runtime::Runtime,
    client: &reqwest::Client,
    url: &str,
    restricted: bool,
    assessment: &Value,
) -> Result<()> {
    let mut cases = if restricted {
        vec![
            ("sources", json!({}), None),
            (
                "find_claims",
                json!({"query":"supplied", "claim":"works"}),
                None,
            ),
            (
                "assess_known",
                json!({"entry_id":"demo", "claim":"works"}),
                Some("entry_id"),
            ),
            (
                "get_known_packet",
                json!({"entry_id":"demo", "assessment_id":assessment["assessment_id"], "claim":"works"}),
                Some("assessment_id"),
            ),
        ]
    } else {
        vec![
            ("describe", json!({}), None),
            ("format", json!({"source":SOURCE}), Some("source")),
            ("validate", json!({"source":SOURCE}), Some("source")),
            (
                "plan",
                json!({"source":SOURCE, "claim":"works"}),
                Some("claim"),
            ),
            (
                "collect",
                json!({"source":SOURCE, "context":{"site":"bench"}}),
                Some("context"),
            ),
            (
                "collect_claim",
                json!({"source":SOURCE, "context":{"site":"bench"}, "claim":"works"}),
                Some("claim"),
            ),
            (
                "prepare",
                json!({"source":SOURCE, "context":{"site":"bench"}, "claim":"works"}),
                Some("context"),
            ),
            (
                "execute",
                json!({"source":SOURCE, "context":{"site":"bench"}, "claim":"works", "now":h.now}),
                Some("context"),
            ),
            (
                "reason",
                json!({"source":SOURCE, "context":{"site":"bench"}, "collection_id":assessment["collection_id"], "now":h.now}),
                Some("context"),
            ),
            (
                "compile_aspic",
                json!({"source":SOURCE, "context":{"site":"bench"}, "collection_id":assessment["collection_id"], "goal":"works", "now":h.now}),
                Some("goal"),
            ),
            (
                "explain",
                json!({"assessment_id":assessment["assessment_id"]}),
                Some("assessment_id"),
            ),
            (
                "packet",
                json!({"assessment_id":assessment["assessment_id"]}),
                Some("assessment_id"),
            ),
            (
                "grounded",
                json!({"arguments":["a","b"], "attacks":[["a","b"]]}),
                Some("arguments"),
            ),
        ]
    };
    // Deterministic order keeps these checks independent of the advertised inventory.
    cases.sort_by_key(|(name, _, _)| *name);
    let host_args = ["--url", url, "--token-env", super::harness::TOKEN_ENV];
    for (name, arguments, required) in cases {
        let mut request = arguments.clone();
        request["operation"] = json!(name);
        let answer = h.host(&request, &host_args, 0)?;
        ensure(
            answer["is_error"] == false,
            format!("Text host rejected valid {name}"),
        )?;
        let answer = runtime.block_on(call(client, url, name, &arguments))?;
        ensure(
            answer["isError"] != true && answer["structuredContent"].is_object(),
            format!("MCP rejected valid {name}: {answer}"),
        )?;

        let mut invalids = vec![{
            let mut value = arguments.clone();
            value["unknown_field"] = json!(true);
            value
        }];
        if let Some(required) = required {
            let mut missing = arguments.clone();
            missing.as_object_mut().unwrap().remove(required);
            invalids.push(missing);
            for wrong in [Value::Null, json!(42)] {
                let mut invalid = arguments.clone();
                invalid[required] = wrong;
                invalids.push(invalid);
            }
            if ["claim", "entry_id", "assessment_id", "goal"].contains(&required) {
                let mut empty = arguments.clone();
                empty[required] = json!("");
                invalids.push(empty);
            }
        } else if ["sources", "find_claims"].contains(&name) {
            for limit in [Value::Null, json!("50"), json!(0)] {
                let mut invalid = arguments.clone();
                invalid["limit"] = limit;
                invalids.push(invalid);
            }
        }
        for mut invalid in invalids {
            let raw = runtime.block_on(call(client, url, name, &invalid))?;
            ensure(
                raw["isError"] == true,
                format!("MCP admitted malformed {name}"),
            )?;
            invalid["operation"] = json!(name);
            let text = h.host(&invalid, &host_args, 2)?;
            ensure(
                text["is_error"] == true,
                format!("Text host admitted malformed {name}"),
            )?;
        }
        // A common transport-boundary probe per operation complements the
        // full per-field corpus in eal-mcp's package tests.
        let mut oversized = arguments.clone();
        oversized["oversized"] = json!("x".repeat(4 * 1024 * 1024 + 1));
        runtime.block_on(reject_oversized(client, url, name, &oversized))?;
        oversized["operation"] = json!(name);
        ensure(
            h.host(&oversized, &host_args, 2)?["is_error"] == true,
            format!("Text host admitted oversized {name}"),
        )?;
    }
    Ok(())
}
async fn call(
    client: &reqwest::Client,
    url: &str,
    operation: &str,
    arguments: &Value,
) -> Result<Value> {
    let request = json!({"jsonrpc":"2.0", "id":31, "method":"tools/call", "params":{"name":format!("eal_{operation}"), "arguments":arguments}});
    let (status, body) =
        super::applications::post(client, url, &request, Some(TOKEN), None, None).await?;
    let value = super::parse_json(&body, 4 * 1024 * 1024)?;
    if status == 200 {
        value
            .get("result")
            .cloned()
            .ok_or_else(|| format!("MCP returned a protocol error for {operation}: {value}"))
    } else if status == 413 {
        Ok(json!({"isError":true}))
    } else {
        Err(format!(
            "MCP operation {operation} returned HTTP {status}: {value}"
        ))
    }
}

async fn reject_oversized(
    client: &reqwest::Client,
    url: &str,
    operation: &str,
    arguments: &Value,
) -> Result<()> {
    let request = json!({"jsonrpc":"2.0", "id":32, "method":"tools/call", "params":{"name":format!("eal_{operation}"), "arguments":arguments}});
    let (status, body) =
        super::applications::post(client, url, &request, Some(TOKEN), None, None).await?;
    if [400, 413, 415].contains(&status) {
        return Ok(());
    }
    let value = super::parse_json(&body, 4 * 1024 * 1024)?;
    ensure(
        status == 200 && value["result"]["isError"] == true,
        format!("MCP admitted oversized {operation}"),
    )
}
