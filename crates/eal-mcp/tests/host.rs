//! Native one-request client configuration, admission and response boundaries.
use eal_mcp::{
    auth::{BearerTokenVerifier, HttpPolicy},
    client::{self, ClientTarget},
    host, redaction,
    settings::{self, ServerSettings},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "eal-host-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| (*s).to_owned()).collect()
}
#[test]
fn configuration_precedence_retains_path_origins_and_replaces_lists() {
    let directory = Directory::new();
    let config_dir = directory.0.join("config");
    std::fs::create_dir(&config_dir).unwrap();
    let config = config_dir.join("server.toml");
    std::fs::write(&config,"[service]\nworkspace='file-workspace'\nregistry='tools.toml'\nknown_entries=['file']\n[server]\nmax_in_flight=2\n[server.http]\nport=1234\n").unwrap();
    let env = BTreeMap::from([
        ("EAL_MCP_CONFIG".into(), config.to_str().unwrap().into()),
        ("EAL_MCP_WORKSPACE".into(), "environment-workspace".into()),
        ("EAL_MCP_KNOWN_ENTRIES".into(), "[\"environment\"]".into()),
        ("EAL_MCP_PORT".into(), "invalid overridden value".into()),
    ]);
    let settings = settings::parse_server_settings(
        &strings(&[
            "--port",
            "32123",
            "--known-entry",
            "one",
            "--known-entry",
            "two",
        ]),
        &env,
        &directory.0,
    )
    .unwrap();
    assert_eq!(
        settings.workspace,
        directory.0.join("environment-workspace")
    );
    assert_eq!(settings.registry, Some(config_dir.join("tools.toml")));
    assert_eq!(settings.known_entries, ["one", "two"]);
    assert_eq!(settings.exposure, "registered");
    assert_eq!(settings.max_in_flight, 2);
    assert_eq!(settings.port, 32123);
    let inherited = settings::parse_server_settings(
        &strings(&["--config", config.to_str().unwrap()]),
        &BTreeMap::new(),
        &directory.0,
    )
    .unwrap();
    assert_eq!(inherited.workspace, config_dir.join("file-workspace"));
    assert!(!settings.workspace.exists());
}
#[test]
fn configuration_rejects_unknown_types_ambiguity_and_remote_without_auth() {
    let directory = Directory::new();
    for args in [
        vec!["--max-in-flight", "0"],
        vec!["--port", "65536"],
        vec!["--known-entry", "one", "--known-entry", "one"],
        vec!["--known-entry", "one", "--exposure", "operator"],
        vec!["--exposure", "registered"],
        vec!["--path", "/mcp/{tenant}"],
        vec!["--path", "/mcp/../other"],
        vec!["--host", "http://localhost"],
        vec!["--transport", "http", "--host", "0.0.0.0"],
        vec![
            "--transport",
            "http",
            "--allowed-origin",
            "http://2130706433",
        ],
    ] {
        assert!(
            settings::parse_server_settings(&strings(&args), &BTreeMap::new(), &directory.0)
                .is_err(),
            "{args:?}"
        );
    }
    let config = directory.0.join("server.toml");
    for source in [
        "[unknown]\nvalue=1",
        "[server]\nmax_in_flight=true",
        "[server]\n[server.http]\nunknown=1",
        "[service]\nknown_entries='one'",
        "[service]\nknown_entries=[1979-05-27]",
    ] {
        std::fs::write(&config, source).unwrap();
        assert!(
            settings::parse_server_settings(
                &strings(&["--config", config.to_str().unwrap()]),
                &BTreeMap::new(),
                &directory.0
            )
            .is_err()
        );
    }
}
#[test]
fn credentials_are_opaque_and_http_admission_matches_source_defaults() {
    let token = "synthetic-token.ABC_123~";
    let settings = ServerSettings {
        transport: "http".into(),
        host: "0.0.0.0".into(),
        token_env: Some("OPERATOR_TOKEN".into()),
        allowed_hosts: vec!["engineering.example".into()],
        allowed_origins: vec!["https://trusted.example".into()],
        ..Default::default()
    };
    assert!(
        HttpPolicy::from_settings(&settings, &BTreeMap::new())
            .unwrap_err()
            .contains("OPERATOR_TOKEN")
    );
    let env = BTreeMap::from([("OPERATOR_TOKEN".into(), token.into())]);
    let policy = HttpPolicy::from_settings(&settings, &env).unwrap();
    assert!(!format!("{policy:?}").contains(token));
    let verifier = BearerTokenVerifier::new(token).unwrap();
    assert!(verifier.verify(token));
    assert!(!verifier.verify("wrong"));
    assert!(!verifier.verify(""));
    let auth = format!("Bearer {token}");
    for (host, origin) in [
        ("127.0.0.1:8000", None),
        ("localhost:8000", Some("https://localhost:9999")),
        ("[::1]:8000", Some("http://127.0.0.1:80")),
        ("engineering.example", Some("http://engineering.example")),
        ("engineering.example", Some("https://trusted.example")),
    ] {
        policy.authorize(Some(host), origin, Some(&auth)).unwrap();
    }
    assert_eq!(
        policy
            .authorize(Some("foreign.example"), None, Some(&auth))
            .unwrap_err()
            .status,
        421
    );
    assert_eq!(
        policy
            .authorize(
                Some("127.0.0.1"),
                Some("https://foreign.example"),
                Some(&auth)
            )
            .unwrap_err()
            .status,
        403
    );
    assert_eq!(
        policy
            .authorize(Some("127.0.0.1"), None, None)
            .unwrap_err()
            .status,
        401
    );
    let stdio = ServerSettings {
        token_env: Some("MISSING_UNUSED_TOKEN".into()),
        ..Default::default()
    };
    HttpPolicy::from_settings(&stdio, &BTreeMap::new()).unwrap();
}
#[test]
fn targets_preserve_operator_http_policy_and_exact_child_configuration() {
    let directory = Directory::new();
    let env = BTreeMap::from([
        ("EAL_MCP_CONFIG".into(), "ignored.toml".into()),
        ("EAL_MCP_HOST".into(), "remote.example".into()),
        ("RETAINED".into(), "yes".into()),
        ("TOKEN".into(), "synthetic-secret".into()),
    ]);
    assert!(
        client::create_http_target("http://remote.example/mcp", None, &env)
            .unwrap_err()
            .contains("beyond loopback")
    );
    assert!(client::create_http_target("http://2130706433/mcp", None, &env).is_err());
    let target =
        client::create_http_target("http://remote.example/mcp", Some("TOKEN"), &env).unwrap();
    assert!(!format!("{target:?}").contains("synthetic-secret"));
    let settings = ServerSettings {
        workspace: directory.0.clone(),
        methods: "all".into(),
        known_entries: vec!["registered".into()],
        exposure: "registered".into(),
        ..Default::default()
    };
    let ClientTarget::Stdio(target) =
        host::stdio_target(&settings, &env, PathBuf::from("/operator/eal-mcp"))
    else {
        panic!()
    };
    assert_eq!(target.command, PathBuf::from("/operator/eal-mcp"));
    assert!(
        target
            .environment
            .keys()
            .all(|k| !k.starts_with("EAL_MCP_"))
    );
    assert_eq!(target.environment["RETAINED"], "yes");
    assert_eq!(target.arguments, host::server_arguments(&settings));
    assert!(!target.arguments.contains(&"--config".into()));
    assert!(
        target
            .arguments
            .windows(2)
            .any(|pair| pair == ["--methods", "all"])
    );
    let wildcard = ServerSettings {
        host: "::".into(),
        ..Default::default()
    };
    assert!(host::configured_http_url(&wildcard).is_err());
}

#[derive(Clone)]
struct RemoteServer {
    calls: std::sync::Arc<AtomicUsize>,
    mode: usize,
    token: String,
}
impl rmcp::ServerHandler for RemoteServer {
    fn get_info(&self) -> rmcp::model::ServerConfig {
        rmcp::model::ServerConfig::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
    }
    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        let required = if self.mode == 2 {
            json!(["extra"])
        } else {
            json!([])
        };
        let tool=serde_json::from_value(json!({"name":"eal_describe","inputSchema":{"type":"object","properties":{"extra":{"type":"string"}},"required":required,"additionalProperties":false}})).unwrap();
        Ok(rmcp::model::ListToolsResult::with_all_items(vec![tool]))
    }
    async fn call_tool(
        &self,
        _request: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResponse, rmcp::ErrorData> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let value = if self.mode == 3 {
            let mut values = serde_json::Map::new();
            values.insert(self.token.clone(), json!("first"));
            values.insert("[redacted]".into(), json!("second"));
            Value::Object(values)
        } else if matches!(self.mode, 4 | 6) {
            json!({"status":"supported","measurement":42})
        } else if self.mode == 5 {
            json!({"status":"supported"})
        } else {
            json!({"remote":self.token})
        };
        Ok(if matches!(self.mode, 1 | 6) {
            rmcp::model::CallToolResult::structured_error(value)
        } else {
            rmcp::model::CallToolResult::structured(value)
        }
        .into())
    }
}
#[tokio::test]
async fn actual_host_binary_checks_discovered_schema_rejects_semantic_leakage_and_preserves_results()
 {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
    };
    use std::{process::Stdio, sync::Arc, time::Duration};
    use tokio::io::AsyncWriteExt;
    for mode in 0..7 {
        let directory = Directory::new();
        let workspace = directory.0.join("must-not-be-created");
        let calls = Arc::new(AtomicUsize::new(0));
        let token = match mode {
            3 => "Credential",
            5 => "supported",
            _ => "synthetic-host-token",
        };
        let server = RemoteServer {
            calls: Arc::clone(&calls),
            mode,
            token: token.into(),
        };
        let cancel = tokio_util::sync::CancellationToken::new();
        let service = StreamableHttpService::new(
            move || Ok(server.clone()),
            Arc::new(NeverSessionManager::default()),
            StreamableHttpServerConfig::default()
                .with_legacy_session_mode(false)
                .with_json_response(true)
                .with_stateless_protocol_metadata_required(false)
                .with_cancellation_token(cancel.clone()),
        );
        let router = axum::Router::new().route_service("/mcp", service);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/mcp", listener.local_addr().unwrap());
        let shutdown = cancel.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown.cancelled_owned())
                .await
                .unwrap();
        });
        let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_eal-host"))
            .args([
                "--url",
                &url,
                "--workspace",
                workspace.to_str().unwrap(),
                "--token-env",
                "TOKEN",
                "--timeout",
                "5",
            ])
            .env_clear()
            .env("TOKEN", token)
            .env("NO_PROXY", "*")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"{\"operation\":\"describe\"}")
            .await
            .unwrap();
        let output = tokio::time::timeout(Duration::from_secs(8), child.wait_with_output())
            .await
            .unwrap()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let result = eal_core::json::parse(&stdout, &eal_core::ExecutionLimits::default()).unwrap();
        assert!(!stdout.contains(token), "mode {mode}: {stdout}");
        assert!(!workspace.exists());
        assert_eq!(
            output.status.code(),
            Some(match mode {
                4 => 0,
                6 => 1,
                _ => 2,
            }),
            "mode {mode}: {stdout}; stderr {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(calls.load(Ordering::SeqCst), usize::from(mode != 2));
        if matches!(mode, 4 | 6) {
            assert_eq!(
                result["result"],
                json!({"status":"supported","measurement":42})
            );
            assert_eq!(result["protocol_version"], "2025-11-25");
            assert_eq!(result["is_error"], mode == 6);
        } else if mode == 2 {
            assert!(
                result["error"]
                    .as_str()
                    .unwrap()
                    .contains("Discovered input schema rejected request")
            );
        } else {
            assert!(result["is_error"].as_bool().unwrap());
            assert!(result.get("result").is_none());
            assert!(
                result["error"]
                    .as_str()
                    .unwrap()
                    .contains("EAL_HOST_CREDENTIAL_LEAK")
            );
        }
        cancel.cancel();
        task.await.unwrap();
    }
}

#[tokio::test]
async fn direct_client_deadline_values_fail_without_panicking_or_starting_a_child() {
    for timeout in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e30, 1e19] {
        let target = ClientTarget::Stdio(client::StdioTarget {
            command: PathBuf::from("/must-not-start-eal-host-deadline-test"),
            arguments: vec![],
            environment: BTreeMap::new(),
            cwd: None,
        });
        let error = client::dispatch_tool("eal_describe", &json!({}), target, timeout)
            .await
            .unwrap_err();
        assert!(error.contains("Host timeout"), "{timeout}: {error}");
    }
}

#[test]
fn packet_retrieval_and_budgets_are_checked_before_connection() {
    let (tool, args) = host::parse_request(r#"{"operation":"get_known_packet","entry_id":"demo","assessment_id":"stored","claim":"works","packet_limits":{"max_bytes":4096,"max_statement_bytes":128}}"#).unwrap();
    assert_eq!(tool, "eal_get_known_packet");
    assert_eq!(args["packet_limits"]["max_bytes"], 4096);
    for operation in ["packet", "assess_known", "get_known_packet"] {
        let mut request = match operation {
            "packet" => json!({"operation":operation,"assessment_id":"stored"}),
            "assess_known" => json!({"operation":operation,"entry_id":"demo","claim":"works"}),
            _ => {
                json!({"operation":operation,"entry_id":"demo","assessment_id":"stored","claim":"works"})
            }
        };
        for budget in [
            json!(null),
            json!({"max_bytes":4096.0}),
            json!({"max_bytes":65537}),
            json!({"max_statement_bytes":0}),
            json!({"max_claims":1}),
        ] {
            request["packet_limits"] = budget;
            assert!(
                host::parse_request(&request.to_string()).is_err(),
                "{request}"
            );
        }
        request["packet_limits"] = json!({});
        assert!(host::parse_request(&request.to_string()).is_ok());
    }
    for text in [
        r#"{"operation":"get_known_packet","entry_id":"demo","assessment_id":"stored","claim":null}"#,
        r#"{"operation":"get_known_packet","entry_id":"","assessment_id":"stored","claim":"works"}"#,
        r#"{"operation":"get_known_packet","entry_id":"demo","assessment_id":"","claim":"works"}"#,
        r#"{"operation":"get_known_packet","entry_id":"demo","assessment_id":"stored","claim":"works","packet_limits":{"max_bytes":4.096e3}}"#,
    ] {
        assert!(host::parse_request(text).is_err(), "{text}");
    }
}

#[test]
fn native_requests_urls_and_redaction_have_explicit_boundaries() {
    let (tool, args) =
        host::parse_request(r#"{"operation":"plan","source":"source","claim":"outcome"}"#).unwrap();
    assert_eq!(tool, "eal_plan");
    assert_eq!(args, json!({"source":"source","claim":"outcome"}));
    for text in [
        "{}{}",
        r#"{"operation":"describe"} false"#,
        r#"{"operation":"eal_describe"}"#,
        r#"{"operation":"describe","extra":true}"#,
    ] {
        assert!(host::parse_request(text).is_err());
    }
    assert!(
        host::parse_request(&" ".repeat(host::MAX_REQUEST_BYTES + 1))
            .unwrap_err()
            .contains("2 MiB")
    );
    assert_eq!(
        client::validate_http_url("http://localhost:8000/mcp").unwrap(),
        "http://localhost:8000/mcp"
    );
    for url in [
        "file:///tmp/socket",
        "http://user:password@localhost/mcp",
        "http://localhost/mcp#fragment",
    ] {
        assert!(client::validate_http_url(url).is_err(), "{url}");
    }
    let redacted = redaction::sanitise_response(
        &json!({"is_error":true,"error":"before TOKEN after"}),
        Some("TOKEN"),
    )
    .unwrap();
    assert_eq!(
        redacted,
        json!({"is_error":true,"error":"before [redacted] after"})
    );
}

#[test]
fn semantic_credentials_fail_without_rewriting_states_evidence_keys_or_numbers() {
    let unchanged = json!({"operation":"reason","protocol_version":"2025-11-25","is_error":false,"result":{"status":"supported","value":42},"content":[]});
    assert_eq!(
        redaction::sanitise_response(&unchanged, Some("unrelated-credential")).unwrap(),
        unchanged
    );
    for (token, result) in [
        ("supported", json!({"status":"supported"})),
        ("secret", json!({"evidence":{"measurement":"secret"}})),
        ("secret", json!({"nested":[{"secret":"measurement"}]})),
        ("42", json!({"measurement":42})),
        ("true", json!({"measurement":true})),
        ("null", json!({"measurement":null})),
        (
            "secret",
            json!({"measurement":"{\"value\":\"\\u0073ecret\"}"}),
        ),
        ("secret", json!({"measurement":"\"\\u0073ecret\""})),
        ("secret", json!({"{\"\\u0073ecret\":true}":"public"})),
    ] {
        let response = json!({"operation":"reason","protocol_version":"2025-11-25","is_error":false,"result":result,"content":[]});
        assert!(
            redaction::sanitise_response(&response, Some(token)).is_err(),
            "{response}"
        );
    }
    let protocol_collision = json!({"operation":"reason","protocol_version":"2025-11-25","is_error":false,"result":{},"content":[]});
    assert!(redaction::sanitise_response(&protocol_collision, Some("reason")).is_err());
    let encoded = json!({"is_error":true,"error":"{\"diagnostic\":\"\\u0073ecret\"}"});
    let diagnostic = redaction::sanitise_response(&encoded, Some("secret")).unwrap();
    assert_eq!(diagnostic, json!({"is_error":true,"error":"[redacted]"}));
    let ambiguous = json!({"operation":"reason","protocol_version":"2025-11-25","is_error":false,"result":{"evidence":"{\"value\":\"public\",\"value\":\"\\u0073ecret\"}"},"content":[]});
    assert!(
        redaction::sanitise_response(&ambiguous, Some("secret"))
            .unwrap_err()
            .contains("EAL_HOST_CREDENTIAL_INSPECTION")
    );
}

#[test]
fn server_configuration_uses_a_capped_regular_file_before_parsing() {
    let directory = Directory::new();
    let config = directory.0.join("server.toml");
    let parse = |path: &std::path::Path| {
        settings::parse_server_settings(
            &strings(&["--config", path.to_str().unwrap()]),
            &BTreeMap::new(),
            &directory.0,
        )
    };
    let mut source = vec![b' '; 1_048_576];
    source[0] = b'\n';
    std::fs::write(&config, &source).unwrap();
    assert!(parse(&config).is_ok());
    std::fs::File::create(&config)
        .unwrap()
        .set_len(1_048_577)
        .unwrap();
    assert!(parse(&config).unwrap_err().contains("exceeds 1 MiB"));
    std::fs::write(&config, [0xff]).unwrap();
    assert!(parse(&config).unwrap_err().contains("UTF-8"));
    assert!(parse(&directory.0).is_err());
    assert!(parse(&directory.0.join("missing")).is_err());
}

#[cfg(unix)]
#[test]
fn server_configuration_fifo_and_fifo_alias_are_rejected_before_open_blocks() {
    let directory = Directory::new();
    let fifo = directory.0.join("server.fifo");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let alias = directory.0.join("server-alias");
    std::os::unix::fs::symlink(&fifo, &alias).unwrap();
    for path in [fifo, alias] {
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        let cwd = directory.0.clone();
        std::thread::spawn(move || {
            let result = settings::parse_server_settings(
                &strings(&["--config", path.to_str().unwrap()]),
                &BTreeMap::new(),
                &cwd,
            );
            let _ = send.send(result);
        });
        assert!(
            receive
                .recv_timeout(std::time::Duration::from_secs(2))
                .expect("Config FIFO must reject without a writer")
                .is_err()
        );
    }
}

#[test]
fn ordinary_prose_with_json_punctuation_retains_its_exact_meaning() {
    let response = json!({"operation":"reason","protocol_version":"2025-11-25","is_error":false,
        "result":{"[Results]":"[Results] calibration passed","{draft}":"{draft} status","quotation":"\"quoted prose remains incomplete"},"content":[]});
    assert_eq!(
        redaction::sanitise_response(&response, Some("unrelated-bearer-credential")).unwrap(),
        response
    );
    let diagnostic = json!({"is_error":true,"error":"[Results] calibration unavailable"});
    assert_eq!(
        redaction::sanitise_response(&diagnostic, Some("unrelated-bearer-credential")).unwrap(),
        diagnostic
    );
    assert!(redaction::sanitise_response(&response, Some("calibration")).is_err());
}
