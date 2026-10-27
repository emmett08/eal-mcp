#![cfg(unix)]
//! Pinned native profiles and actual SDK child exposure agree before and after acquisition.
use eal_coordination::{
    contracts::{ServerSpec, TaskSpec},
    gateway::{CallGate, GatewayOptions, McpGateway},
    host::HostPaths,
    persistent_mcp::PersistentMcp,
    profiles,
};
use eal_mcp::{
    client::{ClientTarget, StdioTarget},
    host,
    settings::ServerSettings,
};
use eal_runtime::catalogue::RegisterOptions;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
const SOURCE: &str = "language \"EAL/4\"\nenvironment lab {\n require true\n}\ntool reader {\n version \"1\"\n}\nevidence reading {\n tool reader\n kind test\n environment lab\n max_age 3600\n input {}\n require passed == true\n}\nreasoning measured {\n method \"structured/1\"\n rationale \"The supplied report supports this bounded claim.\"\n}\nclaim works {\n statement \"The supplied synthetic test passed.\"\n environment lab\n}\nargument support = [evidence reading] via measured => works\n";
struct Fixture {
    path: PathBuf,
    server: ServerSpec,
    host: HostPaths,
    anchor: Value,
}
impl Fixture {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("eal-native-gateway-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("source.eal"), SOURCE).unwrap();
        std::fs::write(
            path.join("tools.toml"),
            "[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n",
        )
        .unwrap();
        std::fs::write(path.join("observation.json"),json!({"observed_at":eal_runtime::store::utc_now(),"context":{},"request":{"tool":"reader","tool_version":"1","input":{},"context":{}},"value":{"passed":true,"private":"collector-private-value"}}).to_string()).unwrap();
        let server=ServerSpec::from_value(&json!({"workspace":path,"registry":path.join("tools.toml"),"database":null,"methods":"engineering","limits":null}),&path).unwrap();
        let host = HostPaths::from_directory(
            Path::new(env!("CARGO_BIN_EXE_eal-coordinate"))
                .parent()
                .unwrap(),
        );
        for file in [&host.eal_mcp, &host.supervisor, &host.method_worker] {
            assert!(
                file.is_file(),
                "Build all native workspace companions before integration tests: {}",
                file.display()
            );
        }
        let knowledge = profiles::knowledge_with_host(&server, &host).unwrap();
        let claims = vec!["works".into()];
        knowledge
            .catalogue
            .register(
                "source.eal",
                RegisterOptions {
                    entry_id: Some("sample"),
                    context: Some(&json!({})),
                    claims: Some(&claims),
                    ..Default::default()
                },
            )
            .unwrap();
        let task=TaskSpec::from_value(&json!({"task_id":"bounded","question":"What does the registered report support?","entry_id":"sample","claims":["works"],"profile":"registered_claims/1","requirements":{"context":{}}})).unwrap();
        let anchor = profiles::pin_task_with_host(&server, &task, &host).unwrap();
        Self {
            path,
            server,
            host,
            anchor,
        }
    }
    fn options(&self) -> GatewayOptions {
        let mut options = GatewayOptions::for_host(&self.host).unwrap();
        options.operation_timeout = Duration::from_secs(10);
        options
            .environment
            .insert("EAL_MCP_TRANSPORT".into(), "http".into());
        options
            .environment
            .insert("EAL_MCP_EXPOSURE".into(), "operator".into());
        options
            .environment
            .insert("EAL_MCP_KNOWN_ENTRIES".into(), "[\"unregistered\"]".into());
        options
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
#[test]
fn fresh_native_registry_and_child_gateway_ignore_resolved_launcher_environment() {
    let fixture = Fixture::new();
    assert_eq!(
        profiles::check_task_with_host(&fixture.server, &fixture.anchor, &fixture.host).unwrap()["ok"],
        true
    );
    let settings = ServerSettings {
        workspace: fixture.server.workspace.clone(),
        registry: fixture.server.registry.clone(),
        database: fixture.server.database.clone(),
        methods: "engineering".into(),
        ..Default::default()
    };
    let mut environment = eal_mcp::application::environment().unwrap();
    environment.insert("EAL_MCP_PATH".into(), "/competing-launcher-path".into());
    let session = PersistentMcp::new(
        ClientTarget::Stdio(StdioTarget {
            command: fixture.host.eal_mcp.clone(),
            arguments: host::server_arguments(&settings),
            environment,
            cwd: None,
        }),
        Duration::from_secs(10),
    )
    .unwrap();
    let description = session
        .call("eal_describe", &json!({}), deadline())
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(
        description["method_registry_fingerprint"],
        fixture.anchor["method_registry_fingerprint"]
    );
    session.close();
    let mut gateway = McpGateway::new(
        fixture.server.clone(),
        fixture.anchor.clone(),
        fixture.options(),
        None,
    )
    .unwrap();
    let finding = gateway.assess(deadline()).unwrap();
    assert_eq!(finding["check"]["ok"], true);
    assert_eq!(finding["collected_count"], 1);
    assert_eq!(finding["assessments"]["works"]["status"], "supported");
    let calls = gateway.calls();
    assert_eq!(
        calls
            .iter()
            .map(|call| call["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "eal_sources",
            "eal_assess_known",
            "eal_packet",
            "eal_explain"
        ]
    );
    assert_eq!(
        calls[1]["arguments"],
        json!({"entry_id":"sample","claim":"works"})
    );
    assert!(calls.iter().all(|call| call["outcome"] == "success"));
    let mut lookup = McpGateway::new(
        fixture.server.clone(),
        fixture.anchor.clone(),
        fixture.options(),
        None,
    )
    .unwrap();
    assert_eq!(lookup.lookup(&finding, deadline()).unwrap(), finding);
    assert!(
        lookup
            .calls()
            .iter()
            .all(|call| ["eal_packet", "eal_explain"].contains(&call["name"].as_str().unwrap()))
    );
    let mut later = McpGateway::new(
        fixture.server.clone(),
        fixture.anchor.clone(),
        fixture.options(),
        None,
    )
    .unwrap();
    let reused = later.assess(deadline()).unwrap();
    assert_eq!(reused["collected_count"], 0);
    assert_eq!(reused["reused_count"], 1);
    let mut tampered = finding;
    tampered["assessments"]["works"]["status"] = json!("contested");
    assert!(
        lookup
            .lookup(&tampered, deadline())
            .unwrap_err()
            .contains("identity")
    );
}
#[test]
fn budget_gate_is_traced_before_effect_and_stale_source_makes_no_mcp_calls() {
    let fixture = Fixture::new();
    let gate: CallGate = Arc::new(|_, _, _| Err("tool quota exhausted".into()));
    let mut gateway = McpGateway::new(
        fixture.server.clone(),
        fixture.anchor.clone(),
        fixture.options(),
        Some(gate),
    )
    .unwrap();
    assert!(gateway.assess(deadline()).unwrap_err().contains("quota"));
    assert_eq!(
        gateway.calls(),
        [
            json!({"name":"eal_sources","arguments":{},"exposure":"registered","outcome":"not_called"})
        ]
    );
    std::fs::write(
        fixture.path.join("source.eal"),
        format!("{SOURCE}\n// Changed after pinning.\n"),
    )
    .unwrap();
    let mut gateway = McpGateway::new(
        fixture.server.clone(),
        fixture.anchor.clone(),
        fixture.options(),
        None,
    )
    .unwrap();
    let finding = gateway.assess(deadline()).unwrap();
    assert_eq!(finding["check"]["ok"], false);
    assert_eq!(finding["assessments"], json!({}));
    assert_eq!(finding["collected_count"], 0);
    assert!(gateway.calls().is_empty());
}

#[test]
fn an_explicit_minimal_command_worker_environment_can_use_the_pinned_engineering_gateway() {
    let fixture = Fixture::new();
    let task = TaskSpec::from_value(&fixture.anchor["task"]).unwrap();
    let coordinator = eal_coordination::coordinator::Coordinator::start_with_host(
        fixture.path.join("coordination-run"),
        fixture.server.clone(),
        task,
        None,
        None,
        fixture.host.clone(),
    )
    .unwrap();
    let reservation = coordinator
        .reserve_worker("assessor", "software_test")
        .unwrap();
    let argv = reservation["context"]["gateway"]["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let output = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .env_clear()
        .env("PATH", "")
        .env("PRIVATE_PROVIDER_SETTING", "synthetic-private-worker-value")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let finding = eal_core::json::parse(
        std::str::from_utf8(&output.stdout).unwrap(),
        &eal_core::ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(finding["check"]["ok"], true, "{finding}");
    assert_eq!(finding["assessments"]["works"]["status"], "supported");
    assert_eq!(finding["collected_count"], 1);
    assert_eq!(
        finding["packets"]["works"]["method_registry_fingerprint"],
        coordinator.anchor["method_registry_fingerprint"]
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("synthetic-private-worker-value"));
}

#[test]
fn a_gateway_rejects_sidecars_that_differ_from_the_actual_mcp_child_runtime() {
    let fixture = Fixture::new();
    let mut options = fixture.options();
    options.host_paths.method_worker = options.host_paths.supervisor.clone();
    assert!(
        McpGateway::new(
            fixture.server.clone(),
            fixture.anchor.clone(),
            options,
            None
        )
        .err()
        .unwrap()
        .contains("companions differ")
    );
}
