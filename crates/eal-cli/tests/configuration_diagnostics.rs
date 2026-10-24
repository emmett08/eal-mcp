//! Public limits entry points exclude malformed operator input from diagnostics.
use std::{path::PathBuf, process::Command};
struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[test]
fn malformed_limits_credentials_are_absent_from_every_public_cli_stream() {
    let path = std::env::temp_dir().join(format!("eal-safe-limits-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let directory = Directory(path);
    std::fs::write(
        directory.0.join("case.eal"),
        "language \"EAL/3\"\nenvironment lab { require true\n}\n",
    )
    .unwrap();
    let secret = "synthetic-limits-credential";
    for invalid in [
        format!("[limits]\nrecord_bytes=\"{secret}\"\n"),
        format!("[limits]\nrecord_bytes=\"{secret}\n"),
        format!("[limits]\nunknown=\"{secret}\"\n"),
        format!("[limits]\nrecord_bytes=1\nrecord_bytes=\"{secret}\"\n"),
        format!("[limits]\n\"κλειδί\"=\"{secret}\"\ninvalid=\n"),
    ] {
        std::fs::write(directory.0.join("limits.toml"), invalid).unwrap();
        for (binary, args) in [
            (
                env!("CARGO_BIN_EXE_eal"),
                vec![
                    "--workspace",
                    directory.0.to_str().unwrap(),
                    "--limits",
                    "limits.toml",
                    "validate",
                    "case.eal",
                ],
            ),
            (
                env!("CARGO_BIN_EXE_eal-runtime"),
                vec![
                    "--workspace",
                    directory.0.to_str().unwrap(),
                    "--limits",
                    "limits.toml",
                    "validate",
                    "case.eal",
                ],
            ),
            (
                env!("CARGO_BIN_EXE_eal-core"),
                vec![
                    "--workspace",
                    directory.0.to_str().unwrap(),
                    "--limits",
                    "limits.toml",
                    "validate",
                    "case.eal",
                ],
            ),
            (
                env!("CARGO_BIN_EXE_eal-aspic-solve"),
                vec!["--limits", "limits.toml"],
            ),
        ] {
            let output = Command::new(binary)
                .current_dir(&directory.0)
                .args(args)
                .output()
                .unwrap();
            assert!(!output.status.success());
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !stdout.contains(secret) && !stderr.contains(secret),
                "{binary}: stdout={stdout} stderr={stderr}"
            );
            assert!(
                stdout.contains("EAL_LIMITS_TOML") || stderr.contains("EAL_LIMITS_TOML"),
                "{binary}: stdout={stdout} stderr={stderr}"
            );
        }
    }
}
