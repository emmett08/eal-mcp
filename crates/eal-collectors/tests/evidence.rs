//! Public command boundaries for support discovery and collection.
use eal_collector_contracts::execution::{
    EvaluationCriterion, EvidenceExecutionPlan, PLAN_SCHEMA, RequiredCoverage,
    RequirementEvaluation, SchedulingPolicy, canonical_digest,
};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
};
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("eal-support-command-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn command(arguments: &[&str], input: Option<&Value>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_eal-collector"))
        .args(arguments)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(input).unwrap())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}
fn pending_plan() -> EvidenceExecutionPlan {
    EvidenceExecutionPlan {
        schema: PLAN_SCHEMA.into(),
        acquisitions: vec![],
        derivations: vec![],
        evaluations: vec![RequirementEvaluation {
            id: "evaluation-review".into(),
            requirement_id: "review".into(),
            inputs: vec![],
            criterion: EvaluationCriterion::Judgement {
                criterion: "independent review".into(),
                scope: None,
            },
            max_age_seconds: None,
            required_coverage: RequiredCoverage::Unspecified,
            applicability: vec![],
            challenges: vec![],
        }],
        scheduling: SchedulingPolicy::default(),
    }
}
fn request(plan: &EvidenceExecutionPlan) -> Value {
    json!({"evidence_id":"review","environment":"engarg_case","tool":"support_graph","tool_version":"1","input":{"schema":"engarg.support-acquisition/1","case_digest":"a".repeat(64),"execution_plan_sha256":canonical_digest(plan).unwrap(),"requirement_id":"review"},"context":{"engarg_export_digest":"a".repeat(64)}})
}
#[test]
fn configured_discovery_emits_one_versioned_report_and_rejects_unknown_flags() {
    let temp = Directory::new();
    let config = temp.0.join("config.json");
    std::fs::write(
        &config,
        serde_json::to_vec(&json!({"schema":"eal.evidence-config/1","connections":[]})).unwrap(),
    )
    .unwrap();
    let output = command(
        &["evidence-discover", "--config", config.to_str().unwrap()],
        None,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema"], "eal.evidence-discovery/1");
    assert_eq!(report["metrics"]["requests"], 0);
    assert_eq!(report["admitted_capabilities"], json!([]));
    let rejected = command(
        &[
            "evidence-discover",
            "--config",
            config.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
        ],
        None,
    );
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
}
#[test]
fn support_request_plan_pins_are_checked_before_config_or_store_io() {
    let temp = Directory::new();
    let plan = pending_plan();
    let path = temp.0.join("plan.json");
    std::fs::write(&path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let mut input = request(&plan);
    input["input"]["execution_plan_sha256"] = json!("b".repeat(64));
    let absent = temp.0.join("absent-config.json");
    let store = temp.0.join("store");
    let output = command(
        &[
            "support-graph",
            "--plan",
            path.to_str().unwrap(),
            "--config",
            absent.to_str().unwrap(),
            "--store",
            store.to_str().unwrap(),
        ],
        Some(&input),
    );
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("digest differs"));
    assert!(!store.exists());
}
#[test]
fn pending_judgement_is_unavailable_without_fabricated_observation_envelope() {
    let temp = Directory::new();
    let plan = pending_plan();
    let path = temp.0.join("plan.json");
    std::fs::write(&path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let config = temp.0.join("config.json");
    std::fs::write(
        &config,
        serde_json::to_vec(&json!({"schema":"eal.evidence-config/1","connections":[]})).unwrap(),
    )
    .unwrap();
    let store = temp.0.join("store");
    let output = command(
        &[
            "support-graph",
            "--plan",
            path.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
            "--store",
            store.to_str().unwrap(),
        ],
        Some(&request(&plan)),
    );
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("independent judgement remains required")
    );
}

#[test]
fn explicit_source_reconciliation_validates_identity_before_opening_store() {
    let temp = Directory::new();
    let store = temp.0.join("store");
    let rejected = command(
        &[
            "evidence-reconcile",
            "--store",
            store.to_str().unwrap(),
            "--source-id",
            "invalid",
        ],
        None,
    );
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
    assert!(!store.exists());
    let digest = "f".repeat(64);
    let output = command(
        &[
            "evidence-reconcile",
            "--store",
            store.to_str().unwrap(),
            "--source-id",
            &digest,
        ],
        None,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["source_id"], digest);
    assert_eq!(value["reconciled"], true);
}

#[test]
fn configured_only_discovery_defers_approved_catalogue_requests_without_changing_config() {
    use std::{
        io::Read,
        net::TcpListener,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::Duration,
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/catalogue.json", listener.local_addr().unwrap());
    let calls = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let server_calls = calls.clone();
    let server_stop = stop.clone();
    let server = std::thread::spawn(move || {
        while !server_stop.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut bytes = [0_u8; 8192];
                    let read = stream.read(&mut bytes).unwrap();
                    assert!(
                        std::str::from_utf8(&bytes[..read])
                            .unwrap()
                            .starts_with("GET /catalogue.json ")
                    );
                    server_calls.fetch_add(1, Ordering::SeqCst);
                    let body=serde_json::to_vec(&json!({"schema":eal_collector_contracts::capability::CAPABILITY_CATALOGUE_SCHEMA,"capabilities":[]})).unwrap();
                    write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).unwrap();
                    stream.write_all(&body).unwrap();
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(2))
                }
                Err(error) => panic!("catalogue listener failed: {error}"),
            }
        }
    });
    let temp = Directory::new();
    let config = temp.0.join("config.json");
    let schema = json!({"type":"object"});
    let candidate = eal_collector_contracts::capability::CapabilityDescriptor {
        id: "configured-candidate".into(),
        version: "1".into(),
        description: "Retained configuration candidate".into(),
        provider: eal_collector_contracts::capability::ProviderIdentity {
            id: "operator".into(),
            version: "1".into(),
            transport: eal_collector_contracts::capability::TransportKind::InProcess,
            connection: None,
            tool: None,
            authentication_scope: "reader".into(),
            configuration_sha256: "0".repeat(64),
        },
        input_schema: schema.clone(),
        input_schema_sha256: eal_collector_contracts::capability::schema_identity(&schema).unwrap(),
        output_schema: schema.clone(),
        output_schema_sha256: eal_collector_contracts::capability::schema_identity(&schema)
            .unwrap(),
        semantics: None,
        configuration_requirements: vec![],
        limits: eal_collector_contracts::capability::CapabilityLimits {
            timeout_seconds: 1,
            output_bytes: 1024,
            concurrency: 1,
            requests_per_minute: None,
        },
        costs: eal_collector_contracts::capability::CapabilityCosts::unknown("unmeasured"),
    };
    let bytes=serde_json::to_vec(&json!({"schema":"eal.evidence-config/1","connections":[],"local_catalogues":[{"schema":eal_collector_contracts::capability::CAPABILITY_CATALOGUE_SCHEMA,"capabilities":[candidate]}],"approved_catalogues":[{"url":url}]})).unwrap();
    std::fs::write(&config, &bytes).unwrap();
    let selected = command(
        &[
            "evidence-discover",
            "--config",
            config.to_str().unwrap(),
            "--configured-only",
        ],
        None,
    );
    assert!(
        selected.status.success(),
        "{}",
        String::from_utf8_lossy(&selected.stderr)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let report: Value = serde_json::from_slice(&selected.stdout).unwrap();
    assert_eq!(report["metrics"]["requests"], 0);
    assert_eq!(std::fs::read(&config).unwrap(), bytes);
    assert_eq!(report["limits"]["requests"], 64);
    assert_eq!(
        report["catalogue"]["capabilities"][0]["id"],
        "configured-candidate"
    );
    for (name, value, field) in [
        ("--request-budget", "0", "requests"),
        ("--byte-budget", "100", "bytes"),
        ("--timeout-secs", "0", "timeout_seconds"),
        ("--pages-per-connection", "0", "pages_per_connection"),
    ] {
        let output = command(
            &[
                "evidence-discover",
                "--config",
                config.to_str().unwrap(),
                name,
                value,
            ],
            None,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let reduced: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(reduced["incomplete"], true);
        assert_eq!(reduced["metrics"]["requests"], 0);
        assert_eq!(reduced["budget_usage"]["requests_upper_bound"], 0);
        assert_eq!(reduced["limits"][field], value.parse::<u64>().unwrap());
        assert_eq!(
            reduced["catalogue"]["capabilities"][0]["id"],
            "configured-candidate"
        );
        assert_eq!(reduced["admitted_capabilities"], json!([]));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
    for (name, value) in [
        ("--request-budget", "65"),
        ("--byte-budget", "4194305"),
        ("--timeout-secs", "61"),
        ("--pages-per-connection", "17"),
        ("--request-budget", "-1"),
        ("--request-budget", "1.5"),
    ] {
        let output = command(
            &[
                "evidence-discover",
                "--config",
                config.to_str().unwrap(),
                name,
                value,
            ],
            None,
        );
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
    let duplicate_budget = command(
        &[
            "evidence-discover",
            "--config",
            config.to_str().unwrap(),
            "--request-budget",
            "1",
            "--request-budget",
            "1",
        ],
        None,
    );
    assert!(!duplicate_budget.status.success());
    assert!(duplicate_budget.stdout.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let full = command(
        &[
            "evidence-discover",
            "--config",
            config.to_str().unwrap(),
            "--request-budget",
            "1",
            "--byte-budget",
            "1024",
            "--timeout-secs",
            "1",
            "--pages-per-connection",
            "1",
        ],
        None,
    );
    stop.store(true, Ordering::SeqCst);
    server.join().unwrap();
    assert!(
        full.status.success(),
        "{}",
        String::from_utf8_lossy(&full.stderr)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let report: Value = serde_json::from_slice(&full.stdout).unwrap();
    assert_eq!(report["metrics"]["requests"], 1);
    assert_eq!(report["limits"]["requests"], 1);
    assert_eq!(report["limits"]["bytes"], 1024);
    assert_eq!(report["limits"]["timeout_seconds"], 1);
    assert_eq!(report["limits"]["pages_per_connection"], 1);
    assert_eq!(report["budget_usage"]["requests_upper_bound"], 1);
    assert_eq!(std::fs::read(&config).unwrap(), bytes);
    let duplicate = command(
        &[
            "evidence-discover",
            "--config",
            config.to_str().unwrap(),
            "--configured-only",
            "--configured-only",
        ],
        None,
    );
    assert!(!duplicate.status.success());
    assert!(duplicate.stdout.is_empty());
}

#[test]
fn prepare_native_provider_normalizes_hash_without_contacting_or_qualifying_the_source() {
    use eal_collector_contracts::capability::*;
    use eal_evidence::{ObservationTime, ResponseFormat, ToolMapping, providers::*};
    let temp = Directory::new();
    let input_schema = json!({"type":"object"});
    let output_schema = json!({"type":"boolean"});
    let mut provider = NativeProviderConfig {
        capability: CapabilityDescriptor {
            id: "prepared".into(),
            version: "1".into(),
            description: "Structurally prepared only".into(),
            provider: ProviderIdentity {
                id: "unavailable-http".into(),
                version: "1".into(),
                transport: TransportKind::Http,
                connection: None,
                tool: None,
                authentication_scope: "operator-reader".into(),
                configuration_sha256: "0".repeat(64),
            },
            input_schema_sha256: schema_identity(&input_schema).unwrap(),
            input_schema,
            output_schema_sha256: schema_identity(&output_schema).unwrap(),
            output_schema,
            semantics: None,
            configuration_requirements: vec![],
            limits: CapabilityLimits {
                timeout_seconds: 1,
                output_bytes: 1024,
                concurrency: 1,
                requests_per_minute: None,
            },
            costs: CapabilityCosts::unknown("not measured"),
        },
        transport: NativeTransport::HttpGet {
            url: "http://127.0.0.1:9/no-source-call".into(),
            query_bindings: Default::default(),
            token_env: Some("MISSING_PREPARE_NATIVE_TOKEN".into()),
        },
        mapping: ToolMapping {
            tool: "prepared".into(),
            raw_output_schema_sha256: None,
            response_format: ResponseFormat::Structured,
            value_pointer: "".into(),
            observation_time: ObservationTime::CollectionTime,
            scope_pointers: Default::default(),
        },
        pinned_files: vec![],
        credential_env: vec!["MISSING_PREPARE_NATIVE_TOKEN".into()],
    };
    let path = temp.0.join("provider.json");
    std::fs::write(&path, serde_json::to_vec(&provider).unwrap()).unwrap();
    let original = std::fs::read(&path).unwrap();
    let output = command(
        &["prepare-native-provider", "--input", path.to_str().unwrap()],
        None,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    provider.capability.provider.configuration_sha256 = provider.configuration_identity().unwrap();
    let returned: NativeProviderConfig = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        serde_json::to_value(&returned).unwrap(),
        serde_json::to_value(&provider).unwrap()
    );
    assert!(returned.capability.semantics.is_none());
    assert_eq!(std::fs::read(path).unwrap(), original);
}

#[cfg(unix)]
#[test]
fn public_support_command_bounds_contended_key_initialisation() {
    use std::time::{Duration, Instant};
    let temp = Directory::new();
    let plan = pending_plan();
    let path = temp.0.join("plan.json");
    std::fs::write(&path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let config = temp.0.join("config.json");
    std::fs::write(
        &config,
        br#"{"schema":"eal.evidence-config/1","connections":[]}"#,
    )
    .unwrap();
    let store = temp.0.join("store");
    eal_evidence::storage::SourceStore::open(&store, 10, 1_048_576).unwrap();
    let held = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(store.join("store.key"))
        .unwrap();
    rustix::fs::flock(&held, rustix::fs::FlockOperation::LockExclusive).unwrap();
    let holder = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        drop(held);
    });
    let started = Instant::now();
    let output = command(
        &[
            "support-graph",
            "--plan",
            path.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
            "--store",
            store.to_str().unwrap(),
            "--timeout-ms",
            "50",
        ],
        Some(&request(&plan)),
    );
    let elapsed = started.elapsed();
    eprintln!("public support command with 50 ms budget returned after {elapsed:?}");
    assert!(
        elapsed < Duration::from_millis(750),
        "includes process startup and scheduler tolerance"
    );
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("deadline exhausted"));
    holder.join().unwrap();
    // A timed-out opener leaves the existing key readable and does not acquire a source.
    eal_evidence::storage::SourceStore::open(&store, 10, 1_048_576).unwrap();
    assert_eq!(
        std::fs::read_dir(store.join("observations"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn support_timeout_requires_a_positive_bounded_integer_and_rejects_duplicates() {
    for value in ["0", "86400001", "-1", "1.5", "NaN"] {
        let output = command(
            &[
                "support-graph",
                "--plan",
                "absent",
                "--config",
                "absent",
                "--store",
                "absent",
                "--timeout-ms",
                value,
            ],
            Some(&json!({})),
        );
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("support timeout requires"));
    }
    let output = command(
        &[
            "support-graph",
            "--plan",
            "absent",
            "--config",
            "absent",
            "--store",
            "absent",
            "--timeout-ms",
            "1",
            "--timeout-ms",
            "1",
        ],
        Some(&json!({})),
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate"));
}

#[cfg(unix)]
#[test]
fn support_command_rejects_fifo_plan_and_configuration_without_waiting_for_a_writer() {
    use std::time::{Duration, Instant};
    for fifo_option in ["plan", "config"] {
        let temp = Directory::new();
        let plan = pending_plan();
        let plan_path = temp.0.join("plan");
        let config_path = temp.0.join("config");
        let store = temp.0.join("store");
        if fifo_option == "plan" {
            rustix::fs::mkfifoat(
                rustix::fs::CWD,
                &plan_path,
                rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
            )
            .unwrap();
            std::fs::write(
                &config_path,
                br#"{"schema":"eal.evidence-config/1","connections":[]}"#,
            )
            .unwrap();
        } else {
            std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
            rustix::fs::mkfifoat(
                rustix::fs::CWD,
                &config_path,
                rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
            )
            .unwrap();
        }
        let mut child = Command::new(env!("CARGO_BIN_EXE_eal-collector"))
            .args([
                "support-graph",
                "--plan",
                plan_path.to_str().unwrap(),
                "--config",
                config_path.to_str().unwrap(),
                "--store",
                store.to_str().unwrap(),
                "--timeout-ms",
                "50",
            ])
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(&request(&plan)).unwrap())
            .unwrap();
        let started = Instant::now();
        while child.try_wait().unwrap().is_none() && started.elapsed() < Duration::from_secs(1) {
            std::thread::sleep(Duration::from_millis(5));
        }
        if child.try_wait().unwrap().is_none() {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("support command blocked opening FIFO {fifo_option}");
        }
        let output = child.wait_with_output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(
            diagnostic.contains("regular file") || diagnostic.contains("deadline exhausted"),
            "{diagnostic}"
        );
        assert!(!store.exists());
    }
}
