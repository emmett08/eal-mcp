//! Native persistent SDK sessions, immutable delivery and terminal transport failures.
use eal_coordination::{
    bus::{Mailbox, MailboxLimits},
    lab_mcp::LabMcpTransport,
    persistent_mcp::{McpError, PersistentMcp},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-mcp-adapters-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn database(&self) -> PathBuf {
        self.0.join("mailbox.sqlite")
    }
    fn transport(&self) -> LabMcpTransport {
        LabMcpTransport::with_options(
            Path::new(env!("CARGO_BIN_EXE_eal-mas-server"))
                .parent()
                .unwrap(),
            self.database(),
            BTreeMap::from([
                ("PATH".into(), "".into()),
                ("SYNTHETIC_API_KEY".into(), "private-provider-value".into()),
            ]),
            Duration::from_secs(3),
        )
        .unwrap()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn message(id: &str) -> Value {
    json!({"schema":"EAL/MAS-coordination/3","kind":"request","message_id":id,"task_id":"synthetic","task_digest":"a".repeat(64),"sender":"reviewer","recipient":"assessor","reply_to":null,"attempt":0})
}
#[test]
fn lazy_close_and_invalid_operation_never_start_a_mailbox_server() {
    let fixture = Workspace::new();
    let bus = fixture.transport();
    assert_eq!(bus.diagnostics()["server_starts"], 0);
    assert!(
        bus.mcp_call("mas_send", &json!({}))
            .unwrap_err()
            .contains("not exposed")
    );
    assert_eq!(bus.diagnostics()["server_starts"], 0);
    assert!(!fixture.database().exists());
    bus.close();
    bus.close();
    assert!(bus.status("run").unwrap_err().contains("closed"));
    assert_eq!(bus.diagnostics()["server_starts"], 0);
    assert!(!fixture.database().exists());
}
#[test]
fn persistent_mailbox_reuses_one_server_and_keeps_delivery_and_call_ledger_semantics() {
    let fixture = Workspace::new();
    let bus = fixture.transport();
    let info = bus.mcp_call("mas_runtime_info", &json!({})).unwrap();
    assert!(!fixture.database().exists());
    let pid = info["server_pid"].as_u64().unwrap();
    assert_ne!(pid, u64::from(std::process::id()));
    bus.create(
        "run",
        &["reviewer".into(), "assessor".into()],
        MailboxLimits {
            max_messages: 2,
            max_bytes: 65536,
            max_calls: 1,
        },
    )
    .unwrap();
    let item = message("request");
    let sent = bus.send("run", &item, "reviewer").unwrap();
    assert_eq!(sent["duplicate"], false);
    assert_eq!(
        bus.send("run", &item, "reviewer").unwrap()["duplicate"],
        true
    );
    assert!(bus.send("run", &message("request"), "assessor").is_err());
    assert!(bus.acknowledge("run", "request", "assessor").is_err());
    let first = bus.receive("run", "assessor", 8).unwrap();
    assert_eq!(first[0]["message"]["payload"], item);
    assert_eq!(first[0]["redelivered"], false);
    assert_eq!(
        bus.receive("run", "assessor", 8).unwrap()[0]["redelivered"],
        true
    );
    assert_eq!(
        bus.acknowledge("run", "request", "assessor").unwrap()["acknowledged"],
        true
    );
    assert_eq!(bus.receive("run", "assessor", 8).unwrap(), json!([]));
    assert_eq!(
        bus.reserve("run", "assessor", "worker-call").unwrap()["reserved"],
        true
    );
    let evidence = json!({"execution_id":"software/turn1","execution_mode":"software_test","actual_agent_execution":false,"tokens":null,"cost_usd":null});
    assert_eq!(
        bus.finish("run", "assessor", "worker-call", &evidence)
            .unwrap()["recorded"],
        true
    );
    assert!(bus.reserve("run", "reviewer", "extra-call").is_err());
    let status = bus.status("run").unwrap();
    assert_eq!(status["usage"]["messages"], 1);
    assert_eq!(status["usage"]["calls"], 1);
    assert_eq!(status["calls"][0]["state"], "recorded");
    assert_eq!(status["calls"][0]["evidence"], evidence);
    let current = bus.mcp_call("mas_runtime_info", &json!({})).unwrap();
    assert_eq!(current["server_pid"], pid);
    assert_eq!(bus.diagnostics()["server_starts"], 1);
    assert_eq!(bus.diagnostics()["codec_mode"], "in_process");
    bus.close();
    assert_eq!(bus.diagnostics()["closed"], true);
    #[cfg(unix)]
    assert_eq!(
        rustix::process::test_kill_process(rustix::process::Pid::from_raw(pid as i32).unwrap()),
        Err(rustix::io::Errno::SRCH)
    );
}
#[test]
fn local_and_remote_schema_rejections_preserve_the_session_without_creating_or_routing_mail() {
    let fixture = Workspace::new();
    let bus = fixture.transport();
    let mut item = message("invalid");
    item["fraction"] = json!(0.25);
    assert!(bus.send("unused", &item, "reviewer").is_err());
    assert!(!fixture.database().exists());
    assert!(bus.validate_wire("not MAS wire").is_err());
    assert!(!fixture.database().exists());
    bus.create(
        "run",
        &["reviewer".into(), "assessor".into()],
        MailboxLimits::default(),
    )
    .unwrap();
    assert_eq!(bus.status("run").unwrap()["usage"]["messages"], 0);
    bus.send("run", &message("valid"), "reviewer").unwrap();
    assert_eq!(bus.status("run").unwrap()["usage"]["messages"], 1);
    assert_eq!(bus.diagnostics()["server_starts"], 1);
}
#[cfg(unix)]
#[test]
fn a_dead_server_is_terminal_and_an_explicit_new_owner_can_reconcile() {
    let fixture = Workspace::new();
    let bus = fixture.transport();
    bus.create(
        "run",
        &["reviewer".into(), "assessor".into()],
        MailboxLimits::default(),
    )
    .unwrap();
    bus.send("run", &message("first"), "reviewer").unwrap();
    let pid = bus.mcp_call("mas_runtime_info", &json!({})).unwrap()["server_pid"]
        .as_i64()
        .unwrap();
    rustix::process::kill_process(
        rustix::process::Pid::from_raw(pid as i32).unwrap(),
        rustix::process::Signal::KILL,
    )
    .unwrap();
    assert!(
        bus.send("run", &message("never-replayed"), "reviewer")
            .is_err()
    );
    assert!(bus.status("run").unwrap_err().contains("closed"));
    assert_eq!(bus.diagnostics()["server_starts"], 1);
    let reopened = fixture.transport();
    assert_eq!(reopened.status("run").unwrap()["usage"]["messages"], 1);
}
#[derive(Clone)]
struct ProbeServer {
    calls: Arc<AtomicUsize>,
    delay: Duration,
}
impl rmcp::ServerHandler for ProbeServer {
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
        Ok(rmcp::model::ListToolsResult::with_all_items(vec![serde_json::from_value(json!({"name":"probe","inputSchema":{"type":"object","properties":{},"additionalProperties":false}})).unwrap()]))
    }
    async fn call_tool(
        &self,
        _request: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResponse, rmcp::ErrorData> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(self.delay).await;
        Ok(rmcp::model::CallToolResult::structured(json!({"observed":true})).into())
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn persistent_total_deadline_marks_uncertain_effect_terminal_without_replay() {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let server = ProbeServer {
        calls: Arc::clone(&calls),
        delay: Duration::from_secs(2),
    };
    let cancel = tokio_util::sync::CancellationToken::new();
    let shutdown = cancel.clone();
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
    let task = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown.cancelled_owned())
            .await
            .unwrap()
    });
    let target = eal_mcp::client::create_http_target_with_client(
        &url,
        None,
        &BTreeMap::new(),
        reqwest::Client::builder().no_proxy().build().unwrap(),
    )
    .unwrap();
    let session = Arc::new(PersistentMcp::new(target, Duration::from_secs(3)).unwrap());
    let owner = Arc::clone(&session);
    let error = tokio::task::spawn_blocking(move || {
        owner.call(
            "probe",
            &json!({}),
            Instant::now() + Duration::from_millis(250),
        )
    })
    .await
    .unwrap()
    .unwrap_err();
    assert!(matches!(error, McpError::Terminal(_)));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(matches!(
        session.call("probe", &json!({}), Instant::now() + Duration::from_secs(1)),
        Err(McpError::Terminal(_))
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(session.diagnostics()["server_starts"], 1);
    cancel.cancel();
    task.await.unwrap();
}
