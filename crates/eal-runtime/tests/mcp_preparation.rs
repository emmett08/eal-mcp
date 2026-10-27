//! Preparation pins live interfaces without treating discovery as evidence.
#![cfg(unix)]
use eal_core::{ExecutionLimits, methods::default_registry};
use eal_runtime::{
    process::Supervisor,
    scheduler::CollectionScheduler,
    service::{ReasoningService, ServiceConfig},
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, time::Duration};

const SOURCE: &str = r#"language "EAL/4"
environment bench {
 require true
}
tool read {
 version "1"
}
evidence observed {
 tool read
 kind observation
 environment bench
 max_age 86400000
 input {"case":"value"}
 require passed == false
}
reasoning measurement {
 method "structured/1"
 rationale "The retained negative is a measurement."
}
claim result {
 statement "The fixture reported a negative."
 environment bench
}
argument measured = [evidence observed] via measurement => result
"#;
const TOOLS: &str = "[tools.read]\nkind='mcp_stdio'\nversion='1'\nargv=['python3','server.py']\ninherit_env=['PATH']\nmax_output_bytes=4096\n[tools.read.direct]\ntool='measurement'\nvalue_pointer='/value'\nobserved_at_pointer='/observed_at'\n";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-prepare-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("tools.toml"), TOOLS).unwrap();
        fs::write(path.join("server.py"),include_str!("fixtures/direct_mcp.py").replace(
   "    elif method == \"tools/call\":", "    elif method == \"tools/call\":\n        open('invocations','a').write('called\\n')")).unwrap();
        Self(path)
    }
    fn service(&self) -> ReasoningService {
        ReasoningService::new(ServiceConfig {
            workspace: self.0.clone(),
            registry_path: Some(self.0.join("tools.toml")),
            database_path: None,
            method_registry: default_registry().clone(),
            method_executor: None,
            scheduler: CollectionScheduler::default(),
            supervisor: Supervisor::new(companion::path()),
            limits: ExecutionLimits::default(),
            lease_timeout: Duration::from_secs(5),
        })
        .unwrap()
    }
    fn pin(&self, receipt: &Value) {
        let mut document = eal_collector_contracts::RegistryDocument::from_toml(TOOLS).unwrap();
        let direct = document
            .tools
            .get_mut("read")
            .unwrap()
            .direct
            .as_mut()
            .unwrap();
        let pins = &receipt["tools"]["read"]["pins"];
        direct.input_schema_sha256 =
            serde_json::from_value(pins["input_schema_sha256"].clone()).unwrap();
        direct.output_schema_sha256 =
            serde_json::from_value(pins["output_schema_sha256"].clone()).unwrap();
        direct.server_identity = serde_json::from_value(pins["server_identity"].clone()).unwrap();
        fs::write(
            self.0.join("tools.toml"),
            toml::to_string(&document).unwrap(),
        )
        .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn prepare_discovers_only_and_execution_retains_the_actual_interface() {
    let fixture = Fixture::new();
    let service = fixture.service();
    let receipt = service.prepare(SOURCE, &json!({}), Some("result")).unwrap();
    assert_eq!(receipt["operation_calls"], 0);
    assert_eq!(receipt["observations_created"], 0);
    assert!(!fixture.0.join("invocations").exists());
    let db = rusqlite::Connection::open(service.store().path()).unwrap();
    let observations: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM records WHERE kind IN ('observation','collection','assessment')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(observations, 0);
    let identity = &receipt["tools"]["read"]["interface"];
    assert_eq!(
        identity["server"],
        json!({"name":"synthetic-direct","version":"1","protocol":"2025-11-25"})
    );
    assert_eq!(identity["tool"], "measurement");
    assert_eq!(identity["input_schema_sha256"].as_str().unwrap().len(), 64);
    // An unpinned call records the same observed identity; preparation is optional.
    let execution = service
        .execute(
            SOURCE,
            &json!({}),
            Some("result"),
            Some("2026-01-02T03:04:06Z"),
        )
        .unwrap();
    assert_eq!(
        execution["collection"]["records"]["observed"]["mcp_interface"],
        *identity
    );
    assert_eq!(
        execution["assessment"]["claims"]["result"]["status"],
        "supported"
    );
    fixture.pin(&receipt);
    let execution = fixture
        .service()
        .execute(SOURCE, &json!({}), None, Some("2026-01-02T03:04:06Z"))
        .unwrap();
    assert_eq!(
        execution["collection"]["records"]["observed"]["mcp_interface"],
        *identity
    );
}
#[test]
fn prepared_schema_and_server_drift_fail_before_the_operation_is_called() {
    for schema_drift in [false, true] {
        let fixture = Fixture::new();
        let receipt = fixture.service().prepare(SOURCE, &json!({}), None).unwrap();
        fixture.pin(&receipt);
        let path = fixture.0.join("server.py");
        let source = fs::read_to_string(&path).unwrap();
        let changed = if schema_drift {
            source.replace("\"type\": \"string\"", "\"enum\": [\"value\"]")
        } else {
            source.replace("\"version\": \"1\"", "\"version\": \"2\"")
        };
        assert_ne!(source, changed);
        fs::write(path, changed).unwrap();
        let service = fixture.service();
        assert!(service.prepare(SOURCE, &json!({}), None).is_err());
        let execution = service.execute(SOURCE, &json!({}), None, None).unwrap();
        assert_eq!(
            execution["collection"]["records"]["observed"]["status"],
            "error"
        );
        assert!(!fixture.0.join("invocations").exists());
    }
}
#[test]
fn completed_mcp_failures_retain_interface_identity() {
    for case in ["error", "large", "text", "schema"] {
        let fixture = Fixture::new();
        if case == "schema" {
            let path = fixture.0.join("server.py");
            let text = fs::read_to_string(&path).unwrap().replace(
                "\"description\": \"Synthetic fixture\",",
                "\"description\": \"Synthetic fixture\", \"outputSchema\": {\"type\":\"object\",\"required\":[\"missing\"]},",
            );
            fs::write(path, text).unwrap();
        }
        let execution = fixture
            .service()
            .execute(
                &SOURCE.replace("\"value\"", &format!("\"{case}\"")),
                &json!({}),
                None,
                None,
            )
            .unwrap();
        let record = &execution["collection"]["records"]["observed"];
        assert_eq!(record["status"], "error", "{case}: {record}");
        assert_eq!(
            record["mcp_interface"]["server"]["name"], "synthetic-direct",
            "{case}: {record}"
        );
        assert_eq!(record["mcp_interface"]["tool"], "measurement");
        assert!(record.get("value").is_none_or(Value::is_null));
    }
}

#[test]
fn preparation_selects_only_the_claim_closure_and_admits_all_bindings_before_discovery() {
    let fixture = Fixture::new();
    let path = fixture.0.join("server.py");
    let text = fs::read_to_string(&path).unwrap().replace(
        "    if method == \"initialize\":",
        "    if method == \"initialize\":\n        open('initialized','a').write('started\\n')",
    );
    fs::write(path, text).unwrap();
    let source = format!(
        "{SOURCE}\ntool unconfigured {{\n version \"1\"\n}}\nevidence unused {{\n tool unconfigured\n kind observation\n environment bench\n max_age 60\n require true\n}}\n"
    );
    let service = fixture.service();
    let receipt = service
        .prepare(&source, &json!({}), Some("result"))
        .unwrap();
    assert_eq!(
        receipt["tools"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["read"]
    );
    assert!(fixture.0.join("initialized").exists());
    fs::remove_file(fixture.0.join("initialized")).unwrap();
    assert!(
        service
            .prepare(&source, &json!({}), None)
            .unwrap_err()
            .contains("operator registry")
    );
    assert!(!fixture.0.join("initialized").exists());
    assert!(!fixture.0.join("invocations").exists());
}

#[test]
fn all_selected_bindings_are_admitted_before_any_source_or_observation() {
    use eal_collector_contracts::{PinnedFile, RegistryDocument, ToolKind};
    use sha2::{Digest, Sha256};
    for issue in [
        "version",
        "credential",
        "implementation",
        "input",
        "authentication",
    ] {
        for operation in ["prepare", "collect", "execute"] {
            let fixture = Fixture::new();
            let path = fixture.0.join("server.py");
            fs::write(&path, fs::read_to_string(&path).unwrap().replace(
                "    if method == \"initialize\":",
                "    if method == \"initialize\":\n        open('initialized','a').write('started\\n')",
            )).unwrap();
            let mut manifest = RegistryDocument::from_toml(TOOLS).unwrap();
            let mut second = manifest.tools["read"].clone();
            match issue {
                "version" => second.version = "2".into(),
                "implementation" => {
                    second.pinned_files = vec![PinnedFile {
                        path: "server.py".into(),
                        sha256: "0".repeat(64),
                    }]
                }
                "input" => {
                    let schema = br#"{"type":"object","properties":{"case":{"type":"string"}},"required":["case"]}"#;
                    fs::write(fixture.0.join("input.json"), schema).unwrap();
                    second.direct.as_mut().unwrap().input_schema = Some(PinnedFile {
                        path: "input.json".into(),
                        sha256: format!("{:x}", Sha256::digest(schema)),
                    });
                }
                _ => {
                    second.kind = ToolKind::McpHttp;
                    second.argv.clear();
                    second.inherit_env = None;
                    let direct = second.direct.as_mut().unwrap();
                    direct.url = Some(if issue == "credential" {
                        "http://127.0.0.1:1/mcp".into()
                    } else {
                        "https://example.invalid/mcp".into()
                    });
                    if issue == "credential" {
                        direct.token_env =
                            Some(format!("UNAVAILABLE_{}", uuid::Uuid::new_v4().simple()));
                    }
                }
            }
            manifest.tools.insert("second".into(), second);
            fs::write(
                fixture.0.join("tools.toml"),
                toml::to_string(&manifest).unwrap(),
            )
            .unwrap();
            let source = format!(
                "{SOURCE}\ntool second {{\n version \"1\"\n}}\nevidence second_reading {{\n tool second\n kind observation\n environment bench\n max_age 60\n input {{\"case\":42}}\n require passed == false\n}}\n"
            );
            let service = fixture.service();
            let result = match operation {
                "prepare" => service.prepare(&source, &json!({}), None),
                "collect" => service.collect(&source, &json!({}), None),
                _ => service.execute(&source, &json!({}), None, None),
            };
            let error = result.unwrap_err();
            let expected = match issue {
                "implementation" => "identity",
                "input" => "schema",
                other => other,
            };
            assert!(error.contains(expected), "{operation}/{issue}: {error}");
            assert!(
                !fixture.0.join("initialized").exists(),
                "{operation}/{issue}"
            );
            assert!(
                !fixture.0.join("invocations").exists(),
                "{operation}/{issue}"
            );
            assert!(
                service
                    .store()
                    .list(Some("observation"), 10)
                    .unwrap()
                    .is_empty()
            );
        }
    }
}

mod companion {
    pub fn path() -> std::path::PathBuf {
        let executable = std::env::current_exe().unwrap();
        let binaries = executable.parent().unwrap().parent().unwrap();
        binaries.join("eal-process-supervisor")
    }
}
