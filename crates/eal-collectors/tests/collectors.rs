//! Local process and HTTP witnesses for the collector trust boundary.
use eal_collectors::{Config, Request};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

async fn collect(config: &Config, request: &Request) -> Result<Value, String> {
    let result = eal_collectors::collect(config, request).await?;
    let descriptor = eal_collectors::catalogue::native()
        .collectors
        .iter()
        .find(|descriptor| descriptor.id == config.provider.name())
        .unwrap();
    jsonschema::validator_for(&descriptor.result_schema)
        .unwrap()
        .validate(&result["value"])
        .unwrap();
    Ok(result)
}

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-collectors-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn inventory(&self, clusters: Value) -> PathBuf {
        let path = self.0.join("inventory.json");
        std::fs::write(
            &path,
            serde_json::to_vec(&json!({"revision":"operator-inventory-42","clusters":clusters}))
                .unwrap(),
        )
        .unwrap();
        path
    }
    #[cfg(unix)]
    fn script(&self, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = self.0.join(format!("fake-{}", uuid::Uuid::new_v4()));
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        path
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn request(input: Value) -> Request {
    Request::parse(&serde_json::to_vec(&json!({"evidence_id":"reading","environment":"production","tool":"reader","tool_version":"1","input":input,"context":{"stage":"production"}})).unwrap()).unwrap()
}
fn config(provider: &str, options: &[(&str, &str)]) -> Config {
    let mut args = vec![provider.to_owned()];
    for (key, value) in options {
        args.extend([(*key).into(), (*value).into()]);
    }
    Config::parse(args).unwrap()
}
fn cluster() -> Value {
    json!([{"id":"prod-a","context":"operator-prod-a","tempo_cluster":"telemetry-prod-a"}])
}
fn supervisor() -> PathBuf {
    let executable = std::env::current_exe().unwrap();
    let path = executable
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("eal-command-supervisor");
    assert!(
        path.is_file(),
        "Build eal-runtime binaries before focused collector tests"
    );
    path
}
fn resource() -> Value {
    json!({"apiVersion":"argoproj.io/v1alpha1","kind":"Application","metadata":{"name":"catalog-sync-operator","namespace":"argocd","uid":"uid-42","resourceVersion":"123","generation":7},"status":{"observedGeneration":7,"sync":{"status":"Synced"},"health":{"status":"Healthy"}}})
}
fn kube<'a>(inventory: &'a Path, executable: &'a Path, supervisor: &'a Path) -> Config {
    config(
        "kubernetes",
        &[
            ("--inventory", inventory.to_str().unwrap()),
            ("--executable", executable.to_str().unwrap()),
            ("--supervisor", supervisor.to_str().unwrap()),
            ("--resource", "applications.argoproj.io"),
            ("--kind", "Application"),
            ("--namespace", "argocd"),
            ("--name", "catalog-sync-operator"),
        ],
    )
}
fn kube_request() -> Request {
    request(
        json!({"resource":"applications.argoproj.io","namespace":"argocd","name":"catalog-sync-operator"}),
    )
}

#[test]
fn strict_requests_and_operator_selectors_reject_ambiguity() {
    assert!(Request::parse(br#"{"evidence_id":"a","evidence_id":"b"}"#).is_err());
    let invalid = json!({"evidence_id":"a","environment":"p","tool":"t","tool_version":"1","input":{},"context":{},"argv":["delete"]});
    assert!(Request::parse(&serde_json::to_vec(&invalid).unwrap()).is_err());
    assert!(Config::parse(vec!["github".into(), "--executable".into(), "gh".into()]).is_err());
    assert!(
        Config::parse(vec![
            "tempo".into(),
            "--url".into(),
            "http://foreign.example/api/search".into(),
            "--inventory".into(),
            "/tmp/inventory".into(),
            "--kind".into(),
            "Application".into(),
            "--name".into(),
            "catalog-sync-operator".into()
        ])
        .is_err()
    );
}

#[cfg(unix)]
#[tokio::test]
async fn named_kubernetes_resources_preserve_inventory_generation_and_exact_envelope() {
    let workspace = Workspace::new();
    let inventory = workspace.inventory(cluster());
    let expected = resource();
    let args = workspace.0.join("args");
    let script = workspace.script(&format!(
        "[ -z \"${{HOME:-}}\" ] || exit 77\nprintf '%s\\n' \"$@\" > '{}'\nprintf '%s' '{}'",
        args.display(),
        expected
    ));
    let configured = kube(&inventory, &script, &supervisor());
    let requested = kube_request();
    let result = collect(&configured, &requested).await.unwrap();
    assert_eq!(result["request"], requested.acquisition());
    assert_eq!(result["context"], requested.context);
    assert_eq!(result["value"]["all_present"], true);
    assert_eq!(result["value"]["cluster_count"], 1);
    assert_eq!(
        result["value"]["clusters"][0]["resource"]["metadata"]["generation"],
        7
    );
    assert_eq!(
        result["value"]["clusters"][0]["resource"]["status"],
        expected["status"]
    );
    assert_eq!(
        std::fs::read_to_string(args).unwrap(),
        "get\napplications.argoproj.io\ncatalog-sync-operator\n--namespace\nargocd\n--context\noperator-prod-a\n--ignore-not-found\n--output\njson\n--request-timeout=30s\n"
    );
    eal_runtime::acquisition::validate_envelope(
        &serde_json::to_vec(&result).unwrap(),
        false,
        &requested.context,
        &requested.acquisition(),
    )
    .unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn measured_resource_absence_differs_from_failed_acquisition() {
    let workspace = Workspace::new();
    let inventory = workspace.inventory(cluster());
    let supervisor = supervisor();
    let absent = workspace.script("exit 0");
    let result = collect(&kube(&inventory, &absent, &supervisor), &kube_request())
        .await
        .unwrap();
    assert_eq!(result["value"]["all_present"], false);
    assert_eq!(result["value"]["clusters"][0]["present"], false);
    for body in [
        "printf secret-token >&2; exit 23",
        "printf '{bad'",
        "printf '%s' '{\"metadata\":{}}'",
    ] {
        let failed = workspace.script(body);
        let error = collect(&kube(&inventory, &failed, &supervisor), &kube_request())
            .await
            .unwrap_err();
        assert!(!error.contains("secret-token"));
    }
    let failed = workspace.script("/bin/sleep 10");
    let mut configured = kube(&inventory, &failed, &supervisor);
    configured.timeout = std::time::Duration::from_secs(1);
    assert!(
        collect(&configured, &kube_request())
            .await
            .unwrap_err()
            .contains("timeout")
    );
}

#[cfg(unix)]
#[tokio::test]
async fn scope_mismatch_fails_before_running_external_commands() {
    let workspace = Workspace::new();
    let inventory = workspace.inventory(cluster());
    let marker = workspace.0.join("called");
    let script = workspace.script(&format!("/usr/bin/touch '{}'", marker.display()));
    let configured = kube(&inventory, &script, &supervisor());
    assert!(
        collect(
            &configured,
            &request(
                json!({"resource":"secrets","namespace":"argocd","name":"catalog-sync-operator"})
            )
        )
        .await
        .is_err()
    );
    assert!(!marker.exists());
    workspace.inventory(json!([{"id":"prod-a","context":"same","tempo_cluster":"a"},{"id":"prod-b","context":"same","tempo_cluster":"b"}]));
    assert!(collect(&configured, &kube_request()).await.is_err());
    assert!(!marker.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn github_checks_exact_commit_identity_and_uses_get_only() {
    let workspace = Workspace::new();
    let sha = "0123456789012345678901234567890123456789";
    let args = workspace.0.join("args");
    let script = workspace.script(&format!(
        "printf '%s\\n' \"$@\" > '{}'\nprintf '%s' '{}'",
        args.display(),
        json!({"sha":sha,"commit":{"tree":{"sha":sha}}})
    ));
    let supervisor = supervisor();
    let configured = config(
        "github",
        &[
            ("--executable", script.to_str().unwrap()),
            ("--supervisor", supervisor.to_str().unwrap()),
            ("--repository", "owner/repository"),
            ("--commit", sha),
        ],
    );
    let requested = request(json!({"repository":"owner/repository","commit":sha}));
    let result = collect(&configured, &requested).await.unwrap();
    assert_eq!(result["value"]["commit"], sha);
    assert_eq!(result["value"]["found"], true);
    assert_eq!(
        std::fs::read_to_string(args).unwrap(),
        format!(
            "api\n--hostname\ngithub.com\n--method\nGET\nrepos/owner/repository/commits/{sha}\n"
        )
    );
    let wrong = workspace.script("printf '%s' '{\"sha\":\"wrong\"}'");
    let mut configured = configured.clone();
    configured
        .options
        .insert("--executable".into(), wrong.to_string_lossy().into_owned());
    assert!(collect(&configured, &requested).await.is_err());
}

fn http(body: Value, status: u16) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut buffer = [0u8; 8192];
        let size = stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..size]).into_owned();
        let bytes = serde_json::to_vec(&body).unwrap();
        write!(stream,"HTTP/1.1 {status} test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",bytes.len()).unwrap();
        stream.write_all(&bytes).unwrap();
        request
    });
    (format!("http://{address}/api/search"), handle)
}
fn tempo(inventory: &Path, url: &str) -> Config {
    config(
        "tempo",
        &[
            ("--inventory", inventory.to_str().unwrap()),
            ("--url", url),
            ("--kind", "Application"),
            ("--name", "catalog-sync-operator"),
            ("--limit", "1"),
        ],
    )
}
fn window() -> Request {
    let end = chrono::Utc::now().timestamp() as u64;
    request(json!({"start_unix":end-60,"end_unix":end}))
}

#[tokio::test]
async fn tempo_uses_scoped_encoded_traceql_and_retains_bounded_positive_evidence() {
    let workspace = Workspace::new();
    let inventory = workspace.inventory(cluster());
    let (url, server) = http(
        json!({"traces":[{"traceID":"01234567890123456789012345678901"}],"metrics":{"completedJobs":1,"totalJobs":3}}),
        200,
    );
    let result = collect(&tempo(&inventory, &url), &window()).await.unwrap();
    let received = server.join().unwrap();
    let target = received
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap();
    let parsed = url::Url::parse(&format!("http://local{target}")).unwrap();
    let params = parsed
        .query_pairs()
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        params["q"],
        "{ span.dpe.rollout_kind = \"Application\" && span.dpe.rollout_name = \"catalog-sync-operator\" && resource.k8s.cluster.name = \"telemetry-prod-a\" }"
    );
    assert_eq!(params["limit"], "1");
    assert_eq!(result["value"]["all_matched"], true);
    assert_eq!(result["value"]["search_exhaustive"], false);
    assert_eq!(result["value"]["negative_inference"], "unavailable");
    assert_eq!(result["value"]["clusters"][0]["result_limit_reached"], true);
    assert_eq!(result["value"]["clusters"][0]["reported_partial"], true);
}

#[tokio::test]
async fn tempo_empty_search_preserves_no_match_and_errors_remain_acquisition_failures() {
    let workspace = Workspace::new();
    let inventory = workspace.inventory(cluster());
    let (url, server) = http(json!({"traces":[]}), 200);
    let result = collect(&tempo(&inventory, &url), &window()).await.unwrap();
    server.join().unwrap();
    assert_eq!(result["value"]["all_matched"], false);
    assert_eq!(result["value"]["negative_inference"], "unavailable");
    for (body, status) in [
        (json!({"traces":[]}), 503),
        (json!({"error":"secret-provider-error"}), 200),
        (json!({"metrics":{}}), 200),
        (json!({"traces":[{"traceID":"forged"}]}), 200),
    ] {
        let (url, server) = http(body, status);
        let error = collect(&tempo(&inventory, &url), &window())
            .await
            .unwrap_err();
        server.join().unwrap();
        assert!(!error.contains("secret-provider-error"));
    }
}

#[cfg(unix)]
#[test]
fn binary_failure_has_no_success_envelope() {
    let mut process = Command::new(env!("CARGO_BIN_EXE_eal-collector"))
        .args([
            "github",
            "--executable",
            "/bin/false",
            "--supervisor",
            "/bin/false",
            "--repository",
            "owner/repo",
            "--commit",
            "0123456789012345678901234567890123456789",
        ])
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    process.stdin.take().unwrap().write_all(b"{}").unwrap();
    let result = process.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
}

#[test]
fn tempo_credentials_never_reach_success_or_error_output() {
    let workspace = Workspace::new();
    let inventory = workspace.inventory(cluster());
    let secret = "synthetic-bearer.ABC_123";
    let (url, server) = http(
        json!({"traces":[{"traceID":"01234567890123456789012345678901","rootServiceName":secret}]}),
        200,
    );
    let mut process = Command::new(env!("CARGO_BIN_EXE_eal-collector"))
        .args([
            "tempo",
            "--inventory",
            inventory.to_str().unwrap(),
            "--url",
            &url,
            "--kind",
            "Application",
            "--name",
            "catalog-sync-operator",
            "--token-env",
            "TEST_BEARER",
        ])
        .env_clear()
        .env("TEST_BEARER", secret)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let requested = window();
    process
        .stdin
        .take()
        .unwrap()
        .write_all(
            &serde_json::to_vec(&json!({
        "evidence_id":requested.evidence_id,"environment":requested.environment,
        "tool":requested.tool,"tool_version":requested.tool_version,"input":requested.input,
        "context":requested.context}))
            .unwrap(),
        )
        .unwrap();
    let result = process.wait_with_output().unwrap();
    let received = server.join().unwrap();
    assert!(received.contains(secret));
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&result.stderr).contains(secret));
}

#[cfg(unix)]
#[tokio::test]
async fn inventory_symlinks_and_aggregate_response_overflow_are_rejected() {
    use std::os::unix::fs::symlink;
    let workspace = Workspace::new();
    let inventory = workspace.inventory(cluster());
    let alias = workspace.0.join("inventory-alias.json");
    symlink(&inventory, &alias).unwrap();
    let script = workspace.script(&format!("printf '%s' '{}'", resource()));
    assert!(
        collect(&kube(&alias, &script, &supervisor()), &kube_request())
            .await
            .is_err()
    );
    workspace.inventory(json!([
        {"id":"prod-a","context":"context-a","tempo_cluster":"telemetry-a"},
        {"id":"prod-b","context":"context-b","tempo_cluster":"telemetry-b"}
    ]));
    let mut selected = kube(&inventory, &script, &supervisor());
    selected.max_bytes = 512;
    assert!(
        collect(&selected, &kube_request())
            .await
            .unwrap_err()
            .contains("allowance")
    );
}

#[tokio::test]
async fn tempo_http_body_has_a_hard_byte_limit() {
    let workspace = Workspace::new();
    let inventory = workspace.inventory(cluster());
    let (url, server) = http(json!({"traces":[], "oversized":"x".repeat(1024)}), 200);
    let mut selected = tempo(&inventory, &url);
    selected.max_bytes = 256;
    assert!(
        collect(&selected, &window())
            .await
            .unwrap_err()
            .contains("byte allowance")
    );
    server.join().unwrap();
}

#[cfg(unix)]
#[test]
fn fifo_inventory_is_rejected_before_a_blocking_read_for_both_inventory_providers() {
    use std::time::{Duration, Instant};
    let workspace = Workspace::new();
    let fifo = workspace.0.join("inventory.fifo");
    rustix::fs::mkfifoat(
        rustix::fs::CWD,
        &fifo,
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
    )
    .unwrap();
    let marker = workspace.0.join("external-read");
    let executable = workspace.script(&format!("/usr/bin/touch '{}'", marker.display()));
    for provider in ["kubernetes", "tempo"] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_eal-collector"));
        command.arg(provider).arg("--inventory").arg(&fifo).args([
            "--kind",
            "Application",
            "--name",
            "catalog-sync-operator",
            "--timeout-seconds",
            "1",
        ]);
        let input = if provider == "kubernetes" {
            command
                .arg("--executable")
                .arg(&executable)
                .arg("--supervisor")
                .arg(supervisor())
                .args([
                    "--resource",
                    "applications.argoproj.io",
                    "--namespace",
                    "argocd",
                ]);
            kube_request().input
        } else {
            command.args(["--url", "http://127.0.0.1:1/api/search"]);
            window().input
        };
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(
                &serde_json::to_vec(&json!({
                    "evidence_id":"reading", "environment":"production", "tool":"reader",
                    "tool_version":"1", "input":input, "context":{}
                }))
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("{provider} blocked while opening a FIFO inventory");
            }
            thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("regular"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!marker.exists());
    }
}
