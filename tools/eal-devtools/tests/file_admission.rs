//! Smoke input files are admitted before optional tools or process effects.
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-smoke-files-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn command(directory: &Directory, target: &std::path::Path, views: bool) -> String {
    let stdout = directory.0.join("stdout");
    let stderr = directory.0.join("stderr");
    let mut command = Command::new(env!("CARGO_BIN_EXE_eal-smoke"));
    command
        .args(["consumer", "--consumer"])
        .arg(if views {
            directory.0.join("consumer-must-not-open")
        } else {
            target.to_owned()
        })
        .arg("--binary")
        .arg(directory.0.join("binary-must-not-start"));
    if views {
        command.arg("--views").arg(target);
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(fs::File::create(&stdout).unwrap())
        .stderr(fs::File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() > Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("Smoke file admission blocked");
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    assert!(!status.success());
    let message = fs::read_to_string(stderr).unwrap();
    assert!(
        !message.contains("binary-must-not-start"),
        "Input was not checked before executable discovery: {message}"
    );
    message
}
#[test]
fn consumer_and_view_files_have_exact_and_sparse_size_bounds() {
    for (views, limit) in [(false, 4 * 1024 * 1024), (true, 64 * 1024 * 1024)] {
        let directory = Directory::new();
        let target = directory.0.join("input");
        fs::File::create(&target)
            .unwrap()
            .set_len(limit + 1)
            .unwrap();
        assert!(command(&directory, &target, views).contains("byte allowance"));
        let mut file = fs::File::create(&target).unwrap();
        file.write_all(b"{}").unwrap();
        // Fill with valid JSON trailing whitespace rather than a sparse zero tail.
        let chunk = vec![b' '; 8192];
        let mut remaining = limit - 2;
        while remaining > 0 {
            let count = remaining.min(chunk.len() as u64) as usize;
            file.write_all(&chunk[..count]).unwrap();
            remaining -= count as u64;
        }
        drop(file);
        assert!(!command(&directory, &target, views).contains("byte allowance"));
    }
}
#[test]
fn smoke_json_loader_rejects_non_utf8_duplicate_keys_missing_and_directory() {
    let directory = Directory::new();
    let input = directory.0.join("input");
    for bytes in [&b"\xff"[..], &b"{"[..], &b"{\"key\":1,\"key\":2}"[..]] {
        fs::write(&input, bytes).unwrap();
        assert!(eal_devtools::smoke::read_json(&input).is_err());
    }
    assert!(eal_devtools::smoke::read_json(&directory.0).is_err());
    assert!(eal_devtools::smoke::read_json(&directory.0.join("missing")).is_err());
    fs::write(&input, b"{}").unwrap();
    assert_eq!(
        eal_devtools::smoke::read_json(&input).unwrap(),
        serde_json::json!({})
    );
}
#[cfg(unix)]
#[test]
fn public_smoke_views_and_consumer_reject_fifos_and_leaf_aliases() {
    use std::os::unix::fs::symlink;
    let directory = Directory::new();
    let fifo = directory.0.join("input.fifo");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let alias = directory.0.join("fifo-alias");
    symlink(&fifo, &alias).unwrap();
    let regular = directory.0.join("regular.json");
    fs::write(&regular, b"{}").unwrap();
    let regular_alias = directory.0.join("regular-alias");
    symlink(&regular, &regular_alias).unwrap();
    for target in [&fifo, &alias, &regular_alias] {
        for views in [true, false] {
            assert!(command(&directory, target, views).contains("regular file"));
        }
    }
}
