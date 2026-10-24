//! The real text host and its sibling server reject malformed registry secrets.
use std::{
    io::Write,
    process::{Command, Stdio},
};
#[test]
fn malformed_registry_does_not_disclose_credentials_through_host() {
    let directory =
        std::env::temp_dir().join(format!("eal-host-safe-registry-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let run = || {
        let mut child = Command::new(env!("CARGO_BIN_EXE_eal-host"))
            .args([
                "--workspace",
                directory.to_str().unwrap(),
                "--registry",
                directory.join("tools.toml").to_str().unwrap(),
                "--exposure",
                "operator",
                "--timeout",
                "5",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"{\"operation\":\"describe\"}")
            .unwrap();
        child.wait_with_output().unwrap()
    };
    std::fs::write(directory.join("tools.toml"), "[tools]\n").unwrap();
    let valid = run();
    assert!(
        valid.status.success(),
        "The installed host/server baseline must succeed: {}",
        String::from_utf8_lossy(&valid.stdout)
    );
    let secret = "synthetic-review-secret-host";
    std::fs::write(directory.join("tools.toml"),format!("[tools.reader]\nkind=\"command\"\nversion=\"1\"\nargv=[\"reader\"]\nenv={{LAB_API_TOKEN=\"{secret}\",BROKEN=}}\n")).unwrap();
    let output = run();
    let _ = std::fs::remove_dir_all(&directory);
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stdout.contains(secret) && !stderr.contains(secret),
        "stdout={stdout} stderr={stderr}"
    );
    assert!(!stdout.contains("LAB_API_TOKEN") && !stderr.contains("LAB_API_TOKEN"));
}
