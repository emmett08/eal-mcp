//! Independent HTTP fixtures exercise actual provider requests and negative observations.
use eal_collectors::{
    Config, Request,
    api::{ApiConfig, ApiProvider},
    config::{CollectorLimits, ContractVersion, ProviderConfig},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

struct Fixture {
    directory: PathBuf,
    path: PathBuf,
    digest: String,
    provider: ApiProvider,
    measurement: String,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn write(path: &Path, value: &Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap();
    std::fs::write(path, &bytes).unwrap();
    hash(&bytes)
}
fn fixture(source: Value, origin: &str) -> Fixture {
    let directory = std::env::temp_dir().join(format!("eal-api-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("source.json");
    let kind = source["kind"].as_str().unwrap();
    let (provider, measurement) = match kind {
        "prometheus" => (ApiProvider::Prometheus, "prometheus_range"),
        "loki" => (ApiProvider::Loki, "loki_search"),
        "openapi" => (ApiProvider::Openapi, "openapi_operation"),
        other => (ApiProvider::Datadog, other),
    };
    let measurement = measurement.to_owned();
    let digest = write(
        &path,
        &json!({"schema":"eal.api-source/1","origin":origin,"source":source}),
    );
    Fixture {
        directory,
        path,
        digest,
        provider,
        measurement,
    }
}
impl Fixture {
    fn selected(&self) -> ApiConfig {
        ApiConfig {
            provider: self.provider,
            measurement: self.measurement.clone(),
            path: self.path.clone(),
            sha256: self.digest.clone(),
        }
    }
    fn config(&self) -> Config {
        Config::new(
            ProviderConfig::Api(self.selected()),
            CollectorLimits::default(),
            ContractVersion::V2,
        )
        .unwrap()
    }
    fn update(&mut self, change: impl FnOnce(&mut Value)) {
        let mut value: Value = serde_json::from_slice(&std::fs::read(&self.path).unwrap()).unwrap();
        change(&mut value);
        self.digest = write(&self.path, &value);
    }
}
fn request(input: Value) -> Request {
    Request::parse(&serde_json::to_vec(&json!({"evidence_id":"e","environment":"production","tool":"measurement","tool_version":"2","input":input,"context":{"stage":"production"}})).unwrap()).unwrap()
}
fn input() -> Value {
    json!({"query":"service=api","start_unix":100,"end_unix":200})
}
fn server(status: u16, body: String) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buf = [0u8; 4096];
        let mut expected = None;
        loop {
            let n = socket.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            bytes.extend_from_slice(&buf[..n]);
            if expected.is_none()
                && let Some(index) = bytes.windows(4).position(|w| w == b"\r\n\r\n")
            {
                let header = String::from_utf8_lossy(&bytes[..index]);
                let size = header
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|n| n.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                expected = Some(index + 4 + size);
            }
            if expected.is_some_and(|n| bytes.len() >= n) {
                break;
            }
        }
        write!(socket,"HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        String::from_utf8(bytes).unwrap()
    });
    (format!("http://{address}"), handle)
}
fn body(request: &str) -> Value {
    serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap()
}

#[tokio::test]
async fn telemetry_preserves_zero_null_warnings_and_empty_searches() {
    let cases = [
        (
            json!({"kind":"prometheus","query":"service=api","step_seconds":10}),
            json!({"status":"success","data":{"resultType":"matrix","result":[{"metric":{"service":"api"},"values":[[100,"0"],[110,"NaN"]]}]},"warnings":["partial source"],"infos":["annotation"]}),
        ),
        (
            json!({"kind":"loki","query":"service=api","limit":10}),
            json!({"status":"success","data":{"resultType":"streams","result":[]}}),
        ),
        (
            json!({"kind":"datadog_metrics","query":"service=api","interval_seconds":10}),
            json!({"data":{"type":"timeseries_response","attributes":{"series":[{"query_index":0}],"times":[100000,110000],"values":[[0,null]],"interval":10000}}}),
        ),
        (
            json!({"kind":"datadog_logs","query":"service=api","limit":10}),
            json!({"data":[],"meta":{"page":{"after":"next-page"},"warnings":["partial"]}}),
        ),
        (
            json!({"kind":"datadog_spans","query":"service=api","limit":10}),
            json!({"data":[{"id":"span-1","attributes":{"service":"api"}}],"meta":{"page":{"after":"next-span"}}}),
        ),
    ];
    for (source, response) in cases {
        let (origin, handle) = server(200, response.to_string());
        let fixture = fixture(source.clone(), &origin);
        let result = eal_collectors::collect(&fixture.config(), &request(input()))
            .await
            .unwrap();
        assert_eq!(result["value"]["observation"], response);
        assert_eq!(result["value"]["measurement_only"], true);
        assert_eq!(result["value"]["config_sha256"], fixture.digest);
        assert_eq!(result["request"]["input"], input());
        let wire = handle.join().unwrap();
        match source["kind"].as_str().unwrap() {
            "prometheus" => {
                assert!(wire.starts_with("GET /api/v1/query_range?"));
                assert!(wire.contains("step=10"));
            }
            "loki" => {
                assert!(wire.contains("start=100000000000"));
                assert_eq!(result["value"]["matched"], false);
            }
            "datadog_metrics" => {
                assert!(wire.starts_with("POST /api/v2/query/timeseries"));
                assert_eq!(body(&wire)["data"]["attributes"]["from"], 100000);
            }
            "datadog_logs" => {
                assert_eq!(body(&wire)["filter"]["query"], "service=api");
                assert_eq!(result["value"]["next_cursor"], "next-page");
            }
            "datadog_spans" => {
                assert_eq!(body(&wire)["data"]["type"], "search_request");
                assert_eq!(
                    body(&wire)["data"]["attributes"]["filter"]["query"],
                    "service=api"
                );
            }
            _ => unreachable!(),
        }
        if source["kind"] != "prometheus" && source["kind"] != "datadog_metrics" {
            assert_eq!(result["value"]["search_exhaustive"], false);
            assert_eq!(result["value"]["negative_inference"], "unavailable");
        }
    }
}

#[tokio::test]
async fn datadog_monitor_and_slo_keep_measured_adverse_states() {
    for (source, input, response, path) in [
        (
            json!({"kind":"datadog_monitor","id":17}),
            json!({"id":17}),
            json!({"id":17,"overall_state":"Alert","query":"avg:errors"}),
            "/api/v1/monitor/17",
        ),
        (
            json!({"kind":"datadog_slo","id":"abc123"}),
            json!({"id":"abc123","start_unix":100,"end_unix":200}),
            json!({"data":{"overall":{"sli_value":82.1,"error_budget_remaining":-10}},"errors":[]}),
            "/api/v1/slo/abc123/history?",
        ),
    ] {
        let (origin, handle) = server(200, response.to_string());
        let fixture = fixture(source, &origin);
        let result = eal_collectors::collect(&fixture.config(), &request(input))
            .await
            .unwrap();
        assert_eq!(result["value"]["observation"], response);
        assert!(handle.join().unwrap().contains(path));
    }
}

#[tokio::test]
async fn failure_and_malformed_payloads_never_become_measured_negatives() {
    for (status, response) in [
        (403, "{\"error\":\"private diagnostics\"}"),
        (429, "{}"),
        (302, "{}"),
        (200, "{\"status\":\"success\",\"data\":{}}"),
        (200, "{\"data\":{},\"data\":{}}"),
        (200, "not-json"),
    ] {
        let (origin, handle) = server(status, response.into());
        let fixture = fixture(
            json!({"kind":"prometheus","query":"service=api","step_seconds":10}),
            &origin,
        );
        let error = eal_collectors::collect(&fixture.config(), &request(input()))
            .await
            .unwrap_err();
        assert!(!error.contains("private diagnostics"));
        handle.join().unwrap();
    }
}

#[tokio::test]
async fn selectors_windows_and_pins_fail_before_network_access() {
    let fixture = fixture(
        json!({"kind":"prometheus","query":"service=api","step_seconds":10}),
        "http://127.0.0.1:9",
    );
    for input in [
        json!({"query":"other","start_unix":100,"end_unix":200}),
        json!({"query":"service=api","start_unix":200,"end_unix":100}),
        json!({"query":"service=api","start_unix":1,"end_unix":86402}),
        json!({"query":"service=api","start_unix":100,"end_unix":200,"origin":"https://other"}),
    ] {
        assert!(
            eal_collectors::collect(&fixture.config(), &request(input))
                .await
                .is_err()
        );
    }
    let mut selected = fixture.selected();
    selected.sha256 = "a".repeat(64);
    assert!(eal_collectors::api::describe(&selected).is_err());
}

fn cli(fixture: &Fixture, input: Value, context: Value, secret: &str) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eal-collector"));
    command.args([
        fixture.provider.name(),
        "--contract-version",
        "2",
        "--measurement",
        &fixture.measurement,
        "--config",
        fixture.path.to_str().unwrap(),
        "--config-sha256",
        &fixture.digest,
    ]);
    command
        .env_clear()
        .env("EAL_FIXTURE_TOKEN", secret)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&json!({"evidence_id":"e","environment":"prod","tool":"m","tool_version":"2","input":input,"context":context})).unwrap()).unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn configured_credentials_and_encoded_request_values_are_inspected_without_disclosure() {
    let secret = "fixture-credential-794028";
    for echo_context in [false, true] {
        let observation = if echo_context {
            json!({"status":"success","data":{"resultType":"streams","result":[]}})
        } else {
            json!({"status":"success","data":{"resultType":"streams","result":[]},"extra":format!("{{\"credential\":\"{secret}\"}}")})
        };
        let (origin, handle) = server(200, observation.to_string());
        let mut fixture = fixture(
            json!({"kind":"loki","query":"service=api","limit":10}),
            &origin,
        );
        fixture.update(|v| v["bearer_env"] = json!("EAL_FIXTURE_TOKEN"));
        let context = if echo_context {
            json!({"encoded":format!("{{\"credential\":\"{secret}\"}}")})
        } else {
            json!({})
        };
        let output = cli(&fixture, input(), context, secret);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains(secret));
        assert!(handle.join().unwrap().contains(&format!("Bearer {secret}")));
    }
}

fn openapi_fixture(origin: &str) -> Fixture {
    let mut fixture = fixture(
        json!({"kind":"openapi","specification":"/placeholder","specification_sha256":"a".repeat(64),"operation_id":"readStatus","read_only":true}),
        origin,
    );
    let spec = fixture.directory.join("openapi.json");
    let digest = write(&spec, &specification());
    fixture.update(|v| {
        v["source"]["specification"] = json!(spec);
        v["source"]["specification_sha256"] = json!(digest);
    });
    fixture
}
fn specification() -> Value {
    json!({"openapi":"3.1.0","info":{"title":"fixture","version":"1"},"paths":{"/status/{service}":{"get":{"operationId":"readStatus","parameters":[{"name":"service","in":"path","required":true,"schema":{"type":"string"}},{"name":"detail","in":"query","schema":{"type":"boolean"}}],"responses":{"200":{"content":{"application/json":{"schema":{"$ref":"#/components/schemas/Status"}}}}}}}},"components":{"schemas":{"Status":{"type":"object","required":["healthy"],"properties":{"healthy":{"type":"boolean"}}}}}})
}

#[tokio::test]
async fn openapi_imports_local_schemas_preserves_false_and_encodes_path_inputs() {
    let (origin, handle) = server(200, json!({"healthy":false}).to_string());
    let fixture = openapi_fixture(&origin);
    let catalogue = eal_collectors::api::describe(&fixture.selected()).unwrap();
    let descriptor = &catalogue.collectors[0];
    assert!(descriptor.id.starts_with("openapi."));
    assert!(
        descriptor
            .validate_input(&json!({"path":{"service":"a/b"}}))
            .is_ok()
    );
    let result = eal_collectors::collect(
        &fixture.config(),
        &request(json!({"path":{"service":"a/b"},"query":{"detail":true}})),
    )
    .await
    .unwrap();
    assert_eq!(result["value"]["observation"]["healthy"], false);
    assert_eq!(result["value"]["response_schema_validated"], true);
    let wire = handle.join().unwrap();
    assert!(wire.starts_with("GET /status/a%2Fb?detail=true "), "{wire}");
}

#[tokio::test]
async fn openapi_rejects_response_mismatch_and_undeclared_success_status() {
    for (status, response) in [
        (200, json!({"healthy":"false"})),
        (201, json!({"healthy":true})),
    ] {
        let (origin, handle) = server(status, response.to_string());
        let fixture = openapi_fixture(&origin);
        assert!(
            eal_collectors::collect(
                &fixture.config(),
                &request(json!({"path":{"service":"api"}}))
            )
            .await
            .is_err()
        );
        handle.join().unwrap();
    }
}

#[test]
fn openapi_rejects_remote_references_cycles_unapproved_operations_and_expansion_exhaustion() {
    let mut fixture = openapi_fixture("http://127.0.0.1:9");
    let path = fixture.directory.join("openapi.json");
    for reference in [
        "https://elsewhere.example/schema",
        "#/components/schemas/Status",
    ] {
        let mut spec = specification();
        spec["components"]["schemas"]["Status"] = json!({"$ref":reference});
        let hash = write(&path, &spec);
        fixture.update(|v| v["source"]["specification_sha256"] = json!(hash));
        assert!(eal_collectors::api::describe(&fixture.selected()).is_err());
    }
    let hash = write(&path, &specification());
    fixture.update(|v| {
        v["source"]["specification_sha256"] = json!(hash);
        v["source"]["read_only"] = json!(false);
    });
    assert!(eal_collectors::api::describe(&fixture.selected()).is_err());
    let mut spec = specification();
    let large = json!({"type":"string","description":"x".repeat(20000)});
    spec["components"]["schemas"]["Big"] = large;
    spec["components"]["schemas"]["Status"] = json!({"anyOf":(0..100).map(|_|json!({"$ref":"#/components/schemas/Big"})).collect::<Vec<_>>()});
    let hash = write(&path, &spec);
    fixture.update(|v| {
        v["source"]["read_only"] = json!(true);
        v["source"]["specification_sha256"] = json!(hash);
    });
    assert!(
        eal_collectors::api::describe(&fixture.selected())
            .unwrap_err()
            .contains("allowance")
    );
}

#[tokio::test]
async fn openapi_approved_post_validates_typed_body_and_retains_measured_negative() {
    let (origin, handle) = server(200, json!({"healthy":false}).to_string());
    let mut fixture = openapi_fixture(&origin);
    let mut spec = specification();
    spec["paths"] = json!({"/search":{"post":{"operationId":"readStatus","requestBody":{"required":true,"content":{"application/json":{"schema":{"type":"object","additionalProperties":false,"required":["ids"],"properties":{"ids":{"type":"array","minItems":1,"items":{"type":"integer"}}}}}}},"responses":{"200":{"content":{"application/json":{"schema":{"$ref":"#/components/schemas/Status"}}}}}}}});
    let hash = write(&fixture.directory.join("openapi.json"), &spec);
    fixture.update(|v| v["source"]["specification_sha256"] = json!(hash));
    let descriptor = &eal_collectors::api::describe(&fixture.selected())
        .unwrap()
        .collectors[0];
    assert!(
        descriptor
            .validate_input(&json!({"body":{"ids":["7"]}}))
            .is_err()
    );
    assert_eq!(descriptor.required_parameters().unwrap(), ["body"]);
    let result =
        eal_collectors::collect(&fixture.config(), &request(json!({"body":{"ids":[7,9]}})))
            .await
            .unwrap();
    assert_eq!(result["value"]["observation"]["healthy"], false);
    let wire = handle.join().unwrap();
    assert!(wire.starts_with("POST /search "));
    assert_eq!(body(&wire), json!({"ids":[7,9]}));
}

#[tokio::test]
async fn openapi_schema_annotations_preserve_literal_objects_with_reserved_names() {
    let literal =
        json!({"nullable":true,"$ref":"literal-data","exclusiveMinimum":true,"healthy":false});
    let (origin, handle) = server(200, literal.to_string());
    let mut fixture = openapi_fixture(&origin);
    let spec = json!({"openapi":"3.1.0","paths":{"/lookup":{"post":{"operationId":"readStatus","requestBody":{"required":true,"content":{"application/json":{"schema":{"const":literal}}}},"responses":{"200":{"content":{"application/json":{"schema":{"const":literal}}}}}}}}});
    let hash = write(&fixture.directory.join("openapi.json"), &spec);
    fixture.update(|v| v["source"]["specification_sha256"] = json!(hash));
    let result = eal_collectors::collect(&fixture.config(), &request(json!({"body":literal})))
        .await
        .unwrap();
    assert_eq!(result["value"]["observation"], literal);
    assert_eq!(body(&handle.join().unwrap()), literal);
}

#[tokio::test]
async fn http_response_and_metric_point_allowances_fail_explicitly() {
    let(origin,handle)=server(200,json!({"status":"success","data":{"resultType":"streams","result":[]},"padding":"x".repeat(4096)}).to_string());
    let fixture = fixture(
        json!({"kind":"loki","query":"service=api","limit":10}),
        &origin,
    );
    let config = fixture
        .config()
        .with_limits(CollectorLimits::new(Duration::from_secs(5), 512).unwrap());
    assert!(
        eal_collectors::collect(&config, &request(input()))
            .await
            .unwrap_err()
            .contains("byte allowance")
    );
    handle.join().unwrap();
    let mut fixture = fixture_for_point_limit();
    fixture.update(|v| v["max_points"] = json!(1));
    assert!(
        eal_collectors::collect(&fixture.config(), &request(input()))
            .await
            .unwrap_err()
            .contains("point allowance")
    );
}
fn fixture_for_point_limit() -> Fixture {
    fixture(
        json!({"kind":"prometheus","query":"service=api","step_seconds":1}),
        "http://127.0.0.1:9",
    )
}

#[test]
fn openapi_nullable_uses_the_selected_schema_dialect() {
    let mut fixture = openapi_fixture("http://127.0.0.1:9");
    for (version, admitted) in [("3.0.4", true), ("3.1.0", false)] {
        let mut spec = specification();
        spec["openapi"] = json!(version);
        spec["paths"] = json!({"/lookup":{"post":{"operationId":"readStatus","requestBody":{"required":true,"content":{"application/json":{"schema":{"type":"object","nullable":true}}}},"responses":{"200":{"content":{"application/json":{"schema":{"type":"object"}}}}}}}});
        let hash = write(&fixture.directory.join("openapi.json"), &spec);
        fixture.update(|v| v["source"]["specification_sha256"] = json!(hash));
        let catalogue = eal_collectors::api::describe(&fixture.selected()).unwrap();
        assert_eq!(
            catalogue.collectors[0]
                .validate_input(&json!({"body":null}))
                .is_ok(),
            admitted
        );
    }
}
