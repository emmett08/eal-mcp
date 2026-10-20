use eal_core::ExecutionLimits;
use eal_runtime::{acquisition::*, process::Supervisor};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::PathBuf};

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-acquisition-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) {
        fs::write(self.0.join(name), bytes).unwrap()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn reference() -> Value {
    eal_core::json::parse(
        include_str!("../../../fixtures/runtime/acquisition-reference.json"),
        &ExecutionLimits::default(),
    )
    .unwrap()
}
fn secret() -> [u8; 32] {
    [b'x'; 32]
}

#[test]
fn complete_registry_validation_and_private_binding_identities_match_python() {
    let fixture = reference();
    let workspace = Workspace::new();
    for (name, value) in fixture["pinned_files"].as_object().unwrap() {
        workspace.write(name, value.as_str().unwrap())
    }
    for case in fixture["registry"].as_array().unwrap() {
        let actual = ToolRegistry::from_toml(case["toml"].as_str().unwrap());
        if let Some(error) = case.get("error") {
            let actual = actual.unwrap_err();
            assert_eq!(actual.type_name(), error["type"], "{}", case["name"]);
            assert_eq!(actual.message, error["message"], "{}", case["name"]);
        } else {
            let actual = actual.unwrap_or_else(|e| panic!("{}: {e}", case["name"]));
            for (name, expected) in case["value"].as_object().unwrap() {
                let b = actual
                    .binding_for(name, expected["binding"]["version"].as_str().unwrap())
                    .unwrap();
                let fields = json!({"name":b.name,"kind":b.kind.as_str(),"version":b.version,"argv":b.argv,"path":b.path,
                    "timeout_seconds":b.timeout_seconds,"max_output_bytes":b.max_output_bytes,"env":b.env,
                    "pinned_files":b.pinned_files.iter().map(|p|json!([p.path,p.sha256])).collect::<Vec<_>>(),
                    "parallel_safe":b.parallel_safe,"inherit_env":b.inherit_env});
                assert_eq!(fields, expected["binding"], "{}", case["name"]);
                assert_eq!(
                    b.binding_digest(&secret(), Some(&workspace.0)).unwrap(),
                    expected["digest"],
                    "{}",
                    case["name"]
                );
                let environment = BTreeMap::from([
                    ("TOKEN".into(), "synthetic-secret".into()),
                    ("LANG".into(), "C".into()),
                ]);
                assert_eq!(
                    json!(
                        b.process_environment_digest(&secret(), Some(&environment))
                            .unwrap()
                    ),
                    expected["environment_digest"],
                    "{}",
                    case["name"]
                );
            }
        }
    }
}

#[test]
fn keyed_domains_and_exact_ascii_request_bytes_match_python() {
    let fixture = reference();
    for case in fixture["digests"].as_array().unwrap() {
        assert_eq!(
            keyed_digest(
                &secret(),
                case["domain"].as_str().unwrap().as_bytes(),
                &case["value"]
            )
            .unwrap(),
            case["digest"],
            "{}",
            case["name"]
        );
    }
    for case in fixture["requests"].as_array().unwrap() {
        assert_eq!(
            encode_request(&case["value"], &ExecutionLimits::default()).unwrap(),
            case["encoded"].as_str().unwrap().as_bytes()
        );
    }
    assert!(keyed_digest(&secret()[..31], b"tool-binding", &Value::Null).is_err());
    assert!(keyed_digest(&secret(), b"", &Value::Null).is_err());
}

#[test]
fn strict_json_and_envelope_correspondence_match_python() {
    let fixture = reference();
    for case in fixture["strict_json"].as_array().unwrap() {
        let actual = strict_json(case["text"].as_str().unwrap());
        if let Some(expected) = case.get("encoded_value") {
            assert_eq!(
                serde_json::to_string(&actual.unwrap_or_else(|e| panic!("{}: {e}", case["name"])))
                    .unwrap(),
                expected.as_str().unwrap(),
                "{}",
                case["name"]
            );
        } else if let Some(expected) = case.get("value") {
            assert_eq!(
                &actual.unwrap_or_else(|e| panic!("{}: {e}", case["name"])),
                expected,
                "{}",
                case["name"]
            )
        } else {
            let error = actual.expect_err(case["name"].as_str().unwrap());
            assert_eq!(
                error.type_name(),
                case["error"]["type"],
                "{}: {error}",
                case["name"]
            )
        }
    }
    for case in fixture["envelopes"].as_array().unwrap() {
        let actual = validate_envelope(
            case["text"].as_str().unwrap().as_bytes(),
            case["file_import"].as_bool().unwrap(),
            &case["context"],
            &case["acquisition"],
        );
        if let Some(expected) = case.get("value") {
            assert_eq!(&actual.unwrap(), expected, "{}", case["name"])
        } else {
            let error = actual.expect_err(case["name"].as_str().unwrap());
            assert_eq!(error.as_record(), case["error"], "{}", case["name"])
        }
    }
}

#[test]
fn pinned_files_are_rechecked_and_relative_imports_remain_workspace_bound() {
    let workspace = Workspace::new();
    workspace.write("collector.bin", b"first");
    let mut binding = ToolBinding::command("reader", "1", vec!["collector".into()]);
    binding.pinned_files.push(PinnedFile {
        path: "collector.bin".into(),
        sha256: format!("{:x}", Sha256::digest(b"first")),
    });
    let before = binding
        .binding_digest(&secret(), Some(&workspace.0))
        .unwrap();
    assert_eq!(
        before,
        binding
            .binding_digest(&secret(), Some(&workspace.0))
            .unwrap()
    );
    workspace.write("collector.bin", b"second");
    assert!(
        binding
            .binding_digest(&secret(), Some(&workspace.0))
            .unwrap_err()
            .message
            .contains("identity differs")
    );
    assert!(
        binding
            .binding_digest(&secret(), None)
            .unwrap_err()
            .message
            .contains("require a workspace")
    );
    assert!(
        bounded_path(&workspace.0, "../outside")
            .unwrap_err()
            .message
            .contains("outside the workspace")
    );
    binding.pinned_files[0].path = ".".into();
    assert!(
        binding
            .binding_digest(&secret(), Some(&workspace.0))
            .unwrap_err()
            .message
            .contains("regular file")
    );
    let large = fs::File::create(workspace.0.join("large")).unwrap();
    large.set_len(MAX_PINNED_FILE_BYTES as u64 + 1).unwrap();
    binding.pinned_files[0].path = "large".into();
    assert!(
        binding
            .binding_digest(&secret(), Some(&workspace.0))
            .unwrap_err()
            .message
            .contains("16 MiB")
    );
}

#[test]
fn file_adapter_retains_raw_bytes_errors_and_truncation_metadata() {
    let workspace = Workspace::new();
    workspace.write("observation.json", b"{\"value\":true}");
    let supervisor = Supervisor::new(PathBuf::from(env!("CARGO_BIN_EXE_eal-command-supervisor")));
    let mut binding = ToolBinding::json_file("reader", "1", "observation.json");
    let result = acquire(
        &binding,
        &json!({}),
        &workspace.0,
        &secret(),
        &supervisor,
        None,
    )
    .unwrap();
    assert_eq!(result.stdout, b"{\"value\":true}");
    assert!(result.error.is_none());
    assert_eq!(result.metadata, json!({"file":"observation.json"}));
    binding.max_output_bytes = 5;
    let result = acquire(
        &binding,
        &json!({}),
        &workspace.0,
        &secret(),
        &supervisor,
        None,
    )
    .unwrap();
    assert_eq!(result.stdout, b"{\"val");
    assert_eq!(result.error.unwrap().message, "output_limit");
    assert_eq!(result.metadata["output_truncated"], true);
    binding.path = Some("missing.json".into());
    let result = acquire(
        &binding,
        &json!({}),
        &workspace.0,
        &secret(),
        &supervisor,
        None,
    )
    .unwrap();
    assert_eq!(result.error.unwrap().type_name(), "FileNotFoundError");
    assert!(result.stdout.is_empty());
}

#[cfg(unix)]
#[test]
fn relative_symlinks_cannot_escape_but_absolute_operator_pins_are_supported() {
    use std::os::unix::fs::symlink;
    let workspace = Workspace::new();
    let outside = Workspace::new();
    outside.write("artefact", b"operator");
    symlink(outside.0.join("artefact"), workspace.0.join("linked")).unwrap();
    assert!(bounded_path(&workspace.0, "linked").is_err());
    let mut binding = ToolBinding::command("reader", "1", vec!["collector".into()]);
    binding.pinned_files.push(PinnedFile {
        path: outside.0.join("artefact").to_str().unwrap().into(),
        sha256: format!("{:x}", Sha256::digest(b"operator")),
    });
    assert!(
        binding
            .binding_digest(&secret(), Some(&workspace.0))
            .is_ok()
    );
}

#[cfg(unix)]
#[test]
fn command_environment_and_execution_metadata_are_bounded_and_secret_free() {
    let workspace = Workspace::new();
    let supervisor = Supervisor::new(PathBuf::from(env!("CARGO_BIN_EXE_eal-command-supervisor")));
    let mut binding = ToolBinding::command(
        "reader",
        "1",
        vec![
            "/bin/sh".into(),
            "-c".into(),
            "printf '%s' \"$SYNTHETIC_TOKEN\"; exit 17".into(),
        ],
    );
    binding.inherit_env = Some(Vec::new());
    binding
        .env
        .insert("SYNTHETIC_TOKEN".into(), "synthetic-secret".into());
    let digest = binding.process_environment_digest(&secret(), None).unwrap();
    let result = acquire(
        &binding,
        &json!({}),
        &workspace.0,
        &secret(),
        &supervisor,
        None,
    )
    .unwrap();
    assert_eq!(result.stdout, b"synthetic-secret");
    assert_eq!(result.error.unwrap().message, "Tool exited with status 17");
    assert_eq!(result.metadata["returncode"], 17);
    assert_eq!(result.metadata["process_environment_digest"], json!(digest));
    assert!(!result.metadata.to_string().contains("synthetic-secret"));
    binding.argv[2] = "printf 'abcdefghijklmnop'".into();
    binding.max_output_bytes = 5;
    let result = acquire(
        &binding,
        &json!({}),
        &workspace.0,
        &secret(),
        &supervisor,
        None,
    )
    .unwrap();
    assert_eq!(result.stdout.len() + result.stderr.len(), 5);
    assert_eq!(result.metadata["execution_error"], "output_limit");
    assert_eq!(result.error.unwrap().message, "output_limit");
    binding.argv[2] = "while :; do :; done".into();
    binding.timeout_seconds = 0.1;
    let result = acquire(
        &binding,
        &json!({}),
        &workspace.0,
        &secret(),
        &supervisor,
        None,
    )
    .unwrap();
    assert_eq!(result.metadata["execution_error"], "timeout");
    assert_eq!(result.error.unwrap().message, "timeout");
}

#[test]
fn exact_request_limit_rejects_before_any_supervisor_is_started() {
    let limits = ExecutionLimits::default();
    // Quoted scalar + newline: three bytes beyond the content itself.
    assert_eq!(
        encode_request(&json!("x".repeat(MAX_REQUEST_BYTES - 3)), &limits)
            .unwrap()
            .len(),
        MAX_REQUEST_BYTES
    );
    assert!(
        encode_request(&json!("x".repeat(MAX_REQUEST_BYTES - 2)), &limits)
            .unwrap_err()
            .message
            .contains("Tool request exceeds")
    );
    let workspace = Workspace::new();
    let supervisor = Supervisor::new(workspace.0.join("must-not-start"));
    let binding = ToolBinding::command("reader", "1", vec!["must-not-start".into()]);
    assert!(
        acquire(
            &binding,
            &json!({"input":"x".repeat(MAX_REQUEST_BYTES)}),
            &workspace.0,
            &secret(),
            &supervisor,
            None
        )
        .unwrap_err()
        .message
        .contains("Tool request exceeds")
    );
}
