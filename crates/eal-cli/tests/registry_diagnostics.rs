//! Credential-safe registry rejection through both installed CLI entry points.
use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[test]
fn malformed_registry_credentials_are_absent_from_cli_streams_and_json_errors() {
    let path = std::env::temp_dir().join(format!(
        "eal-safe-registry-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).unwrap();
    let directory = Directory(path);
    std::fs::write(
        directory.0.join("case.eal"),
        "language \"EAL/4\"\nenvironment lab { require true\n}\n",
    )
    .unwrap();
    let secret = "synthetic-review-secret-cli";
    for invalid in [
        format!("env={{LAB_API_TOKEN=\"{secret}\",BROKEN=}}"),
        format!("env={{LAB_API_TOKEN=\"{secret}"),
        format!("env={{LAB_API_TOKEN=\"\"\"{secret}\n"),
        format!("env={{LAB_API_TOKEN=\"{secret}\",LAB_API_TOKEN=\"{secret}\"}}"),
        format!("\"κλειδί\"=\"{secret}\"\ninvalid="),
    ] {
        let registry = directory.0.join("tools.toml");
        std::fs::write(
            &registry,
            format!(
                "[tools.reader]\nkind=\"command\"\nversion=\"1\"\nargv=[\"reader\"]\n{invalid}\n"
            ),
        )
        .unwrap();
        for (binary, args) in [
            (
                env!("CARGO_BIN_EXE_eal-runtime"),
                vec!["validate", "case.eal"],
            ),
            (
                env!("CARGO_BIN_EXE_eal"),
                vec!["collect", "case.eal", "--context", "{}"],
            ),
        ] {
            let result = Command::new(binary)
                .current_dir(&directory.0)
                .arg("--workspace")
                .arg(&directory.0)
                .arg("--registry")
                .arg(&registry)
                .args(args)
                .output()
                .unwrap();
            assert!(!result.status.success());
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            assert!(
                !stdout.contains(secret) && !stderr.contains(secret),
                "stdout={stdout} stderr={stderr}"
            );
            assert!(
                stdout.contains("EAL_REGISTRY_SYNTAX"),
                "stdout={stdout} stderr={stderr}"
            );
            assert!(!stdout.contains("LAB_API_TOKEN") && !stderr.contains("LAB_API_TOKEN"));
        }
    }
}
