//! Revision/attempt correspondence and retained report outcome fixtures.
#![cfg(unix)]
use eal_collectors::{Config, Request};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("eal-delivery-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn write(path: &Path, bytes: &[u8]) -> String {
    std::fs::write(path, bytes).unwrap();
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn request(input: Value) -> Request {
    Request::parse(&serde_json::to_vec(&json!({"evidence_id":"e","environment":"prod","tool":"measurement","tool_version":"2","input":input,"context":{}})).unwrap()).unwrap()
}
fn report(directory: &Directory, format: &str, bytes: &[u8]) -> (Config, Request) {
    let path = directory.0.join("report");
    let artifact_sha256 = write(&path, bytes);
    let manifest = directory.0.join("manifest.json");
    let digest=write(&manifest,&serde_json::to_vec(&json!({"schema":"eal.report-source/1","repository":"owner/repo","revision":"a".repeat(40),"observed_at":"2020-02-03T04:05:06Z","format":format,"artifact":path,"artifact_sha256":artifact_sha256})).unwrap());
    let config = Config::parse(vec![
        "report".into(),
        "--contract-version".into(),
        "2".into(),
        "--manifest".into(),
        manifest.to_str().unwrap().into(),
        "--manifest-sha256".into(),
        digest,
    ])
    .unwrap();
    (
        config,
        request(json!({"revision":"a".repeat(40),"report_sha256":artifact_sha256})),
    )
}

#[tokio::test]
async fn junit_counts_explicit_cases_and_retains_original_age_and_raw_report() {
    let directory = Directory::new();
    let xml=br#"<?xml version="1.0" encoding="UTF-8"?><testsuites tests="999" failures="0"><testsuite name="fixture"><testcase name="passed"/><testcase name="negative"><failure>threshold exceeded &#10;</failure></testcase><testcase name="error"><error/></testcase><testcase name="skip"><skipped/></testcase></testsuite></testsuites>"#;
    let (config, request) = report(&directory, "junit", xml);
    let result = eal_collectors::collect(&config, &request).await.unwrap();
    assert_eq!(result["observed_at"], "2020-02-03T04:05:06Z");
    assert_eq!(result["value"]["summary"]["test_cases"], 4);
    for field in ["passed", "failed", "errors", "skipped"] {
        assert_eq!(result["value"]["summary"][field], 1);
    }
    assert_eq!(result["value"]["report"], std::str::from_utf8(xml).unwrap());
    assert_eq!(
        result["value"]["provenance"],
        "operator_asserted_revision_correspondence"
    );
}
#[tokio::test]
async fn junit_rejects_malformed_ambiguous_and_custom_entity_documents() {
    for xml in [
        "<testsuite><testcase name='x'></testsuite>",
        "<!DOCTYPE testsuite [<!ENTITY x SYSTEM 'file:///private'>]><testsuite/>",
        "<testsuite><testcase name='x'><failure/><skipped/></testcase></testsuite>",
        "<testsuite/><testsuite/>",
        "<testsuite><testcase name='x'><rerunFailure/></testcase></testsuite>",
        "<testsuite><testcase name='x' name='y'/></testsuite>",
        "<testsuite><testcase/></testsuite>",
    ] {
        let directory = Directory::new();
        let (config, request) = report(&directory, "junit", xml.as_bytes());
        assert!(
            eal_collectors::collect(&config, &request).await.is_err(),
            "{xml}"
        );
    }
}
#[tokio::test]
async fn sarif_findings_and_json_benchmark_negatives_are_preserved() {
    for (format, value) in [
        (
            "sarif",
            json!({"version":"2.1.0","runs":[{"tool":{"driver":{"name":"scanner"}},"invocations":[{"executionSuccessful":false}],"results":[{"ruleId":"SEC-1","level":"error","message":{"text":"finding"}}]}]}),
        ),
        (
            "json",
            json!({"within_budget":false,"duration_seconds":11.4,"budget_seconds":10}),
        ),
    ] {
        let directory = Directory::new();
        let (config, request) = report(&directory, format, &serde_json::to_vec(&value).unwrap());
        let result = eal_collectors::collect(&config, &request).await.unwrap();
        assert_eq!(result["value"]["report"], value);
        assert_eq!(result["observed_at"], "2020-02-03T04:05:06Z");
        if format == "sarif" {
            assert_eq!(result["value"]["summary"]["finding_count"], 1);
        }
    }
}
#[tokio::test]
async fn retained_reports_reject_changed_bytes_and_mismatched_revision() {
    let directory = Directory::new();
    let (config, mut request) = report(&directory, "json", b"{\"passed\":false}");
    request.input["revision"] = json!("b".repeat(40));
    assert!(eal_collectors::collect(&config, &request).await.is_err());
    request.input["revision"] = json!("a".repeat(40));
    std::fs::write(directory.0.join("report"), "{\"passed\":true}").unwrap();
    assert!(eal_collectors::collect(&config, &request).await.is_err());
}

fn supervisor() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_eal-collector"))
        .parent()
        .unwrap()
        .join("eal-process-supervisor")
}
fn gh(directory: &Directory, response: &Value) -> PathBuf {
    let data = directory.0.join("response.json");
    write(&data, &serde_json::to_vec(response).unwrap());
    let script = directory.0.join("gh");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\ncat '{}'\n",
            directory.0.join("argv").display(),
            data.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
    script
}
fn delivery_config(directory: &Directory, provider: &str, response: &Value) -> Config {
    assert!(
        supervisor().is_file(),
        "Build native companions before integration tests"
    );
    let script = gh(directory, response);
    let mut args = vec![
        provider.into(),
        "--contract-version".into(),
        "2".into(),
        "--executable".into(),
        script.to_str().unwrap().into(),
        "--supervisor".into(),
        supervisor().to_str().unwrap().into(),
        "--repository".into(),
        "owner/repo".into(),
        "--commit".into(),
        "a".repeat(40),
        "--inherit-env".into(),
        "PATH".into(),
    ];
    if provider == "github_workflow" {
        args.extend([
            "--run-id".into(),
            "17".into(),
            "--attempt".into(),
            "2".into(),
        ]);
    }
    Config::parse(args).unwrap()
}
#[tokio::test]
async fn github_workflow_retains_failed_attempt_and_checks_exact_identity() {
    let directory = Directory::new();
    let mut response = json!({"id":17,"run_attempt":2,"head_sha":"a".repeat(40),"repository":{"full_name":"owner/repo"},"status":"completed","conclusion":"failure"});
    let input = json!({"repository":"owner/repo","commit":"a".repeat(40),"run_id":17,"attempt":2});
    let config = delivery_config(&directory, "github_workflow", &response);
    let result = eal_collectors::collect(&config, &request(input.clone()))
        .await
        .unwrap();
    assert_eq!(result["value"]["workflow"]["conclusion"], "failure");
    let argv = std::fs::read_to_string(directory.0.join("argv")).unwrap();
    assert!(argv.contains("GET\nrepos/owner/repo/actions/runs/17/attempts/2"));
    response["run_attempt"] = json!(1);
    write(
        &directory.0.join("response.json"),
        &serde_json::to_vec(&response).unwrap(),
    );
    assert!(
        eal_collectors::collect(&config, &request(input))
            .await
            .is_err()
    );
}
#[tokio::test]
async fn github_checks_preserve_pending_failure_and_bounded_page_coverage() {
    let directory = Directory::new();
    let response = json!({"total_count":200,"check_runs":(0..100).map(|id|json!({"id":id,"name":format!("check-{id}"),"head_sha":"a".repeat(40),"status":"queued","conclusion":null})).collect::<Vec<_>>()});
    let config = delivery_config(&directory, "github_checks", &response);
    let input = json!({"repository":"owner/repo","commit":"a".repeat(40)});
    let result = eal_collectors::collect(&config, &request(input.clone()))
        .await
        .unwrap();
    assert_eq!(result["value"]["reported_pages_complete"], false);
    assert_eq!(result["value"]["search_exhaustive"], false);
    assert_eq!(result["value"]["checks"][0]["conclusion"], Value::Null);
    let bad = json!({"total_count":1,"check_runs":[{"id":1,"name":"check","head_sha":"b".repeat(40),"status":"completed","conclusion":"success"}]});
    write(
        &directory.0.join("response.json"),
        &serde_json::to_vec(&bad).unwrap(),
    );
    assert!(
        eal_collectors::collect(&config, &request(input))
            .await
            .is_err()
    );
}

#[test]
fn third_catalogue_is_additive_and_all_extended_descriptors_are_runtime_exportable() {
    let old = eal_collectors::catalogue::native_version("2").unwrap();
    let new = eal_collectors::catalogue::native_version("3").unwrap();
    new.validate_for(eal_collector_contracts::AuthoringProfile::RuntimeExportable)
        .unwrap();
    for descriptor in &old.collectors {
        assert_eq!(
            serde_json::to_value(descriptor).unwrap(),
            serde_json::to_value(
                new.collectors
                    .iter()
                    .find(|v| v.id == descriptor.id)
                    .unwrap()
            )
            .unwrap()
        );
    }
    for id in [
        "datadog_metrics",
        "datadog_logs",
        "datadog_spans",
        "datadog_monitor",
        "datadog_slo",
        "prometheus_range",
        "loki_search",
        "github_checks",
        "github_workflow",
        "report",
    ] {
        assert!(new.collectors.iter().any(|d| d.id == id));
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_eal-collector"))
        .args(["describe", "--catalogue-version", "3"])
        .env_clear()
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        serde_json::to_value(new).unwrap()
    );
}

#[test]
fn github_ci_cannot_retain_encoded_forwarded_credentials() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let directory = Directory::new();
    let secret = "fixture-gh-token-612903";
    let response = json!({"id":17,"run_attempt":2,"head_sha":"a".repeat(40),"repository":{"full_name":"owner/repo"},"status":"completed","conclusion":"failure","display_title":format!("{{\"nested\":\"{secret}\"}}")});
    let executable = gh(&directory, &response);
    let mut child = Command::new(env!("CARGO_BIN_EXE_eal-collector"))
        .args([
            "github_workflow",
            "--contract-version",
            "2",
            "--executable",
            executable.to_str().unwrap(),
            "--supervisor",
            supervisor().to_str().unwrap(),
            "--repository",
            "owner/repo",
            "--commit",
            &"a".repeat(40),
            "--run-id",
            "17",
            "--attempt",
            "2",
            "--inherit-env",
            "PATH",
            "--inherit-env",
            "GITHUB_TOKEN",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GITHUB_TOKEN", secret)
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
            &serde_json::to_vec(&request(
                json!({"repository":"owner/repo","commit":"a".repeat(40),"run_id":17,"attempt":2}),
            ))
            .unwrap(),
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&output.stderr).contains(secret));
}
