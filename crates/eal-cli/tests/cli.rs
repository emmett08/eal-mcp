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
