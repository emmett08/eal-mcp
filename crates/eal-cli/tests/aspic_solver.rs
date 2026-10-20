//! Native visualiser protocol: no Python, Java, workspace state or ambient budgets.
use serde_json::{Value, json};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

const MAX_REQUEST_BYTES: usize = 64 * 1024;
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "eal-aspic-solver-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn command(&self, arguments: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_eal-aspic-solve"));
        command
            .args(arguments)
            .current_dir(&self.0)
            .env_clear()
            .env("PATH", "")
            .env("EAL_LIMITS_FILE", self.0.join("ambient-missing.toml"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    fn run(&self, bytes: &[u8], arguments: &[&str]) -> Output {
        let mut child = self.command(arguments).spawn().unwrap();
        let _ = child.stdin.take().unwrap().write_all(bytes);
        child.wait_with_output().unwrap()
    }

    fn json(&self, value: &Value, arguments: &[&str]) -> Output {
        self.run(&serde_json::to_vec(value).unwrap(), arguments)
    }

    fn write(&self, name: &str, value: &str) {
        std::fs::write(self.0.join(name), value).unwrap();
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn successful(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    assert!(output.stdout.len() <= 4 * 1024 * 1024);
    eal_core::json::parse(
        std::str::from_utf8(&output.stdout).unwrap(),
        &eal_core::ExecutionLimits::default(),
    )
    .unwrap()
}

fn failed(output: Output, exit: i32, code: &str) -> String {
    assert_eq!(output.status.code(), Some(exit));
    assert!(output.stdout.is_empty(), "failed computation leaked a view");
    assert!(output.stderr.len() <= 12 * 1024);
    assert_eq!(output.stderr.last(), Some(&b'\n'));
    let envelope = eal_core::json::parse(
        std::str::from_utf8(&output.stderr).unwrap(),
        &eal_core::ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(envelope.as_object().unwrap().len(), 1);
    assert_eq!(envelope["error"].as_object().unwrap().len(), 2);
    assert_eq!(envelope["error"]["code"], code);
    envelope["error"]["message"].as_str().unwrap().to_owned()
}

fn simple() -> Value {
    json!({"theory":{
        "premises":[{"atom":"p","kind":"axiom"}],
        "rules":[{"id":"derive","kind":"strict","antecedents":["p"],"consequent":"q"}],
        "contraries":[],"goal":"q"
    }})
}

#[test]
fn installed_protocol_matches_frozen_complete_exports_with_the_method_profile() {
    let directory = Directory::new();
    let reference: Value = serde_json::from_str(include_str!(
        "../../../fixtures/aspic/export-reference.json"
    ))
    .unwrap();
    assert_eq!(
        reference["source_revision"],
        "e2d6f346a37f92483b9f9a13923b98feb053447b"
    );
    let mut count = 0;
    for case in reference["cases"].as_array().unwrap() {
        if !case["name"].as_str().unwrap().starts_with("formal-") {
            continue;
        }
        let mut expected = case["view"].clone();
        // The process adapter, like the source adapter, declares the method
        // selector. The retained export oracle otherwise uses its default profile.
        expected["profile"] = json!(eal_aspic::METHOD);
        let payload = json!({"theory":case["input"]["theory"]});
        let actual = successful(directory.json(&payload, &[]));
        assert_eq!(actual, expected, "{}", case["name"]);
        count += 1;
    }
    assert_eq!(count, 5);
    assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 0);
}

#[test]
fn strict_utf8_unique_keys_closed_shape_and_finite_numbers_are_admitted_before_solving() {
    let directory = Directory::new();
    for bytes in [
        b"[]".as_slice(),
        b"{}",
        b"null",
        b"{\"theory\":null}",
        b"{\"theory\":{},\"formal\":{}}",
        b"{\"theory\":{},\"theory\":{}}",
        b"{\"theory\":{\"goal\":\"p\",\"goal\":\"q\"}}",
        b"{\"theory\":{\"goal\":NaN}}",
        b"{\"theory\":{\"goal\":1e400}}",
        b"{\"theory\":{\"goal\":\"\xff\"}}",
        b"{\"theory\":{}} {}",
        b"{\"theory\":{\"$serde_json::private::Number\":\"1\"}}",
    ] {
        failed(directory.run(bytes, &[]), 2, "invalid_request");
    }
    let oversized_diagnostic = json!({"theory":{"\u{0001}".repeat(10_000):null}});
    let message = failed(
        directory.json(&oversized_diagnostic, &[]),
        2,
        "invalid_request",
    );
    assert!(message.chars().count() <= 1800);
}

#[test]
fn hard_request_limit_includes_whitespace_and_configured_json_limits_still_apply() {
    let directory = Directory::new();
    let mut exact = serde_json::to_vec(&simple()).unwrap();
    exact.resize(MAX_REQUEST_BYTES, b' ');
    assert_eq!(
        successful(directory.run(&exact, &[]))["query_status"],
        "accepted"
    );
    exact.push(b' ');
    assert!(failed(directory.run(&exact, &[]), 2, "invalid_request").contains("64 KiB"));

    directory.write("small.toml", "[limits]\nrecord_bytes = 8\n");
    assert!(
        failed(
            directory.json(&simple(), &["--limits", "small.toml"]),
            2,
            "invalid_request"
        )
        .contains("byte budget")
    );
    directory.write("depth.toml", "[limits]\njson_depth = 1\n");
    failed(
        directory.json(&simple(), &["--limits", "depth.toml"]),
        2,
        "invalid_request",
    );
    directory.write(
        "raised.toml",
        "[limits]\njson_depth = 1000000\nrecord_bytes = 1000000\n",
    );
    let deep = format!("{{\"theory\":{}0{}}}", "[".repeat(200), "]".repeat(200));
    failed(
        directory.run(deep.as_bytes(), &["--limits", "raised.toml"]),
        2,
        "invalid_request",
    );
}

#[test]
fn explicit_budgets_preserve_typed_incomplete_without_a_partial_view() {
    let directory = Directory::new();
    assert_eq!(
        successful(directory.json(&simple(), &[]))["query_status"],
        "accepted"
    );
    directory.write("limits.toml", "[limits]\nformal_arguments = 1\n");
    let message = failed(
        directory.json(&simple(), &["--limits", "limits.toml"]),
        1,
        "incomplete",
    );
    assert!(message.contains("argument budget"), "{message}");

    let odd_cycle = json!({"theory":{
        "premises":[{"atom":"a","kind":"ordinary","rank":500},{"atom":"b","kind":"ordinary","rank":500},{"atom":"c","kind":"ordinary","rank":500}],
        "rules":[],"contraries":[{"attacker":"a","target":"b"},{"attacker":"b","target":"c"},{"attacker":"c","target":"a"}],"goal":"a",
        "options":{"semantics":"stable","query_mode":"sceptical"}
    }});
    let complete = successful(directory.json(&odd_cycle, &[]));
    assert_eq!(complete["query_status"], "no_extension");
    assert_eq!(complete["extensions"], json!([]));
    directory.write("search.toml", "[limits]\nextension_search = 1\n");
    failed(
        directory.json(&odd_cycle, &["--limits", "search.toml"]),
        1,
        "incomplete",
    );
}

#[test]
fn empty_graph_rejection_is_distinct_from_unconstructed_goals_and_empty_extensions() {
    let directory = Directory::new();
    let empty = json!({"theory":{"premises":[],"rules":[],"contraries":[],"goal":"p"}});
    failed(directory.json(&empty, &[]), 2, "invalid_request");
    let unrelated = json!({"theory":{"premises":[{"atom":"q","kind":"axiom"}],"rules":[],"contraries":[],"goal":"p"}});
    assert_eq!(
        successful(directory.json(&unrelated, &[]))["query_status"],
        "unconstructed"
    );
    let preferred = json!({"theory":{
        "premises":[{"atom":"a","kind":"ordinary","rank":500},{"atom":"b","kind":"ordinary","rank":500},{"atom":"c","kind":"ordinary","rank":500}],
        "rules":[],"contraries":[{"attacker":"a","target":"b"},{"attacker":"b","target":"c"},{"attacker":"c","target":"a"}],"goal":"a",
        "options":{"semantics":"preferred"}
    }});
    assert_eq!(
        successful(directory.json(&preferred, &[]))["extensions"],
        json!([[]])
    );
}

#[test]
fn operator_configuration_errors_and_unavailable_files_have_closed_error_categories() {
    let directory = Directory::new();
    for arguments in [
        vec!["--help"],
        vec!["--limits"],
        vec!["--limits", "x", "extra"],
        vec!["file.json"],
    ] {
        failed(directory.json(&simple(), &arguments), 2, "invalid_request");
    }
    failed(
        directory.json(&simple(), &["--limits", "missing.toml"]),
        3,
        "unavailable",
    );
    for (name, text) in [
        ("unknown.toml", "[limits]\nunknown = 1\n"),
        ("zero.toml", "[limits]\nformal_arguments = 0\n"),
        ("bool.toml", "[limits]\nformal_arguments = true\n"),
        (
            "duplicate.toml",
            "[limits]\nformal_arguments = 1\nformal_arguments = 2\n",
        ),
        (
            "extra.toml",
            "[limits]\nformal_arguments = 1\n[unexpected]\na = 1\n",
        ),
        ("large.toml", &" ".repeat(16 * 1024 + 1)),
    ] {
        directory.write(name, text);
        failed(
            directory.json(&simple(), &["--limits", name]),
            2,
            "invalid_request",
        );
    }
}

#[cfg(unix)]
#[test]
fn a_fifo_limits_path_is_rejected_without_waiting_for_a_writer() {
    use std::time::{Duration, Instant};
    let directory = Directory::new();
    rustix::fs::mkfifoat(
        rustix::fs::CWD,
        directory.0.join("fifo"),
        rustix::fs::Mode::RUSR,
    )
    .unwrap();
    let mut child = directory.command(&["--limits", "fifo"]).spawn().unwrap();
    drop(child.stdin.take());
    let deadline = Instant::now() + Duration::from_secs(2);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("limits FIFO blocked the request");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    failed(child.wait_with_output().unwrap(), 2, "invalid_request");
}
