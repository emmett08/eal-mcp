use eal_collector_contracts::PinnedFile;
use eal_tool_runtime::{
    ToolchainProfile,
    package::{PACKAGE_SCHEMA, ToolPackage, digest},
};
use serde_json::{Value, json};
use std::{fs, path::Path};
fn resource(root: &Path, path: &str, value: &[u8]) -> PinnedFile {
    fs::write(root.join(path), value).unwrap();
    PinnedFile {
        path: path.into(),
        sha256: digest(value),
    }
}
fn package(root: &Path) -> ToolPackage {
    fs::create_dir(root.join("source")).unwrap();
    let source = resource(
        &root.join("source"),
        "main.py",
        b"import json,sys\nr=json.load(sys.stdin)\n",
    );
    let input=resource(root,"input.json",br#"{"type":"object","required":["items"],"properties":{"items":{"type":"array"}},"additionalProperties":false}"#);
    let output=resource(root,"output.json",br#"{"type":"object","required":["collected","present"],"properties":{"collected":{"const":true},"present":{"type":"boolean"}}}"#);
    let verification = resource(
        root,
        "verification.json",
        br#"{"retained":"original authoring receipt"}"#,
    );
    let manifest = ToolPackage {
        schema: PACKAGE_SCHEMA.into(),
        tool: "fixture".into(),
        version: "1".into(),
        image_id: format!("sha256:{}", "0".repeat(64)),
        require_observed_at: true,
        profile: ToolchainProfile {
            schema: eal_tool_runtime::profile::TOOLCHAIN_SCHEMA.into(),
            language: "python".into(),
            docker: "/no-such-eal-test-docker".into(),
            image: "python:tag".into(),
            build_argv: vec![],
            run_argv: vec!["python3".into(), "/src/main.py".into()],
            inherit_env: vec![],
            runtime_network: "none".into(),
            timeout_seconds: 5,
            memory_mb: 64,
            output_bytes: 65536,
        },
        source: vec![source],
        input_schema: input,
        output_schema: output,
        verification,
    };
    save(root, &manifest);
    manifest
}
fn save(root: &Path, manifest: &ToolPackage) {
    fs::write(root.join("tool.toml"), toml::to_string(manifest).unwrap()).unwrap();
}
fn request() -> Vec<u8> {
    serde_json::to_vec(&json!({"evidence_id":"e1","environment":"local","tool":"operator_alias","tool_version":"1","input":{"items":[]},"context":{"source":"exact"}})).unwrap()
}
#[test]
fn relocated_source_schema_and_provenance_pins_preserve_negative_and_original_clock() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original");
    fs::create_dir(&original).unwrap();
    package(&original);
    let moved = temp.path().join("moved");
    fs::rename(&original, &moved).unwrap();
    let loaded = ToolPackage::load(&moved.join("tool.toml")).unwrap();
    let request = loaded.validate_request(&request()).unwrap();
    let negative = json!({"value":{"collected":true,"present":false,"n":18446744073709551616u128},"observed_at":"2026-01-01T00:00:00Z","context":request.context,"request":request.acquisition()});
    let result = loaded
        .validate_output(&request, &serde_json::to_vec(&negative).unwrap())
        .unwrap();
    assert_eq!(result["value"]["present"], false);
    assert_eq!(result["observed_at"], "2026-01-01T00:00:00Z");
    assert_eq!(result["value"]["n"].to_string(), "18446744073709551616");
    let mut forged = negative;
    forged["context"] = json!({"source":"changed"});
    assert!(
        loaded
            .validate_output(&request, &serde_json::to_vec(&forged).unwrap())
            .is_err()
    );
}
#[test]
fn altered_sources_schemas_receipts_and_manifest_authority_fail_admission() {
    for path in [
        "source/main.py",
        "input.json",
        "output.json",
        "verification.json",
    ] {
        let dir = tempfile::tempdir().unwrap();
        package(dir.path());
        fs::write(dir.path().join(path), b"tampered").unwrap();
        assert!(
            ToolPackage::load(&dir.path().join("tool.toml"))
                .unwrap_err()
                .to_string()
                .contains("pinned")
        );
    }
    let dir = tempfile::tempdir().unwrap();
    let mut manifest = package(dir.path());
    manifest.profile.runtime_network = "host".into();
    save(dir.path(), &manifest);
    assert!(ToolPackage::load(&dir.path().join("tool.toml")).is_err());
}
#[test]
fn external_schema_references_and_source_aliases_are_rejected_without_io() {
    let dir = tempfile::tempdir().unwrap();
    let mut manifest = package(dir.path());
    manifest.input_schema = resource(
        dir.path(),
        "input.json",
        br#"{"$ref":"https://example.invalid/external.json"}"#,
    );
    save(dir.path(), &manifest);
    assert!(
        ToolPackage::load(&dir.path().join("tool.toml"))
            .unwrap_err()
            .to_string()
            .contains("local references")
    );
    #[cfg(unix)]
    {
        let dir = tempfile::tempdir().unwrap();
        package(dir.path());
        fs::rename(dir.path().join("source"), dir.path().join("real-source")).unwrap();
        std::os::unix::fs::symlink("real-source", dir.path().join("source")).unwrap();
        assert!(ToolPackage::load(&dir.path().join("tool.toml")).is_err());
    }
}
#[test]
fn standalone_launcher_rejects_invalid_input_before_trying_docker() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let dir = tempfile::tempdir().unwrap();
    package(dir.path());
    let mut bad: Value = serde_json::from_slice(&request()).unwrap();
    bad["input"] = json!({"items":"not-array"});
    let mut child = Command::new(env!("CARGO_BIN_EXE_eal-tool-run"))
        .arg("--package")
        .arg("tool.toml")
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&bad).unwrap())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("pinned schema"));
}
