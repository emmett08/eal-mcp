//! Actual server and host configuration failures preserve operator confidentiality.
use std::{path::PathBuf, process::Command};
struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[test]
fn malformed_server_configuration_and_limits_do_not_echo_credentials() {
    let path = std::env::temp_dir().join(format!("eal-safe-server-config-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let directory = Directory(path);
    let secret = "synthetic-server-credential";
    for invalid in [
        format!("[server.http]\ntoken_env=\"{secret}\n"),
        format!("[server.http]\ntoken_env=\"{secret}\"\ninvalid=\n"),
        format!("[server.http]\ntoken_env=\"{secret}\"\ntoken_env=\"duplicate\"\n"),
    ] {
        let config = directory.0.join("server.toml");
        std::fs::write(&config, invalid).unwrap();
        for binary in [
            env!("CARGO_BIN_EXE_eal-mcp"),
            env!("CARGO_BIN_EXE_eal-host"),
        ] {
            let output = Command::new(binary)
                .arg("--config")
                .arg(&config)
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
                stdout.contains("EAL_SERVER_CONFIGURATION_TOML")
                    || stderr.contains("EAL_SERVER_CONFIGURATION_TOML"),
                "{binary}: stdout={stdout} stderr={stderr}"
            );
        }
    }
    for invalid in [
        format!("[limits]\nrecord_bytes=\"{secret}\"\n"),
        format!("[limits]\nrecord_bytes=\"{secret}\n"),
    ] {
        let limits = directory.0.join("limits.toml");
        std::fs::write(&limits, invalid).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_eal-mcp"))
            .arg("--workspace")
            .arg(&directory.0)
            .arg("--limits")
            .arg(&limits)
            .output()
            .unwrap();
        assert!(!output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stdout.contains(secret) && !stderr.contains(secret),
            "stdout={stdout} stderr={stderr}"
        );
        assert!(
            stdout.contains("EAL_LIMITS_TOML") || stderr.contains("EAL_LIMITS_TOML"),
            "stdout={stdout} stderr={stderr}"
        );
    }
}
