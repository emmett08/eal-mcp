//! Exercise installed CLI, durable assessment and actual stdio and HTTP sessions.
use super::{
    Options, Result, ensure,
    harness::{Harness, SOURCE, TOKEN, TOKEN_ENV},
    process::Background,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};
const OPERATOR: &[&str] = &[
    "eal_describe",
    "eal_format",
    "eal_validate",
    "eal_plan",
    "eal_collect",
    "eal_collect_claim",
    "eal_reason",
    "eal_compile_aspic",
    "eal_explain",
    "eal_packet",
    "eal_grounded",
];
const REGISTERED: &[&str] = &[
    "eal_sources",
    "eal_find_claims",
    "eal_assess_known",
    "eal_get_known_packet",
];
pub fn run(options: &Options) -> Result<Value> {
    let h = Harness::new(
        options,
        "all",
        &[
            "eal",
            "eal-mcp",
            "eal-host",
            "eal-command-supervisor",
            "eal-method-worker",
        ],
    )?;
    let mut checks = Vec::new();
    ensure(
        h.cli(&["validate", "case.eal"])?["valid"] == true,
        "CLI validation failed",
    )?;
    ensure(
        h.cli(&["plan", "case.eal", "--claim", "works"])?["evidence_ids"] == json!(["reading"]),
        "Incorrect evidence plan",
    )?;
    let entry = h.cli(&[
        "register",
        "case.eal",
        "--entry-id",
        "demo",
        "--claim",
        "works",
        "--context",
        "@context.json",
    ])?;
    ensure(
        entry["entry_id"] == "demo"
            && entry["claims"] == json!(["works"])
            && entry.get("source").is_none(),
        "Invalid source registration",
    )?;
    ensure(
        h.cli(&["sources"])?["entries"][0]["entry_id"] == "demo",
        "Registered source absent",
    )?;
    ensure(
        h.cli(&["find", "supplied", "--claim", "works"])?["matches"][0]["source_digest"]
            == entry["source_digest"],
        "Find returned a different revision",
    )?;
    checks.push("production CLI validate/plan/register/sources/find");
    let first = h.cli(&[
        "assess-known",
        "demo",
        "--claim",
        "works",
        "--now",
        &h.now,
        "--reuse",
        "fresh",
    ])?;
    ensure(
        first["status"] == "supported"
            && first["collected_count"] == 1
            && first["reused_count"] == 0,
        "Fresh registered acquisition failed",
    )?;
    let reused = h.cli(&["assess-known", "demo", "--claim", "works", "--now", &h.now])?;
    ensure(
        reused["status"] == "supported"
            && reused["collected_count"] == 0
            && reused["reused_count"] == 1,
        "Compatible acquisition reuse failed",
    )?;
    let id = reused["assessment_id"]
        .as_str()
        .ok_or("Missing assessment ID")?;
    let packet = h.cli(&["packet", id, "--claim", "works"])?;
    ensure(
        packet["schema"] == "EAL/assessment-packet/2"
            && packet["assessment_id"] == reused["assessment_id"]
            && packet["collection_id"] == reused["collection_id"],
        "Packet correspondence failed",
    )?;
    ensure(
        !packet.to_string().contains("collector-secret"),
        "Packet exposed raw observations",
    )?;
    ensure(
        h.cli(&["explain", id, "--claim", "works"])?["result"]["status"] == "supported",
        "Explanation mismatch",
    )?;
    checks.push(
        "durable registered assessment, compatible observation reuse, explanation and packet",
    );
    let prepared = h.cli(&[
        "model-context",
        "demo",
        "--claim",
        "works",
        "--question",
        "Did the supplied test pass?",
        "--now",
        &h.now,
    ])?;
    ensure(
        prepared["schema"] == "EAL/model-context/2"
            && prepared["assessment"]["reused_count"] == 1
            && prepared["assessment"]["collected_count"] == 0,
        "Model context reacquired evidence",
    )?;
    ensure(
        prepared["messages"][1] == json!({"role":"user","content":"Did the supplied test pass?"})
            && !prepared["messages"]
                .to_string()
                .contains("collector-secret"),
        "Model context scope or disclosure mismatch",
    )?;
    let history = h.cli(&[
        "history",
        "demo",
        "--source-digest",
        entry["source_digest"]
            .as_str()
            .ok_or("Missing source digest")?,
    ])?;
    ensure(
        history["assessments"][0]["assessment_id"] == prepared["assessment"]["assessment_id"]
            && history["assessments"][0]["claim_statuses"]["works"] == "supported",
        "Revision-bound history mismatch",
    )?;
    checks.push("model context reuses evidence, excludes raw tool values, and appears in revision-bound history");
    let answer = h.host(
        &json!({"operation":"grounded","arguments":["a","b","c"],"attacks":[["a","b"],["b","c"]]}),
        &[],
        0,
    )?;
    ensure(
        answer["protocol_version"] == "2025-11-25"
            && answer["is_error"] == false
            && answer["result"]["accepted"] == json!(["a", "c"])
            && answer["result"]["rejected"] == json!(["b"]),
        "Native stdio grounded exchange failed",
    )?;
    let answer = h.host(
        &json!({"operation":"assess_known","entry_id":"demo","claim":"works"}),
        &["--known-entry", "demo"],
        0,
    )?;
    ensure(
        answer["result"]["status"] == "supported"
            && answer["result"]["reused_count"] == 1
            && answer["result"]["collected_count"] == 0,
        "Restricted stdio reuse failed",
    )?;
    checks.push(
        "actual eal-host stdio full and restricted SDK sessions with one JSON stdout response",
    );
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    for restricted in [false, true] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        drop(listener);
        let url = format!("http://127.0.0.1:{port}/mcp");
        let mut args = h.common();
        args.extend([
            "--transport".into(),
            "http".into(),
            "--host".into(),
            "127.0.0.1".into(),
            "--port".into(),
            port.to_string(),
            "--token-env".into(),
            TOKEN_ENV.into(),
        ]);
        if restricted {
            args.extend(["--known-entry".into(), "demo".into()]);
        }
        let mut server =
            Background::start(&h.prefix.join("eal-mcp"), &args, &h.root, &h.environment)?;
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;
        let initialize = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"installed-native-smoke","version":"1"}}});
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            ensure(server.alive()?, "HTTP server exited during startup")?;
            if runtime
                .block_on(post(&client, &url, &initialize, None, None, None))
                .is_ok_and(|(status, _)| status == 401)
            {
                break;
            }
            ensure(
                Instant::now() < deadline,
                "HTTP server startup deadline exceeded",
            )?;
            std::thread::sleep(Duration::from_millis(25));
        }
        let (status, body) =
            runtime.block_on(post(&client, &url, &initialize, Some(TOKEN), None, None))?;
        let value = super::parse_json(&body, 4 * 1024 * 1024)?;
        ensure(
            status == 200 && value["result"]["protocolVersion"] == "2025-11-25",
            "HTTP initialization mismatch",
        )?;
        let (status, body) = runtime.block_on(post(
            &client,
            &url,
            &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
            Some(TOKEN),
            None,
            None,
        ))?;
        let value = super::parse_json(&body, 4 * 1024 * 1024)?;
        let actual = value["result"]["tools"]
            .as_array()
            .ok_or("Missing HTTP tool inventory")?
            .iter()
            .map(|v| v["name"].as_str().unwrap_or(""))
            .collect::<BTreeSet<_>>();
        let expected = if restricted { REGISTERED } else { OPERATOR }
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        ensure(
            status == 200 && actual == expected,
            "HTTP tool inventory mismatch",
        )?;
        for (bearer, origin, host, status) in [
            (None, None, None, 401),
            (Some("wrong-token"), None, None, 401),
            (Some(TOKEN), Some("https://foreign.example"), None, 403),
            (Some(TOKEN), None, Some("foreign.example"), 421),
        ] {
            ensure(
                runtime
                    .block_on(post(&client, &url, &initialize, bearer, origin, host))?
                    .0
                    == status,
                "HTTP authentication/origin/host policy failed",
            )?;
        }
        let host_args = ["--url", url.as_str(), "--token-env", TOKEN_ENV];
        if restricted {
            let answer = h.host(&json!({"operation":"sources"}), &host_args, 0)?;
            ensure(
                answer["is_error"] == false && answer["result"]["sources"][0]["entry_id"] == "demo",
                "Restricted HTTP discovery failed",
            )?;
            let answer = h.host(
                &json!({"operation":"assess_known","entry_id":"demo","claim":"works"}),
                &host_args,
                0,
            )?;
            let assessment = &answer["result"];
            ensure(
                assessment["status"] == "supported"
                    && assessment["reused_count"] == 1
                    && assessment["collected_count"] == 0,
                "Restricted HTTP assessment failed",
            )?;
            let answer=h.host(&json!({"operation":"get_known_packet","entry_id":"demo","claim":"works","assessment_id":assessment["assessment_id"],"packet_limits":{"max_bytes":4096,"max_statement_bytes":3}}),&host_args,0)?;
            let packet = &answer["result"]["packet"];
            ensure(
                answer["is_error"] == false
                    && answer["result"]["assessed_at"] == assessment["assessed_at"]
                    && packet["claims"]["works"]["statement"] == "The"
                    && packet["summary_complete"] == false
                    && serde_json::to_vec(packet).map_err(|e| e.to_string())?.len() <= 4096,
                "Bounded stored-packet HTTP retrieval failed",
            )?;
            let rejected = h.host(
                &json!({"operation":"validate","source":SOURCE}),
                &host_args,
                2,
            )?;
            ensure(
                rejected["error"]
                    .as_str()
                    .is_some_and(|s| s.contains("does not offer eal_validate")),
                "Restricted HTTP exposed operator validation",
            )?;
            checks.push("authenticated restricted HTTP inventory, reusable assessment and bounded stored-packet retrieval; operator tool unavailable");
        } else {
            let source = SOURCE.replace("The supplied test passed at this bench.", TOKEN);
            let answer = h.host(
                &json!({"operation":"format","source":source}),
                &host_args,
                0,
            )?;
            ensure(
                answer["is_error"] == false
                    && answer["protocol_version"] == "2025-11-25"
                    && !answer.to_string().contains(TOKEN)
                    && answer["result"]["source"]
                        .as_str()
                        .is_some_and(|s| s.contains("[redacted]")),
                "HTTP bearer redaction failed",
            )?;
            let answer=h.host(&json!({"operation":"reason","source":SOURCE,"context":{"site":"bench"},"collection_id":reused["collection_id"],"now":h.now}),&host_args,0)?;
            ensure(
                answer["result"]["claims"]["works"]["status"] == "supported",
                "Full HTTP reasoning failed",
            )?;
            checks.push("authenticated full HTTP inventory and reasoning; host redacts bearer from structured and encoded content");
        }
        checks.push(if restricted {
            "restricted HTTP rejects missing/wrong bearer, foreign Origin, foreign Host"
        } else {
            "full HTTP rejects missing/wrong bearer, foreign Origin, foreign Host"
        });
        server.finish()?;
    }
    checks
        .push("all installed native children ran with PATH empty; HTTP servers emitted no stdout");
    ensure(
        checks.len() == 9,
        "Application acceptance group count changed",
    )?;
    h.finish(json!({"check_groups":checks.len(),"checks":checks}))
}
async fn post(
    client: &reqwest::Client,
    url: &str,
    payload: &Value,
    bearer: Option<&str>,
    origin: Option<&str>,
    host: Option<&str>,
) -> Result<(u16, Vec<u8>)> {
    let mut request = client
        .post(url)
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2025-11-25")
        .json(payload);
    if let Some(v) = bearer {
        request = request.bearer_auth(v);
    }
    if let Some(v) = origin {
        request = request.header("Origin", v);
    }
    if let Some(v) = host {
        request = request.header("Host", v);
    }
    let mut response = request.send().await.map_err(|e| e.to_string())?;
    let status = response.status().as_u16();
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        ensure(
            body.len() + chunk.len() <= 4 * 1024 * 1024,
            "HTTP smoke response exceeds 4 MiB",
        )?;
        body.extend_from_slice(&chunk);
    }
    Ok((status, body))
}
