use eal_core::ExecutionLimits;
use eal_runtime::{acquisition::*, process::Supervisor};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

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
fn secret() -> [u8; 32] {
    [b'x'; 32]
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
    let supervisor = Supervisor::new(companion::path());
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
    let supervisor = Supervisor::new(companion::path());
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

#[test]
fn public_regular_file_reader_checks_path_identity_before_opening() {
    let workspace = Workspace::new();
    workspace.write("reading.json", b"identified observation");
    let root = workspace.0.canonicalize().unwrap();
    let path = root.join("reading.json");
    assert_eq!(
        read_regular(&path, 64, "regular file required").unwrap(),
        b"identified observation"
    );
    let relative = std::path::Path::new("reading.json");
    assert!(
        read_regular(relative, 64, "regular file required")
            .unwrap_err()
            .message
            .contains("absolute path")
    );
    std::fs::create_dir(root.join("nested")).unwrap();
    assert!(
        read_regular(
            &root.join("nested/../reading.json"),
            64,
            "regular file required"
        )
        .unwrap_err()
        .message
        .contains("canonical parent")
    );
}

#[path = "common/companion.rs"]
mod companion;
