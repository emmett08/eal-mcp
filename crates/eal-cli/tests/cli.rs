//! Check installed-style execution, diagnostics and bounded input through the binary.

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run(command: &str, source: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_eal-syntax"))
        .args([command, "-"])
        .env("PATH", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(source).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn runs_with_an_empty_executable_search_path() {
    let output = run(
        "parse",
        b"language \"EAL/3\"\nclaim c {\nstatement \"Scoped claim\"\nenvironment e\n}\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_object());
    let formatted = run("format", b"language \"EAL/3\"\n");
    assert!(formatted.status.success());
    assert_eq!(formatted.stdout, b"language \"EAL/3\"\n");
}

#[test]
fn lexical_errors_fail_with_no_success_output() {
    let output = run("recognise", b"language \"EAL/3\"\n@\n");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn source_limit_applies_before_parsing() {
    let output = run("recognise", &vec![b' '; 1024 * 1024 + 1]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("input limit"));
}

#[test]
fn rejects_non_utf8_input() {
    let output = run("recognise", &[0xff]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[test]
fn formatting_reports_expansion_limits_without_partial_output() {
    let source = format!(
        "language \"EAL/3\"\nenvironment e {{\nrequire {}x\n}}\n",
        "not ".repeat(63)
    );
    assert!(run("parse", source.as_bytes()).status.success());
    let output = run("format", source.as_bytes());
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn source_files_are_bounded_before_parsing_and_operator_aliases_resolve_once() {
    let directory = std::env::temp_dir().join(format!("eal-syntax-files-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("source.eal");
    let mut source = b"language \"EAL/3\"\n".to_vec();
    source.resize(1024 * 1024, b' ');
    std::fs::write(&path, &source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_eal-syntax"))
        .arg("format")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::File::create(&path)
        .unwrap()
        .set_len(1024 * 1024 + 1)
        .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_eal-syntax"))
        .arg("format")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("input limit"));
    #[cfg(unix)]
    {
        std::fs::write(&path, b"language \"EAL/3\"\n").unwrap();
        let alias = directory.join("source-alias");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(
            Command::new(env!("CARGO_BIN_EXE_eal-syntax"))
                .arg("format")
                .arg(alias)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[test]
fn syntax_source_fifo_is_rejected_without_waiting_for_a_writer() {
    use std::time::{Duration, Instant};
    let directory = std::env::temp_dir().join(format!("eal-syntax-fifo-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let fifo = directory.join("source.fifo");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let alias = directory.join("fifo-alias");
    std::os::unix::fs::symlink(&fifo, &alias).unwrap();
    for path in [&fifo, &alias] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_eal-syntax"))
            .arg("parse")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let started = Instant::now();
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(!status.success());
                break;
            }
            if started.elapsed() > Duration::from_secs(5) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("Source FIFO admission blocked");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}
